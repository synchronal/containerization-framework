//! A container made from the suite's image, and processes run in it.

use super::network;
use super::store;
use containerization_framework as cfw;
use std::io::Read;
use std::os::fd::AsRawFd;

/// Holds a container open; the image's own `Cmd` would exit at once.
pub const KEEPALIVE: [&str; 3] = ["/bin/sh", "-c", "while :; do sleep 86400; done"];

/// A created and started container, and the manager that made it.
///
/// The VM dies with this process whatever happens here, so dropping one is
/// about the store: its directory would otherwise be left behind per run.
pub struct Container {
  manager: cfw::containerization::container::ContainerManager,
  container: cfw::containerization::container::LinuxContainer,
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
    configure: impl FnOnce(&mut cfw::containerization::container::linux_container::Configuration) + Send + 'static,
  ) -> Self {
    Self::try_boot_with(name, configure).unwrap_or_else(|error| panic!("{name} should boot: {error}"))
  }

  /// The same, reporting a container that fails to create or start rather
  /// than panicking.
  pub fn try_boot_with(
    name: &str,
    configure: impl FnOnce(&mut cfw::containerization::container::linux_container::Configuration) + Send + 'static,
  ) -> Result<Self, cfw::Error> {
    let (image, mut manager) = prepare(name, store::manager);
    let container = manager.create(
      name,
      &image,
      create_options(false),
      suite_configuration(name, configure),
    )?;

    Self::start(manager, container)
  }

  /// Creates and starts [`store::IMAGE`] on `network`, which gives the
  /// container its interface and resolver in place of the suite's.
  pub fn boot_on(name: &str, network: cfw::containerization::network::VmnetNetwork) -> Self {
    let (image, mut manager) = prepare(name, |image_store| {
      store::manager_with(
        image_store,
        cfw::containerization::container::container_manager::ContainerManagerOptions {
          network: Some(network),
          ..Default::default()
        },
      )
    });

    manager
      .create(name, &image, create_options(true), keep_alive)
      .and_then(|container| Self::start(manager, container))
      .unwrap_or_else(|error| panic!("{name} should boot: {error}"))
  }

  /// Creates and starts [`store::IMAGE`] from `rootfs`, a block the test
  /// unpacked, rather than one the manager unpacks.
  ///
  /// Its boot log goes beside the rootfs: the manager seeds one in a container
  /// directory that only the image `create` makes.
  pub fn boot_from_rootfs(name: &str, rootfs: cfw::containerization::container::Mount) -> Self {
    let boot_log = std::path::Path::new(&rootfs.source).with_extension("log");
    let (image, mut manager) = prepare(name, store::manager);
    let options = cfw::containerization::container::container_manager::CreateWithRootfsOptions {
      networking: false,
      vm: super::vm(),
      ..Default::default()
    };

    manager
      .create_with_rootfs(
        name,
        &image,
        rootfs,
        options,
        suite_configuration(name, move |configuration| {
          configuration.boot_log = Some(cfw::containerization::vm::BootLog::file(boot_log));
        }),
      )
      .and_then(|container| Self::start(manager, container))
      .unwrap_or_else(|error| panic!("{name} should boot: {error}"))
  }

  fn start(
    manager: cfw::containerization::container::ContainerManager,
    container: cfw::containerization::container::LinuxContainer,
  ) -> Result<Self, cfw::Error> {
    let booted = Self { manager, container };

    booted.container.create()?;
    booted.container.start()?;

    Ok(booted)
  }

  pub fn container(&self) -> &cfw::containerization::container::LinuxContainer {
    &self.container
  }

  /// The container's directory, where the manager puts its rootfs and boot log.
  pub fn directory(&self) -> std::path::PathBuf {
    store::root().join("containers").join(self.container.id())
  }

  /// Runs `configuration` to its end, returning its exit code.
  pub fn run(&self, id: &str, configuration: cfw::containerization::process::LinuxProcessConfiguration) -> i32 {
    let arguments = configuration.arguments.clone();
    let name = self.container.id();
    let process = self
      .container
      .exec(id, configuration)
      .unwrap_or_else(|error| panic!("{arguments:?} should exec in {name}: {error}"));

    finish(&process, &format!("{arguments:?} in {name}"))
  }

  /// Runs a command with nothing attached, reporting its exit code.
  pub fn exec(&self, id: &str, arguments: &[&str]) -> i32 {
    self.run(
      id,
      cfw::containerization::process::LinuxProcessConfiguration::new(arguments),
    )
  }

  /// The same, reporting what it wrote to stdout.
  ///
  /// The guest writes into a pipe of this test's making: Swift streams a
  /// duplicate of the descriptor, so nothing drains the pipe until the process
  /// exits, and what is run must write less than a pipe holds.
  pub fn capture(&self, id: &str, arguments: &[&str]) -> String {
    self.capture_with(
      id,
      cfw::containerization::process::LinuxProcessConfiguration::new(arguments),
    )
  }

  /// The same, for a process configured beyond its arguments.
  pub fn capture_with(
    &self,
    id: &str,
    configuration: cfw::containerization::process::LinuxProcessConfiguration,
  ) -> String {
    let arguments = configuration.arguments.clone();
    let name = self.container.id();

    let (code, output) = captured(&format!("{arguments:?} in {name}"), |stdout| {
      self.container.exec(
        id,
        cfw::containerization::process::LinuxProcessConfiguration {
          stdout: Some(stdout),
          ..configuration
        },
      )
    });

    assert_eq!(code, 0, "{arguments:?} failed in {name}");
    output
  }

  /// What a `/bin/sh` script wrote to stdout, which it must exit 0 from.
  pub fn sh(&self, id: &str, script: &str) -> String {
    self.capture(id, &["/bin/sh", "-c", script])
  }
}

