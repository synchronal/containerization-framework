//! A container booted from the suite's image, and running processes in it.

use super::lock::Lock;
use super::network;
use super::store;
use containerization_framework as cfw;
use std::io::Read;
use std::os::fd::AsRawFd;
use std::os::fd::IntoRawFd;
use std::os::fd::RawFd;

/// Holds a container open; the base's own `Cmd` would exit at once.
const KEEPALIVE: [&str; 3] = ["/bin/sh", "-c", "while :; do sleep 86400; done"];

const UNPACK_LOCK: &str = ".unpack.lock";

/// A booted container, and the session that owns it.
///
/// The VM dies with this process whatever happens here, so dropping one is
/// about the store: its rootfs clone would otherwise be left behind per run.
pub struct Container {
  session: cfw::Session,
  name: String,
}

impl Container {
  /// Boots [`store::TEST_IMAGE`] under a name unique across the suite — it
  /// names the container and its directory in the shared store — held open by
  /// a process that outlives every `exec`.
  ///
  /// A first boot unpacks the image, which is the slow part of a cold run, and
  /// concurrent tests would each do it again. So the unpack alone is
  /// serialized: a boot that finds it done waits for nobody.
  pub fn boot(name: &str) -> Self {
    Self::boot_with(name, |_| {})
  }

  /// The same, with the spec changed by `configure` before it boots.
  pub fn boot_with(name: &str, configure: impl FnOnce(&mut cfw::BootSpec)) -> Self {
    Self::try_boot_with(name, configure).unwrap_or_else(|error| panic!("{name} should boot: {error}"))
  }

  /// The same, reporting a boot that fails rather than panicking. The
  /// container's directory goes either way.
  pub fn try_boot_with(name: &str, configure: impl FnOnce(&mut cfw::BootSpec)) -> Result<Self, cfw::Error> {
    Self::try_boot_in(store::image(), name, configure)
  }

  /// The same, from `store` rather than the suite's own, which must already
  /// hold [`store::TEST_IMAGE`].
  pub fn try_boot_in(
    store: cfw::Store,
    name: &str,
    configure: impl FnOnce(&mut cfw::BootSpec),
  ) -> Result<Self, cfw::Error> {
    let session = cfw::Session::new(store);
    let mut unpacking = None;

    if !is_unpacked(&session) {
      let lock = Lock::take(session.store().root().join(UNPACK_LOCK));

      // Whoever held it may have just unpacked the image.
      if !is_unpacked(&session) {
        unpacking = Some(lock);
      }
    }

    let mut spec = cfw::BootSpec::new(name, store::TEST_IMAGE);

    spec.rootfs_size_in_bytes = super::TEST_ROOTFS_SIZE_IN_BYTES;
    spec.vm = super::vm();
    spec.configuration.process = cfw::model::LinuxProcessConfiguration::new(&KEEPALIVE);
    spec.configuration.cpus = super::TEST_CPUS;
    spec.configuration.memory_in_bytes = super::TEST_MEMORY_IN_BYTES;
    spec.configuration.interfaces = vec![network::interface(name)];
    spec.configuration.dns = Some(network::gateway_dns());

    configure(&mut spec);

    let booted = session.boot(&spec);

    // Dropped here, not at the end of the test: the next boot waits on the
    // unpack, not on this container.
    drop(unpacking);

    let container = Self {
      session,
      name: name.to_string(),
    };

    // Returned only now, so a failed boot's directory goes with `container`.
    booted.map(|_| container)
  }

  pub fn session(&self) -> &cfw::Session {
    &self.session
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  /// Runs a command with nothing attached, reporting its exit code.
  pub fn exec(&self, id: &str, arguments: &[&str]) -> i32 {
    self
      .session
      .exec(
        &self.name,
        id,
        &cfw::model::LinuxProcessConfiguration::new(arguments),
        cfw::Stdio::nothing(),
      )
      .unwrap_or_else(|error| panic!("{arguments:?} should run in {}: {error}", self.name))
  }

  /// The same, reporting what it wrote to stdout.
  ///
  /// The guest writes into a pipe of this test's making, since this crate hands
  /// descriptors over rather than relaying bytes. Nothing drains it until the
  /// process exits, so what is run must write less than a pipe holds.
  pub fn capture(&self, id: &str, arguments: &[&str]) -> String {
    self.capture_with(id, &cfw::model::LinuxProcessConfiguration::new(arguments))
  }

  /// The same, for a process configured beyond its arguments.
  pub fn capture_with(&self, id: &str, configuration: &cfw::model::LinuxProcessConfiguration) -> String {
    let pipe = Pipe::new();
    let stdio = cfw::Stdio {
      terminal: cfw::UNATTACHED,
      stdin: cfw::UNATTACHED,
      stdout: pipe.write,
      stderr: cfw::UNATTACHED,
    };

    let arguments = &configuration.arguments;
    let code = self
      .session
      .exec(&self.name, id, configuration, stdio)
      .unwrap_or_else(|error| panic!("{arguments:?} should run in {}: {error}", self.name));

    assert_eq!(code, 0, "{arguments:?} failed in {}", self.name);

    pipe.drain()
  }

  /// What a `/bin/sh` script wrote to stdout, which it must exit 0 from.
  pub fn sh(&self, id: &str, script: &str) -> String {
    self.capture(id, &["/bin/sh", "-c", script])
  }
}

impl Drop for Container {
  fn drop(&mut self) {
    let _ = std::fs::remove_dir_all(self.session.store().container_dir(&self.name));
  }
}

/// Whether a boot would clone a rootfs rather than unpack one. A store that
/// cannot say counts as not unpacked; the boot after it reports what is wrong.
fn is_unpacked(session: &cfw::Session) -> bool {
  session.is_unpacked(store::TEST_IMAGE).unwrap_or(false)
}

/// A pipe whose write end a guest process is given.
///
/// Never read to end-of-file: something still holds the write end open when
/// `exec` returns, so that blocks forever. It has returned by then, so a
/// non-blocking drain gets everything the guest wrote.
///
/// The write end is released, not closed: `exec` closes what it is handed.
struct Pipe {
  read: std::io::PipeReader,
  write: RawFd,
}

impl Pipe {
  fn new() -> Self {
    let (read, write) = std::io::pipe().expect("a pipe should be creatable");

    Self {
      read,
      write: write.into_raw_fd(),
    }
  }

  /// Everything already in the pipe, stopping where a read would block.
  fn drain(mut self) -> String {
    // SAFETY: the read end is ours; `F_SETFL` only changes how it reads.
    unsafe { libc::fcntl(self.read.as_raw_fd(), libc::F_SETFL, libc::O_NONBLOCK) };

    let mut written = Vec::new();
    let mut chunk = [0u8; 8192];

    loop {
      match self.read.read(&mut chunk) {
        Ok(0) => break,
        Ok(read) => written.extend_from_slice(&chunk[..read]),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
        Err(error) => panic!("the guest's output should be readable: {error}"),
      }
    }

    String::from_utf8_lossy(&written).into_owned()
  }
}
