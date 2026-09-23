//! What the integration tests share: a store to boot from, and the guards that
//! keep concurrent tests apart.
//!
//! Provisioning downloads a kernel and building pulls a base, so the store is
//! filled once and kept between runs rather than thrown away per test. What a
//! test owns is its container, whose rootfs clone goes when the test does.
//!
//! The entitlement these need is `bin/dev/test-integration`'s business.

#![allow(dead_code)]

use containerization_framework::{
  BootSpec, BuildPlan, BuildStep, Builder, ExecRequest, Network, Resources, Session, Shell, Stdio, Store, UNATTACHED,
};
use std::fs::File;
use std::io::Read;
use std::os::fd::{FromRawFd, RawFd};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Kept apart from any store a developer's own tools use: these tests write
/// containers into it.
const STORE_IN_HOME: &str = ".cache/containerization-framework-tests";

/// Registry-qualified: nothing expands a short reference.
///
/// Alpine rather than a Debian: every boot clones the unpacked rootfs, so the
/// base's size is paid per container. What these tests assert — exit codes,
/// descriptors, a file a step wrote — needs a shell and busybox, not a distro.
pub const BASE_IMAGE: &str = "docker.io/library/alpine:3";
/// A local tag, so nothing tries to pull it. Named for the base, so a change of
/// base is a different image rather than a stale one a store still holds.
pub const TEST_IMAGE: &str = "containerization-framework/test-base:alpine3";

/// Written by the image's one step, read back from a container booted from it.
pub const MARKER_PATH: &str = "/etc/containerization-framework-test";
pub const MARKER: &str = "built-by-the-integration-suite";

/// Enough to run a shell; `.config/nextest.toml` caps how many run at once.
const TEST_CPUS: u32 = 1;
const TEST_MEMORY_IN_BYTES: u64 = 512 * 1024 * 1024;

/// The rootfs ceiling, against the crate's 8 GiB default: the block is made at
/// this size when the image is unpacked and cloned at it for every container,
/// and nothing here writes more than a marker file.
const TEST_ROOTFS_CAPACITY_IN_BYTES: u64 = 1024 * 1024 * 1024;

/// Nextest runs each test in a process of its own, so a `Mutex` would not do.
const PROVISION_LOCK: &str = ".provision.lock";
const IMAGE_LOCK: &str = ".image.lock";
const UNPACK_LOCK: &str = ".unpack.lock";
/// Long enough for a kernel download and a build, short enough that a lock left
/// by a killed test does not stop the next run.
const LOCK_TIMEOUT: Duration = Duration::from_secs(20 * 60);
const LOCK_POLL: Duration = Duration::from_millis(250);

/// Virtualization.framework's built-in NAT. `.1` is the gateway, `.255` the
/// broadcast.
const GATEWAY: &str = "192.168.64.1";
const PREFIX: u32 = 24;
const FIRST_HOST: u32 = 2;
const LAST_HOST: u32 = 250;

pub fn home() -> PathBuf {
  PathBuf::from(std::env::var("HOME").expect("a test runs with a home"))
}

/// The suite's store, named but not touched.
pub fn store() -> Store {
  Store::at(home().join(STORE_IN_HOME))
}

/// The same store with a kernel and the init image in it.
///
/// Cheap once there is nothing to do, so every test that needs a store calls
/// it. The lock is for a first run, where two would download into the same
/// directory at once.
pub fn provisioned() -> Store {
  let store = store();
  std::fs::create_dir_all(store.root()).expect("the store directory should be creatable");

  if store.ready().is_ok() {
    return store;
  }

  let _lock = Lock::take(store.root().join(PROVISION_LOCK));

  if store.ready().is_ok() {
    return store;
  }

  Builder::new(store.clone())
    .provision()
    .expect("the store should provision");

  store
}

/// The same store with [`TEST_IMAGE`] in it, built if it is not there already.
pub fn image() -> Store {
  let store = provisioned();

  if store.holds(TEST_IMAGE) {
    return store;
  }

  let _lock = Lock::take(store.root().join(IMAGE_LOCK));

  if store.holds(TEST_IMAGE) {
    return store;
  }

  build_image(&store);

  store
}