/// The suite's image, and a manager on its store that `manager` makes. A run
/// that died before dropping its container may have left one named `name`
/// behind, so that one is deleted first.
fn prepare(
  name: &str,
  manager: impl FnOnce(&cfw::containerization::image::ImageStore) -> cfw::containerization::container::ContainerManager,
) -> (
  cfw::containerization::image::Image,
  cfw::containerization::container::ContainerManager,
) {
  let image_store = store::image_store();
  let image = store::image(&image_store);
  let mut manager = manager(&image_store);

  let _ = manager.delete(name);

  (image, manager)
}

/// Options that create a container in the suite's limits, with the manager's
/// network or without it.
fn create_options(networking: bool) -> cfw::containerization::container::container_manager::CreateOptions {
  cfw::containerization::container::container_manager::CreateOptions {
    rootfs_size_in_bytes: super::TEST_ROOTFS_SIZE_IN_BYTES,
    networking,
    vm: super::vm(),
    ..Default::default()
  }
}

/// The suite's changes to a seeded configuration, then `configure`'s.
fn suite_configuration(
  name: &str,
  configure: impl FnOnce(&mut cfw::containerization::container::linux_container::Configuration) + Send + 'static,
) -> impl FnOnce(&mut cfw::containerization::container::linux_container::Configuration) -> Result<(), cfw::Error>
+ Send
+ 'static {
  let interface = network::interface(name);

  move |configuration| {
    keep_alive(configuration)?;
    configuration.interfaces = vec![cfw::containerization::network::Interface::Nat(interface)];
    configuration.dns = Some(network::gateway_dns());

    configure(configuration);
    Ok(())
  }
}

/// Runs the process that `exec` makes with stdout on a new pipe, and returns
/// its exit code and what it wrote. As with [`Container::capture`], it must
/// write less than a pipe holds.
pub fn captured(
  what: &str,
  exec: impl FnOnce(std::os::fd::RawFd) -> Result<cfw::containerization::process::LinuxProcess, cfw::Error>,
) -> (i32, String) {
  let (mut read, write) = std::io::pipe().expect("a pipe should be creatable");
  let process = exec(write.as_raw_fd()).unwrap_or_else(|error| panic!("{what} should exec: {error}"));
  let code = finish(&process, what);

  drop(write);
  (code, read_to_end(&mut read))
}

/// Starts an unstarted process, waits for it to exit, deletes it, and returns
/// its exit code. `what` names it in a failure.
pub fn finish(process: &cfw::containerization::process::LinuxProcess, what: &str) -> i32 {
  process
    .start()
    .unwrap_or_else(|error| panic!("{what} should start: {error}"));

  let status = process
    .wait(None)
    .unwrap_or_else(|error| panic!("{what} should finish: {error}"));

  let _ = process.delete();

  status.exit_code
}

/// A process that holds the container open, in the suite's limits.
pub fn keep_alive(
  configuration: &mut cfw::containerization::container::linux_container::Configuration,
) -> Result<(), cfw::Error> {
  configuration.process.arguments = KEEPALIVE.map(String::from).to_vec();
  configuration.cpus = super::TEST_CPUS;
  configuration.memory_in_bytes = super::TEST_MEMORY_IN_BYTES;
  Ok(())
}

/// The same, for a container in a pod.
pub fn keep_alive_in_pod(
  configuration: &mut cfw::containerization::container::linux_pod::ContainerConfiguration,
) -> Result<(), cfw::Error> {
  configuration.process.arguments = KEEPALIVE.map(String::from).to_vec();
  configuration.cpus = super::TEST_CPUS;
  configuration.memory_in_bytes = super::TEST_MEMORY_IN_BYTES;
  Ok(())
}

/// Everything written to the pipe, up to end of file. That comes once the
/// test has dropped its write end and the process that had it is deleted.
///
/// It panics if end of file doesn't come within ten seconds, because then a
/// duplicate of the write end is still open.
pub fn read_to_end(read: &mut std::io::PipeReader) -> String {
  let mut ready = libc::pollfd {
    fd: read.as_raw_fd(),
    events: libc::POLLIN,
    revents: 0,
  };
  let mut written = Vec::new();
  let mut chunk = [0u8; 8192];

  loop {
    // SAFETY: `ready` is one entry, for the read end, which is ours.
    let polled = unsafe { libc::poll(&mut ready, 1, 10_000) };
    assert!(
      polled > 0,
      "the pipe should reach end of file, but a write end is still open"
    );

    match read.read(&mut chunk).expect("the pipe should be readable") {
      0 => return String::from_utf8_lossy(&written).into_owned(),
      count => written.extend_from_slice(&chunk[..count]),
    }
  }
}

impl Drop for Container {
  fn drop(&mut self) {
    let _ = self.container.stop();
    let _ = self.manager.delete(&self.container.id());
  }
}
