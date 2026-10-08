//! `LinuxContainer`, and its nested `LinuxContainer.Configuration`.

use super::ContainerStatistics;
use super::FilesystemOperation;
use super::Mount;
use super::StatCategory;
use super::UnixSocketConfiguration;
use crate::containerization::GIB;
use crate::containerization::network;
use crate::containerization::process;
use crate::containerization::strings;
use crate::containerization::vm;
use crate::containerization_oci;
use crate::containerization_os::terminal;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::collections::BTreeMap;
use std::fmt;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::path::Path;

/// `LinuxContainer.Configuration.SeccompProfile`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum SeccompProfile {
  #[default]
  Unconfined,
  Default,
  /// A profile of its own. [`containerization_oci::runtime::LinuxSeccomp::decode`]
  /// reads one from JSON.
  Profile(containerization_oci::runtime::LinuxSeccomp),
}

/// `LinuxContainer.Configuration`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Configuration {
  pub process: process::LinuxProcessConfiguration,
  pub cpus: u32,
  pub memory_in_bytes: u64,
  pub hostname: Option<String>,
  pub sysctl: BTreeMap<String, String>,
  pub interfaces: Vec<network::Interface>,
  pub sockets: Vec<UnixSocketConfiguration>,
  pub mounts: Vec<Mount>,
  pub masked_paths: Vec<String>,
  pub readonly_paths: Vec<String>,
  pub dns: Option<network::DNS>,
  pub hosts: Option<network::Hosts>,
  pub virtualization: bool,
  pub boot_log: Option<vm::BootLog>,
  pub oci_runtime_path: Option<String>,
  pub seccomp_profile: SeccompProfile,
  pub use_init: bool,
}

/// `LinuxContainer.Configuration()`.
impl Default for Configuration {
  fn default() -> Self {
    Self {
      process: process::LinuxProcessConfiguration::default(),
      cpus: 4,
      memory_in_bytes: GIB,
      hostname: None,
      sysctl: BTreeMap::new(),
      interfaces: Vec::new(),
      sockets: Vec::new(),
      mounts: LinuxContainer::default_mounts(),
      masked_paths: LinuxContainer::default_masked_paths(),
      readonly_paths: LinuxContainer::default_readonly_paths(),
      dns: None,
      hosts: None,
      virtualization: false,
      boot_log: None,
      oci_runtime_path: None,
      seccomp_profile: SeccompProfile::Unconfined,
      use_init: false,
    }
  }
}

/// `copyIn(from:to:mode:createParents:chunkSize:)`'s defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CopyInOptions {
  pub mode: u32,
  pub create_parents: bool,
  pub chunk_size: usize,
}

/// Mode `0o644`, creating parents, in chunks of
/// [`LinuxContainer::DEFAULT_COPY_CHUNK_SIZE`].
impl Default for CopyInOptions {
  fn default() -> Self {
    Self {
      mode: 0o644,
      create_parents: true,
      chunk_size: LinuxContainer::DEFAULT_COPY_CHUNK_SIZE,
    }
  }
}

/// `copyOut(from:to:createParents:chunkSize:)`'s defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CopyOutOptions {
  pub create_parents: bool,
  pub chunk_size: usize,
}

/// Creating parents, in chunks of [`LinuxContainer::DEFAULT_COPY_CHUNK_SIZE`].
impl Default for CopyOutOptions {
  fn default() -> Self {
    Self {
      create_parents: true,
      chunk_size: LinuxContainer::DEFAULT_COPY_CHUNK_SIZE,
    }
  }
}

/// `LinuxContainer`. Made by [`super::ContainerManager::create`], or by
/// [`Self::new`] on a VM manager of the caller's.
pub struct LinuxContainer {
  pub(crate) handle: ffi::CzLinuxContainer,
}

impl fmt::Debug for LinuxContainer {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("LinuxContainer")
      .finish_non_exhaustive()
  }
}

// Swift's `LinuxContainer` is `Sendable`.
unsafe impl Send for LinuxContainer {}
unsafe impl Sync for LinuxContainer {}

impl LinuxContainer {
  /// `LinuxContainer.maxIDLength`.
  pub const MAX_ID_LENGTH: usize = 64;

  /// `LinuxContainer.defaultCopyChunkSize`: 1 MiB.
  pub const DEFAULT_COPY_CHUNK_SIZE: usize = 1024 * 1024;