/// [`BASE_IMAGE`] plus one step leaving something a container can read back.
fn build_image(store: &Store) {
  let mut plan = BuildPlan::new(
    "cfw-test-builder",
    BASE_IMAGE,
    TEST_IMAGE,
    resources(),
    network("cfw-test-builder"),
    "base-0",
  );

  // Alpine carries no bash, which the default shell is.
  plan.shell = Shell(["/bin/sh", "-ec"].map(String::from).to_vec());
  plan.rootfs_capacity_in_bytes = TEST_ROOTFS_CAPACITY_IN_BYTES;
  plan.steps = vec![BuildStep {
    name: "marker".to_string(),
    script: format!("echo {MARKER} > {MARKER_PATH}"),
    user: None,
    cache_key: "marker-0".to_string(),
  }];

  Builder::new(store.clone())
    .build(&plan)
    .expect("the test image should build");
}

pub fn resources() -> Resources {
  Resources {
    cpus: TEST_CPUS,
    memory_in_bytes: TEST_MEMORY_IN_BYTES,
  }
}

/// Where a container by this name sits on the NAT network.
///
/// Nothing hands out leases, so the caller allocates. Hashed from the name:
/// stable per container, distinct between concurrent ones.
pub fn network(name: &str) -> Network {
  // FNV-1a: short and well spread.
  let mut hash: u64 = 0xcbf2_9ce4_8422_2325;

  for byte in name.as_bytes() {
    hash ^= u64::from(*byte);
    hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
  }

  let host = FIRST_HOST + (hash % u64::from(LAST_HOST - FIRST_HOST + 1)) as u32;
  let (subnet, _) = GATEWAY
    .rsplit_once('.')
    .expect("the gateway is a dotted quad");

  Network {
    ipv4_address: format!("{subnet}.{host}/{PREFIX}"),
    ipv4_gateway: GATEWAY.to_string(),
  }
}

/// A booted container, and the session that owns it.
///
/// The VM dies with this process whatever happens here, so dropping one is
/// about the store: its rootfs clone would otherwise be left behind per run.
pub struct Container {
  session: Session,
  name: String,
}

impl Container {
  /// Boots [`TEST_IMAGE`] under a name unique across the suite — it names the
  /// container and its directory in the shared store — held open by a process
  /// that outlives every `exec`.
  ///
  /// A first boot unpacks the image, which is the slow part of a cold run, and
  /// concurrent tests would each do it again. So the unpack alone is
  /// serialized: a boot that finds it done waits for nobody.
  pub fn boot(name: &str) -> Self {
    let session = Session::new(image());
    let mut unpacking = None;

    if !is_unpacked(&session) {
      let lock = Lock::take(session.store().root().join(UNPACK_LOCK));

      // Whoever held it may have just unpacked the image.
      if !is_unpacked(&session) {
        unpacking = Some(lock);
      }
    }

    let mut spec = BootSpec::new(name, TEST_IMAGE, resources(), network(name));

    spec.rootfs_capacity_in_bytes = TEST_ROOTFS_CAPACITY_IN_BYTES;

    // The first process takes the container with it when it exits, and the
    // base's own `Cmd` would exit at once.
    spec.arguments = ["/bin/sh", "-c", "while :; do sleep 86400; done"]
      .map(String::from)
      .to_vec();

    session
      .boot(&spec)
      .unwrap_or_else(|error| panic!("{name} should boot: {error}"));

    // Dropped here, not at the end of the test: the next boot waits on the
    // unpack, not on this container.
    drop(unpacking);

    Self {
      session,
      name: name.to_string(),
    }
  }

