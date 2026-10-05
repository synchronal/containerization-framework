//! A container made from the suite's image, and processes run in it.

use super::network;
use super::store;
use containerization_framework as cfw;
use std::io::Read;
use std::os::fd::AsRawFd;

/// Holds a container open; the image's own `Cmd` would exit at once.
const KEEPALIVE: [&str; 3] = ["/bin/sh", "-c", "while :; do sleep 86400; done"];

/// A created and started container, and the manager that made it.
///
/// The VM dies with this process whatever happens here, so dropping one is
/// about the store: its directory would otherwise be left behind per run.
pub struct Container {
  manager: cfw::containerization::ContainerManager,
  container: cfw::containerization::LinuxContainer,
}

impl Container {
  /// Creates and starts [`store::IMAGE`] under a name unique across the suite —
  /// it names the container and its directory in the shared store — held open
  /// by a process that outlives every `exec`.
  pub fn boot(name: &str) -> Self {
    Self::boot_with(name, |_| {})
  }

  /// The same, with the configuration changed by `configure` after the
  /// suite's own changes.
  pub fn boot_with(
    name: &str,
    configure: impl FnOnce(&mut cfw::containerization::linux_container::Configuration) + Send + 'static,
  ) -> Self {
    Self::try_boot_with(name, configure).unwrap_or_else(|error| panic!("{name} should boot: {error}"))
  }

  /// The same, reporting a container that fails to create or start rather
  /// than panicking.
  pub fn try_boot_with(
    name: &str,
    configure: impl FnOnce(&mut cfw::containerization::linux_container::Configuration) + Send + 'static,
  ) -> Result<Self, cfw::Error> {
    let image_store = store::image_store();
    let image = store::image(&image_store);
    let mut manager = store::manager(&image_store);

    // Left by a run that died before dropping its container.
    let _ = manager.delete(name);

    let interface = network::interface(name);
    let options = cfw::containerization::container_manager::CreateOptions {
      rootfs_size_in_bytes: super::TEST_ROOTFS_SIZE_IN_BYTES,
      networking: false,
      vm: super::vm(),
      ..Default::default()
    };

    let container = manager.create(name, &image, options, move |configuration| {
      configuration.process.arguments = KEEPALIVE.map(String::from).to_vec();
      configuration.cpus = super::TEST_CPUS;
      configuration.memory_in_bytes = super::TEST_MEMORY_IN_BYTES;
      configuration.interfaces = vec![interface];
      configuration.dns = Some(network::gateway_dns());

      configure(configuration);
    })?;

    let booted = Self { manager, container };

    booted.container.create()?;
    booted.container.start()?;

    Ok(booted)
  }

  pub fn container(&self) -> &cfw::containerization::LinuxContainer {
    &self.container
  }

  /// The container's directory, where the manager puts its rootfs and boot log.
  pub fn directory(&self) -> std::path::PathBuf {
    store::root().join("containers").join(self.container.id())
  }

  /// Runs `configuration` to its end, returning its exit code.
  pub fn run(&self, id: &str, configuration: cfw::containerization::LinuxProcessConfiguration) -> i32 {
    let arguments = configuration.arguments.clone();
    let name = self.container.id();
    let process = self
      .container
      .exec(id, configuration)
      .unwrap_or_else(|error| panic!("{arguments:?} should exec in {name}: {error}"));

    process
      .start()
      .unwrap_or_else(|error| panic!("{arguments:?} should start in {name}: {error}"));

    let status = process
      .wait(None)
      .unwrap_or_else(|error| panic!("{arguments:?} should finish in {name}: {error}"));

    let _ = process.delete();

    status.exit_code
  }

  /// Runs a command with nothing attached, reporting its exit code.
  pub fn exec(&self, id: &str, arguments: &[&str]) -> i32 {
    self.run(id, cfw::containerization::LinuxProcessConfiguration::new(arguments))
  }

  /// The same, reporting what it wrote to stdout.
  ///
  /// The guest writes into a pipe of this test's making: Swift streams a
  /// duplicate of the descriptor, so nothing drains the pipe until the process
  /// exits, and what is run must write less than a pipe holds.
  pub fn capture(&self, id: &str, arguments: &[&str]) -> String {
    self.capture_with(id, cfw::containerization::LinuxProcessConfiguration::new(arguments))
  }

  /// The same, for a process configured beyond its arguments.
  pub fn capture_with(&self, id: &str, configuration: cfw::containerization::LinuxProcessConfiguration) -> String {
    let (mut read, write) = std::io::pipe().expect("a pipe should be creatable");
    let arguments = configuration.arguments.clone();

    let code = self.run(
      id,
      cfw::containerization::LinuxProcessConfiguration {
        stdout: Some(write.as_raw_fd()),
        ..configuration
      },
    );

    drop(write);
    assert_eq!(code, 0, "{arguments:?} failed in {}", self.container.id());

    drain(&mut read)
  }

  /// What a `/bin/sh` script wrote to stdout, which it must exit 0 from.
  pub fn sh(&self, id: &str, script: &str) -> String {
    self.capture(id, &["/bin/sh", "-c", script])
  }
}

/// Everything already in the pipe, stopping where a read would block.
///
/// Not read to end-of-file: Swift's duplicate of the write end may outlive the
/// process. The process has exited by now, so what it wrote is all there.
fn drain(read: &mut std::io::PipeReader) -> String {
  // SAFETY: the read end is ours; `F_SETFL` only changes how it reads.
  unsafe { libc::fcntl(read.as_raw_fd(), libc::F_SETFL, libc::O_NONBLOCK) };

  let mut written = Vec::new();
  let mut chunk = [0u8; 8192];

  loop {
    match read.read(&mut chunk) {
      Ok(0) => break,
      Ok(count) => written.extend_from_slice(&chunk[..count]),
      Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
      Err(error) => panic!("the guest's output should be readable: {error}"),
    }
  }

  String::from_utf8_lossy(&written).into_owned()
}

impl Drop for Container {
  fn drop(&mut self) {
    let _ = self.container.stop();
    let _ = self.manager.delete(&self.container.id());
  }
}