  /// `LinuxContainer(_:rootfs:writableLayer:vmm:vm:configuration:logger:)`.
  pub fn new(
    id: &str,
    rootfs: Mount,
    writable_layer: Option<Mount>,
    vmm: &vm::VZVirtualMachineManager,
    vm: vm::VMResources,
    configuration: Configuration,
  ) -> Result<Self, Error> {
    let (has_writable_layer, writable_layer) = writable_layer_crossing(writable_layer);

    platform::outcome(
      ffi::cz_linux_container_new(
        id,
        rootfs,
        has_writable_layer,
        writable_layer,
        vmm.handle.duplicate(),
        vm.cpus,
        vm.memory_in_bytes,
        configuration,
      ),
      format!("make {id}"),
    )
    .map(|outcome| Self {
      handle: outcome.linux_container(),
    })
  }

  /// `LinuxContainer(_:rootfs:writableLayer:vmm:vm:logger:configuration:)`,
  /// whose closure changes a `LinuxContainer.Configuration()` before
  /// [`Self::new`] takes it, as Swift's convenience init does.
  pub fn new_with(
    id: &str,
    rootfs: Mount,
    writable_layer: Option<Mount>,
    vmm: &vm::VZVirtualMachineManager,
    vm: vm::VMResources,
    configuration: impl FnOnce(&mut Configuration) -> Result<(), Error>,
  ) -> Result<Self, Error> {
    let mut config = Configuration::default();
    configuration(&mut config)?;

    Self::new(id, rootfs, writable_layer, vmm, vm, config)
  }

  /// `LinuxContainer.id`.
  pub fn id(&self) -> String {
    self.handle.id()
  }

  /// `LinuxContainer.rootfs`.
  pub fn rootfs(&self) -> Mount {
    self.handle.rootfs().mount()
  }

  /// `LinuxContainer.writableLayer`.
  pub fn writable_layer(&self) -> Option<Mount> {
    self.handle.writable_layer().optional_mount()
  }

  /// `LinuxContainer.config`. Its process's `stdin`, `stdout` and `stderr`
  /// are `None`, since Swift's streams don't cross back.
  pub fn config(&self) -> Configuration {
    let ((), config) = platform::filled(|receive| self.handle.config(Configuration::default(), receive));

    config.expect("Swift hands back the configuration it fills")
  }

  /// `LinuxContainer.vm`.
  pub fn vm(&self) -> vm::VMResources {
    vm::VMResources {
      cpus: self.handle.vm_cpus(),
      memory_in_bytes: self.handle.vm_memory_in_bytes(),
    }
  }

  /// `LinuxContainer.cpus`: the configuration's.
  pub fn cpus(&self) -> u32 {
    self.config().cpus
  }

  /// `LinuxContainer.memoryInBytes`: the configuration's.
  pub fn memory_in_bytes(&self) -> u64 {
    self.config().memory_in_bytes
  }

  /// `LinuxContainer.interfaces`: the configuration's.
  pub fn interfaces(&self) -> Vec<network::Interface> {
    self.config().interfaces
  }

  /// `LinuxContainer.create()`.
  pub fn create(&self) -> Result<(), Error> {
    platform::outcome(self.handle.create(), format!("create {}", self.id())).map(drop)
  }

  /// `LinuxContainer.start()`.
  pub fn start(&self) -> Result<(), Error> {
    platform::outcome(self.handle.start(), format!("start {}", self.id())).map(drop)
  }

  /// `LinuxContainer.stop()`.
  pub fn stop(&self) -> Result<(), Error> {
    platform::outcome(self.handle.stop(), format!("stop {}", self.id())).map(drop)
  }

  /// `LinuxContainer.kill(_:)`.
  pub fn kill(&self, signal: process::Signal) -> Result<(), Error> {
    platform::outcome(self.handle.kill(signal.raw_value), format!("signal {}", self.id())).map(drop)
  }

  /// `LinuxContainer.wait(timeoutInSeconds:)`.
  pub fn wait(&self, timeout_in_seconds: Option<i64>) -> Result<process::ExitStatus, Error> {
    platform::outcome(self.handle.wait(timeout_in_seconds), format!("wait for {}", self.id()))
      .map(|outcome| platform::exit_status(&outcome))
  }

  /// `LinuxContainer.resize(to:)`.
  pub fn resize(&self, to: terminal::Size) -> Result<(), Error> {
    platform::outcome(self.handle.resize(to.width, to.height), format!("resize {}", self.id())).map(drop)
  }