  pub fn session(&self) -> &Session {
    &self.session
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  /// Runs a command with nothing attached, reporting its exit code.
  pub fn exec(&self, id: &str, arguments: &[&str]) -> i32 {
    self
      .session
      .exec(&ExecRequest::new(
        &self.name,
        id,
        arguments
          .iter()
          .map(|argument| argument.to_string())
          .collect(),
        Stdio::nothing(),
      ))
      .unwrap_or_else(|error| panic!("{arguments:?} should run in {}: {error}", self.name))
  }

  /// The same, reporting what it wrote to stdout.
  ///
  /// The guest writes into a pipe of this test's making, since this crate hands
  /// descriptors over rather than relaying bytes. Nothing drains it until the
  /// process exits, so what is run must write less than a pipe holds.
  pub fn capture(&self, id: &str, arguments: &[&str]) -> String {
    let pipe = Pipe::new();
    let stdio = Stdio {
      terminal: UNATTACHED,
      stdin: UNATTACHED,
      stdout: pipe.write,
      stderr: UNATTACHED,
    };

    let code = self
      .session
      .exec(&ExecRequest::new(
        &self.name,
        id,
        arguments
          .iter()
          .map(|argument| argument.to_string())
          .collect(),
        stdio,
      ))
      .unwrap_or_else(|error| panic!("{arguments:?} should run in {}: {error}", self.name));

    assert_eq!(code, 0, "{arguments:?} failed in {}", self.name);

    pipe.drain()
  }
}

impl Drop for Container {
  fn drop(&mut self) {
    let _ = std::fs::remove_dir_all(self.session.store().container_dir(&self.name));
  }
}

/// Whether a boot would clone a rootfs rather than unpack one. A store that
/// cannot say counts as not unpacked; the boot after it reports what is wrong.
fn is_unpacked(session: &Session) -> bool {
  session.is_unpacked(TEST_IMAGE).unwrap_or(false)
}

/// A pipe whose write end a guest process is given.
///
/// Never read to end-of-file: something still holds the write end open when
/// `exec` returns, so that blocks forever. It has returned by then, so a
/// non-blocking drain gets everything the guest wrote.
///
/// The write end is never closed here either: a second close would take
/// whatever descriptor number was reused since.
struct Pipe {
  read: RawFd,
  write: RawFd,
}

impl Pipe {
  fn new() -> Self {
    let mut ends = [0; 2];

    // SAFETY: `pipe` writes two descriptors into the array it is given.
    let made = unsafe { libc::pipe(ends.as_mut_ptr()) };
    assert_eq!(
      made,
      0,
      "a pipe should be creatable: {}",
      std::io::Error::last_os_error()
    );

    Self {
      read: ends[0],
      write: ends[1],
    }
  }

  /// Everything already in the pipe, stopping where a read would block.
  fn drain(self) -> String {
    // SAFETY: the read end is ours; `F_SETFL` only changes how it reads.
    unsafe { libc::fcntl(self.read, libc::F_SETFL, libc::O_NONBLOCK) };

    // SAFETY: the read end is ours, and this is the one owner of it.
    let mut reader = unsafe { File::from_raw_fd(self.read) };
    let mut written = Vec::new();
    let mut chunk = [0u8; 8192];

    loop {
      match reader.read(&mut chunk) {
        Ok(0) => break,
        Ok(read) => written.extend_from_slice(&chunk[..read]),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
        Err(error) => panic!("the guest's output should be readable: {error}"),
      }
    }

    String::from_utf8_lossy(&written).into_owned()
  }
}

/// A lock held by a file's existence, released by dropping it. One left by a
/// test that died is taken over after [`LOCK_TIMEOUT`], so a crash costs one
/// slow run rather than every run after it.
pub struct Lock(PathBuf);

impl Lock {
  pub fn take(path: PathBuf) -> Self {
    loop {
      match std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
      {
        Ok(_) => return Self(path),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
          if held_too_long(&path) {
            let _ = std::fs::remove_file(&path);
          }
          std::thread::sleep(LOCK_POLL);
        }
        Err(error) => panic!("could not take {}: {error}", path.display()),
      }
    }
  }
}

impl Drop for Lock {
  fn drop(&mut self) {
    let _ = std::fs::remove_file(&self.0);
  }
}

/// Whether whoever made a lock file is gone. A missing one has just been
/// released, which is not a timeout.
fn held_too_long(path: &Path) -> bool {
  std::fs::metadata(path)
    .and_then(|metadata| metadata.modified())
    .is_ok_and(|taken| taken.elapsed().is_ok_and(|held| held > LOCK_TIMEOUT))
}
