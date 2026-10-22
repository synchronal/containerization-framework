//! `LinuxPod`, its nested `LinuxPod.Configuration`,
//! `LinuxPod.ContainerConfiguration` and `LinuxPod.PodVolume`, the volume's
//! `Source` in [`pod_volume`], and the [`StatisticsOptions`] that
//! `statistics` takes.

use super::ContainerStatistics;
use super::LinuxContainer;
use super::Mount;
use super::StatCategory;
use super::UnixSocketConfiguration;
use super::linux_container;
use crate::containerization::GIB;
use crate::containerization::network;
use crate::containerization::process;
use crate::containerization::strings;
use crate::containerization::vm;
use crate::containerization_os::terminal;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::collections::BTreeMap;
use std::os::fd::OwnedFd;

/// `LinuxPod.statistics(containerIDs:categories:)`'s defaulted arguments.
/// [`Default`] uses Swift's defaults: every container, and
/// [`StatCategory::ALL`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StatisticsOptions<'a> {
  /// The containers to report on. `None` asks for every container's
  /// statistics.
  pub container_ids: Option<&'a [&'a str]>,
  pub categories: StatCategory,
}

impl Default for StatisticsOptions<'_> {
  fn default() -> Self {
    Self {
      container_ids: None,
      categories: StatCategory::ALL,
    }
  }
}

/// `LinuxPod.Configuration`, without `extensions`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Configuration {
  pub interfaces: Vec<network::Interface>,
  pub virtualization: bool,
  pub boot_log: Option<vm::BootLog>,
  pub share_process_namespace: bool,
  pub hostname: Option<String>,
  pub dns: Option<network::DNS>,
  pub hosts: Option<network::Hosts>,
  pub volumes: Vec<PodVolume>,
  pub oci_runtime_path: Option<String>,
  pub seccomp_profile: linux_container::configuration::SeccompProfile,
}

/// `LinuxPod.ContainerConfiguration`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContainerConfiguration {
  pub process: process::LinuxProcessConfiguration,
  pub cpus: u32,
  pub memory_in_bytes: u64,
  pub hostname: Option<String>,
  pub sysctl: BTreeMap<String, String>,
  pub mounts: Vec<Mount>,
  pub masked_paths: Vec<String>,
  pub readonly_paths: Vec<String>,
  pub sockets: Vec<UnixSocketConfiguration>,
  pub dns: Option<network::DNS>,
  pub hosts: Option<network::Hosts>,
  /// `None` uses the pod's profile.
  pub seccomp_profile: Option<linux_container::configuration::SeccompProfile>,
  pub use_init: bool,
  /// The cgroup v2 files to write, as in
  /// [`linux_container::Configuration::unified`].
  pub unified: BTreeMap<String, String>,
}

/// `LinuxPod.ContainerConfiguration()`.
impl Default for ContainerConfiguration {
  fn default() -> Self {
    Self {
      process: process::LinuxProcessConfiguration::default(),
      cpus: 4,
      memory_in_bytes: GIB,
      hostname: None,
      sysctl: BTreeMap::new(),
      mounts: LinuxContainer::default_mounts(),
      masked_paths: LinuxContainer::default_masked_paths(),
      readonly_paths: LinuxContainer::default_readonly_paths(),
      sockets: Vec::new(),
      dns: None,
      hosts: None,
      seccomp_profile: None,
      use_init: false,
      unified: BTreeMap::new(),
    }
  }
}

/// `LinuxPod.PodVolume`, storage attached to the pod that its containers
/// mount by name with [`Mount::shared_mount`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PodVolume {
  pub name: String,
  pub source: pod_volume::Source,
  pub format: String,
}

pub mod pod_volume {
  //! `LinuxPod.PodVolume.Source`.

  use std::path::PathBuf;
  use std::time::Duration;

  /// `LinuxPod.PodVolume.Source`.
  #[derive(Clone, Debug, Eq, PartialEq)]
  pub enum Source {
    /// `.nbd(url:timeout:readOnly:)`, a network block device.
    Nbd {
      url: String,
      timeout: Option<Duration>,
      read_only: bool,
    },
    /// `.diskImage(path:readOnly:)`, a disk image on the host.
    DiskImage { path: PathBuf, read_only: bool },
    /// `.tmpfs(sizeBytes:)`, a volume in the guest's memory.
    Tmpfs { size_bytes: Option<u64> },
  }
}

/// `LinuxPod`, which runs several containers in one VM. Swift marks it
/// experimental.
pub struct LinuxPod {
  pub(crate) handle: ffi::CzLinuxPod,
}

opaque_debug!(LinuxPod);

// Swift's `LinuxPod` is `Sendable`.
unsafe impl Send for LinuxPod {}
unsafe impl Sync for LinuxPod {}