  /// `LinuxContainer.exec(_:configuration:)`.
  pub fn exec(
    &self,
    id: &str,
    configuration: process::LinuxProcessConfiguration,
  ) -> Result<process::LinuxProcess, Error> {
    platform::outcome(
      self.handle.exec(id, configuration),
      format!("exec {id} in {}", self.id()),
    )
    .map(|outcome| process::LinuxProcess {
      handle: outcome.linux_process(),
    })
  }

  /// `LinuxContainer.exec(_:configuration:)` with a closure, which changes a
  /// `LinuxProcessConfiguration()`. Swift calls it once, on its own thread,
  /// and an error it returns is thrown.
  pub fn exec_with(
    &self,
    id: &str,
    configuration: impl FnOnce(&mut process::LinuxProcessConfiguration) -> Result<(), Error> + Send + 'static,
  ) -> Result<process::LinuxProcess, Error> {
    platform::configured(
      configuration,
      |configure| {
        self
          .handle
          .exec_with(id, process::LinuxProcessConfiguration::default(), configure)
      },
      format!("exec {id} in {}", self.id()),
    )
    .map(|outcome| process::LinuxProcess {
      handle: outcome.linux_process(),
    })
  }

  /// `LinuxContainer.dialVsock(port:)`. Swift hands over the connection's
  /// descriptor.
  pub fn dial_vsock(&self, port: u32) -> Result<OwnedFd, Error> {
    platform::outcome(
      self.handle.dial_vsock(port),
      format!("dial vsock port {port} in {}", self.id()),
    )
    // SAFETY: Swift's `FileHandle` doesn't close its descriptor, and
    // forgets it once it crosses.
    .map(|outcome| unsafe { OwnedFd::from_raw_fd(outcome.int32()) })
  }

  /// `LinuxContainer.withVirtualMachineInstance(_:)`. Swift runs its closure
  /// on the instance it hands out, once it has checked the container is
  /// created; `body` runs the same way, on this thread.
  pub fn with_virtual_machine_instance<T>(
    &self,
    body: impl FnOnce(&vm::VZVirtualMachineInstance) -> Result<T, Error>,
  ) -> Result<T, Error> {
    let instance = platform::outcome(
      self.handle.virtual_machine_instance(),
      format!("reach {}'s virtual machine", self.id()),
    )?;

    body(&vm::VZVirtualMachineInstance {
      handle: instance.virtual_machine_instance(),
    })
  }

  /// `LinuxContainer.closeStdin()`.
  pub fn close_stdin(&self) -> Result<(), Error> {
    platform::outcome(self.handle.close_stdin(), format!("close {}'s stdin", self.id())).map(drop)
  }

  /// `LinuxContainer.statistics(categories:)`.
  pub fn statistics(&self, categories: StatCategory) -> Result<ContainerStatistics, Error> {
    platform::outcome(
      self.handle.statistics(categories.raw_value),
      format!("read {}'s statistics", self.id()),
    )
    .map(|outcome| outcome.container_statistics())
  }

  /// `LinuxContainer.filesystemOperation(operation:path:)`.
  pub fn filesystem_operation(&self, operation: FilesystemOperation, path: &str) -> Result<(), Error> {
    platform::outcome(
      self.handle.filesystem_operation(operation.into(), path),
      format!("{operation:?} {path} in {}", self.id()),
    )
    .map(drop)
  }

  /// `LinuxContainer.copyIn(from:to:mode:createParents:chunkSize:)`, from a
  /// host path to a guest path.
  pub fn copy_in(&self, source: &Path, destination: &Path, options: CopyInOptions) -> Result<(), Error> {
    platform::outcome(
      self.handle.copy_in(
        &source.display().to_string(),
        &destination.display().to_string(),
        options.mode,
        options.create_parents,
        options.chunk_size,
      ),
      format!("copy {} into {}", source.display(), self.id()),
    )
    .map(drop)
  }

  /// `LinuxContainer.copyOut(from:to:createParents:chunkSize:)`, from a guest
  /// path to a host path.
  pub fn copy_out(&self, source: &Path, destination: &Path, options: CopyOutOptions) -> Result<(), Error> {
    platform::outcome(
      self.handle.copy_out(
        &source.display().to_string(),
        &destination.display().to_string(),
        options.create_parents,
        options.chunk_size,
      ),
      format!("copy {} out of {}", source.display(), self.id()),
    )
    .map(drop)
  }