impl LinuxPod {
  /// `LinuxPod(_:vmm:vm:logger:configuration:)`, whose closure changes a
  /// `LinuxPod.Configuration()`. It runs here, before Swift is called, and
  /// Swift's closure takes what it made. Unlike Swift's, it runs even when
  /// Swift then rejects the id as too long. Swift's `vm` defaults to
  /// `VMResources.default`.
  pub fn new(
    id: &str,
    vmm: &vm::VZVirtualMachineManager,
    vm: vm::VMResources,
    configuration: impl FnOnce(&mut Configuration) -> Result<(), Error>,
  ) -> Result<Self, Error> {
    let mut config = Configuration::default();
    configuration(&mut config)?;

    platform::outcome(
      ffi::cz_linux_pod_new(id, vmm.handle.duplicate(), vm.cpus, vm.memory_in_bytes, config),
      format!("make pod {id}"),
    )
    .map(|outcome| Self {
      handle: outcome.linux_pod(),
    })
  }

  /// `LinuxPod.id`.
  pub fn id(&self) -> String {
    self.handle.id()
  }

  /// `LinuxPod.config`.
  pub fn config(&self) -> Configuration {
    let ((), config) = platform::filled(|receive| self.handle.config(Configuration::default(), receive));

    config.expect("Swift hands back the configuration it fills")
  }

  /// `LinuxPod.vm`.
  pub fn vm(&self) -> vm::VMResources {
    vm::VMResources {
      cpus: self.handle.vm_cpus(),
      memory_in_bytes: self.handle.vm_memory_in_bytes(),
    }
  }

  /// `LinuxPod.cpus`, which Swift reads from the VM's resources.
  pub fn cpus(&self) -> u32 {
    self.vm().cpus
  }

  /// `LinuxPod.memoryInBytes`, which Swift reads from the VM's resources.
  pub fn memory_in_bytes(&self) -> u64 {
    self.vm().memory_in_bytes
  }

  /// `LinuxPod.interfaces`, which Swift reads from the configuration.
  pub fn interfaces(&self) -> Vec<network::Interface> {
    self.config().interfaces
  }

  /// `LinuxPod.addContainer(_:rootfs:configuration:)`, whose closure changes a
  /// `LinuxPod.ContainerConfiguration()`. Swift calls it once, on its own
  /// thread. If it returns an error, `add_container` returns that error.
  ///
  /// The closure shouldn't call a method that wraps an `async` Swift method,
  /// for the reason [`super::ContainerManager::create`] gives. It must not
  /// call this pod at all, because Swift holds the pod's lock while the
  /// closure runs, so the call would never return.
  pub fn add_container(
    &self,
    id: &str,
    rootfs: Mount,
    configuration: impl FnOnce(&mut ContainerConfiguration) -> Result<(), Error> + Send + 'static,
  ) -> Result<(), Error> {
    platform::configured(
      configuration,
      |configure| {
        self
          .handle
          .add_container(id, rootfs, ContainerConfiguration::default(), configure)
      },
      format!("add {id} to pod {}", self.id()),
    )
    .map(drop)
  }

  /// `LinuxPod.create()`.
  pub fn create(&self) -> Result<(), Error> {
    platform::outcome(self.handle.create(), format!("create pod {}", self.id())).map(drop)
  }

  /// `LinuxPod.startContainer(_:)`.
  pub fn start_container(&self, container_id: &str) -> Result<(), Error> {
    platform::outcome(
      self.handle.start_container(container_id),
      format!("start {container_id} in pod {}", self.id()),
    )
    .map(drop)
  }

  /// `LinuxPod.stopContainer(_:)`.
  pub fn stop_container(&self, container_id: &str) -> Result<(), Error> {
    platform::outcome(
      self.handle.stop_container(container_id),
      format!("stop {container_id} in pod {}", self.id()),
    )
    .map(drop)
  }

  /// `LinuxPod.stop()`.
  pub fn stop(&self) -> Result<(), Error> {
    platform::outcome(self.handle.stop(), format!("stop pod {}", self.id())).map(drop)
  }

  /// `LinuxPod.killContainer(_:signal:)`.
  pub fn kill_container(&self, container_id: &str, signal: process::Signal) -> Result<(), Error> {
    platform::outcome(
      self.handle.kill_container(container_id, signal.raw_value),
      format!("signal {container_id} in pod {}", self.id()),
    )
    .map(drop)
  }