  /// `LinuxContainer.defaultMounts()`.
  pub fn default_mounts() -> Vec<Mount> {
    let defaults = ["nosuid", "noexec", "nodev"];

    vec![
      Mount::any("proc", "proc", "/proc", &[], &[]),
      Mount::any("sysfs", "sysfs", "/sys", &defaults, &[]),
      Mount::any("devtmpfs", "none", "/dev", &["nosuid", "mode=755"], &[]),
      Mount::any("mqueue", "mqueue", "/dev/mqueue", &defaults, &[]),
      Mount::any(
        "tmpfs",
        "tmpfs",
        "/dev/shm",
        &["nosuid", "noexec", "nodev", "mode=1777", "size=65536k"],
        &[],
      ),
      Mount::any("cgroup2", "none", "/sys/fs/cgroup", &defaults, &[]),
      Mount::any(
        "devpts",
        "devpts",
        "/dev/pts",
        &["nosuid", "noexec", "newinstance", "gid=5", "mode=0620", "ptmxmode=0666"],
        &[],
      ),
    ]
  }

  /// `LinuxContainer.defaultMaskedPaths()`.
  pub fn default_masked_paths() -> Vec<String> {
    strings(&[
      "/proc/asound",
      "/proc/acpi",
      "/proc/kcore",
      "/proc/keys",
      "/proc/latency_stats",
      "/proc/timer_list",
      "/proc/timer_stats",
      "/proc/sched_debug",
      "/proc/scsi",
      "/sys/firmware",
      "/sys/devices/virtual/powercap",
    ])
  }

  /// `LinuxContainer.defaultReadonlyPaths()`.
  pub fn default_readonly_paths() -> Vec<String> {
    strings(&["/proc/bus", "/proc/fs", "/proc/irq", "/proc/sys", "/proc/sysrq-trigger"])
  }

  /// `LinuxContainer.defaultOCIMounts()`: the mounts OCI runtimes expect, with
  /// `/dev` as a tmpfs.
  pub fn default_oci_mounts() -> Vec<Mount> {
    let defaults = ["nosuid", "noexec", "nodev"];

    vec![
      Mount::any("proc", "proc", "/proc", &[], &[]),
      Mount::any("tmpfs", "tmpfs", "/dev", &["nosuid", "mode=755", "size=65536k"], &[]),
      Mount::any(
        "devpts",
        "devpts",
        "/dev/pts",
        &["nosuid", "noexec", "newinstance", "gid=5", "mode=0620", "ptmxmode=0666"],
        &[],
      ),
      Mount::any("sysfs", "sysfs", "/sys", &defaults, &[]),
      Mount::any("mqueue", "mqueue", "/dev/mqueue", &defaults, &[]),
      Mount::any(
        "tmpfs",
        "tmpfs",
        "/dev/shm",
        &["nosuid", "noexec", "nodev", "mode=1777", "size=65536k"],
        &[],
      ),
      Mount::any("cgroup2", "none", "/sys/fs/cgroup", &defaults, &[]),
    ]
  }
}

/// A `Mount?`, as it crosses to Swift: the mount stands for `nil` when the
/// flag is false.
fn writable_layer_crossing(writable_layer: Option<Mount>) -> (bool, Mount) {
  match writable_layer {
    Some(writable_layer) => (true, writable_layer),
    None => (false, Mount::any("", "", "", &[], &[])),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn defaults_a_configuration_as_containerization_does() {
    let configuration = Configuration::default();

    assert_eq!(configuration.cpus, 4);
    assert_eq!(configuration.memory_in_bytes, GIB);
    assert_eq!(configuration.mounts.len(), 7);
    assert!(
      configuration
        .masked_paths
        .contains(&"/proc/kcore".to_string())
    );
    assert!(
      configuration
        .readonly_paths
        .contains(&"/proc/sys".to_string())
    );
    assert_eq!(configuration.dns, None);
    assert_eq!(configuration.seccomp_profile, SeccompProfile::Unconfined);
    assert_eq!(configuration.process.working_directory, "/");
    assert_eq!(
      configuration.process.environment_variables,
      [format!("PATH={}", process::LinuxProcessConfiguration::DEFAULT_PATH)]
    );
  }
}