  /// `LinuxPod.waitContainer(_:timeoutInSeconds:)`. Swift's
  /// `timeoutInSeconds` defaults to `nil`.
  pub fn wait_container(
    &self,
    container_id: &str,
    timeout_in_seconds: Option<i64>,
  ) -> Result<process::ExitStatus, Error> {
    platform::outcome(
      self.handle.wait_container(container_id, timeout_in_seconds),
      format!("wait for {container_id} in pod {}", self.id()),
    )
    .and_then(|outcome| process::ExitStatus::from_outcome(&outcome))
  }

  /// `LinuxPod.resizeContainer(_:to:)`.
  pub fn resize_container(&self, container_id: &str, to: terminal::Size) -> Result<(), Error> {
    platform::outcome(
      self
        .handle
        .resize_container(container_id, to.width, to.height),
      format!("resize {container_id} in pod {}", self.id()),
    )
    .map(drop)
  }

  /// `LinuxPod.execInContainer(_:processID:configuration:)`. The closure
  /// changes a copy of the container's process configuration, with the
  /// arguments, terminal and stdio reset. Swift calls it once, on its own
  /// thread. If it returns an error, `exec_in_container` returns that error.
  ///
  /// The closure shouldn't call a method that wraps an `async` Swift method,
  /// for the reason [`super::ContainerManager::create`] gives. It must not
  /// call this pod at all, because Swift holds the pod's lock while the
  /// closure runs, so the call would never return.
  pub fn exec_in_container(
    &self,
    container_id: &str,
    process_id: &str,
    configuration: impl FnOnce(&mut process::LinuxProcessConfiguration) -> Result<(), Error> + Send + 'static,
  ) -> Result<process::LinuxProcess, Error> {
    platform::configured(
      configuration,
      |configure| {
        self.handle.exec_in_container(
          container_id,
          process_id,
          process::LinuxProcessConfiguration::default(),
          configure,
        )
      },
      format!("exec {process_id} in {container_id} in pod {}", self.id()),
    )
    .map(|outcome| process::LinuxProcess {
      handle: outcome.linux_process(),
    })
  }

  /// `LinuxPod.listContainers()`.
  pub fn list_containers(&self) -> Vec<String> {
    self.handle.list_containers()
  }

  /// `LinuxPod.statistics(containerIDs:categories:)`.
  pub fn statistics(&self, options: StatisticsOptions<'_>) -> Result<Vec<ContainerStatistics>, Error> {
    platform::outcome(
      self.handle.statistics(
        options.container_ids.is_some(),
        strings(options.container_ids.unwrap_or_default()),
        options.categories.raw_value,
      ),
      format!("read pod {}'s statistics", self.id()),
    )
    .map(|outcome| outcome.container_statistics_list())
  }

  /// `LinuxPod.dialVsock(port:)`. The caller owns the connection's
  /// descriptor.
  pub fn dial_vsock(&self, port: u32) -> Result<OwnedFd, Error> {
    platform::owned_fd(
      self.handle.dial_vsock(port),
      format!("dial vsock port {port} in pod {}", self.id()),
    )
  }

  /// `LinuxPod.withVirtualMachineInstance(_:)`. Like Swift's, it fails unless
  /// the pod is created, and otherwise runs `body` with the instance on this
  /// thread.
  pub fn with_virtual_machine_instance<T>(
    &self,
    body: impl FnOnce(&vm::VZVirtualMachineInstance) -> Result<T, Error>,
  ) -> Result<T, Error> {
    let instance = platform::outcome(
      self.handle.virtual_machine_instance(),
      format!("reach pod {}'s virtual machine", self.id()),
    )?;

    body(&vm::VZVirtualMachineInstance {
      handle: instance.virtual_machine_instance(),
    })
  }

  /// `LinuxPod.filesystemOperation(_:operation:path:)`.
  pub fn filesystem_operation(
    &self,
    container_id: &str,
    operation: vm::FilesystemOperation,
    path: &str,
  ) -> Result<(), Error> {
    platform::outcome(
      self
        .handle
        .filesystem_operation(container_id, operation.into(), path),
      format!("{operation:?} {path} in {container_id} in pod {}", self.id()),
    )
    .map(drop)
  }

  /// `LinuxPod.closeContainerStdin(_:)`.
  pub fn close_container_stdin(&self, container_id: &str) -> Result<(), Error> {
    platform::outcome(
      self.handle.close_container_stdin(container_id),
      format!("close {container_id}'s stdin in pod {}", self.id()),
    )
    .map(drop)
  }

  /// `LinuxPod.relayUnixSocket(_:socket:)`.
  pub fn relay_unix_socket(&self, container_id: &str, socket: UnixSocketConfiguration) -> Result<(), Error> {
    platform::outcome(
      self.handle.relay_unix_socket(container_id, socket.handle),
      format!("relay a socket for {container_id} in pod {}", self.id()),
    )
    .map(drop)
  }
}
