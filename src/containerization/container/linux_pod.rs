//! `LinuxPod`, its nested `LinuxPod.Configuration`,
//! `LinuxPod.ContainerConfiguration` and `LinuxPod.PodVolume`, and the
//! volume's `Source` in [`pod_volume`].

use super::ContainerStatistics;
use super::FilesystemOperation;
use super::LinuxContainer;
use super::Mount;
use super::StatCategory;
use super::UnixSocketConfiguration;
use super::linux_container::SeccompProfile;
use crate::containerization::GIB;
use crate::containerization::network::Dns;
use crate::containerization::network::Hosts;
use crate::containerization::network::Interface;
use crate::containerization::process::ExitStatus;
use crate::containerization::process::LinuxProcess;
use crate::containerization::process::LinuxProcessConfiguration;
use crate::containerization::process::Signal;
use crate::containerization::vm::BootLog;
use crate::containerization::vm::VmResources;
use crate::containerization::vm::VzVirtualMachineInstance;
use crate::containerization::vm::VzVirtualMachineManager;
use crate::containerization_os::terminal;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::collections::BTreeMap;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;

/// `LinuxPod.Configuration`, without `extensions`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Configuration {
  pub interfaces: Vec<Interface>,
  pub virtualization: bool,
  pub boot_log: Option<BootLog>,
  pub share_process_namespace: bool,
  pub hostname: Option<String>,
  pub dns: Option<Dns>,
  pub hosts: Option<Hosts>,
  pub volumes: Vec<PodVolume>,
  pub oci_runtime_path: Option<String>,
  pub seccomp_profile: SeccompProfile,
}

/// `LinuxPod.ContainerConfiguration`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContainerConfiguration {
  pub process: LinuxProcessConfiguration,
  pub cpus: u32,
  pub memory_in_bytes: u64,
  pub hostname: Option<String>,
  pub sysctl: BTreeMap<String, String>,
  pub mounts: Vec<Mount>,
  pub masked_paths: Vec<String>,
  pub readonly_paths: Vec<String>,
  pub sockets: Vec<UnixSocketConfiguration>,
  pub dns: Option<Dns>,
  pub hosts: Option<Hosts>,
  /// The pod's when `None`.
  pub seccomp_profile: Option<SeccompProfile>,
  pub use_init: bool,
}

/// `LinuxPod.ContainerConfiguration()`.
impl Default for ContainerConfiguration {
  fn default() -> Self {
    Self {
      process: LinuxProcessConfiguration::default(),
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
    }
  }
}

/// `LinuxPod.PodVolume`: storage attached to the pod, which its containers
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
    /// `.tmpfs(sizeBytes:)`, in the guest's memory.
    Tmpfs { size_bytes: Option<u64> },
  }
}

/// `LinuxPod`: containers sharing one VM.
pub struct LinuxPod {
  pub(crate) handle: ffi::CzLinuxPod,
}

// Swift's `LinuxPod` is `Sendable`.
unsafe impl Send for LinuxPod {}
unsafe impl Sync for LinuxPod {}

impl LinuxPod {
  /// `LinuxPod(_:vmm:vm:logger:configuration:)`, whose closure changes a
  /// `LinuxPod.Configuration()`. Swift calls it before anything else, so it
  /// runs here, and Swift's closure takes what it made.
  pub fn new(
    id: &str,
    vmm: &VzVirtualMachineManager,
    vm: VmResources,
    configuration: impl FnOnce(&mut Configuration),
  ) -> Result<Self, Error> {
    let mut config = Configuration::default();
    configuration(&mut config);

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
  pub fn vm(&self) -> VmResources {
    VmResources {
      cpus: self.handle.vm_cpus(),
      memory_in_bytes: self.handle.vm_memory_in_bytes(),
    }
  }

  /// `LinuxPod.cpus`: the VM's.
  pub fn cpus(&self) -> u32 {
    self.vm().cpus
  }

  /// `LinuxPod.memoryInBytes`: the VM's.
  pub fn memory_in_bytes(&self) -> u64 {
    self.vm().memory_in_bytes
  }

  /// `LinuxPod.interfaces`: the configuration's.
  pub fn interfaces(&self) -> Vec<Interface> {
    self.config().interfaces
  }

  /// `LinuxPod.addContainer(_:rootfs:configuration:)`, whose closure changes a
  /// `LinuxPod.ContainerConfiguration()`. Swift calls it once, on its own
  /// thread.
  pub fn add_container(
    &self,
    id: &str,
    rootfs: Mount,
    configuration: impl FnOnce(&mut ContainerConfiguration) + Send + 'static,
  ) -> Result<(), Error> {
    platform::outcome(
      self.handle.add_container(
        id,
        rootfs,
        ContainerConfiguration::default(),
        platform::Configure::new(configuration),
      ),
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
  pub fn kill_container(&self, container_id: &str, signal: Signal) -> Result<(), Error> {
    platform::outcome(
      self.handle.kill_container(container_id, signal.raw_value),
      format!("signal {container_id} in pod {}", self.id()),
    )
    .map(drop)
  }

  /// `LinuxPod.waitContainer(_:timeoutInSeconds:)`.
  pub fn wait_container(&self, container_id: &str, timeout_in_seconds: Option<i64>) -> Result<ExitStatus, Error> {
    platform::outcome(
      self.handle.wait_container(container_id, timeout_in_seconds),
      format!("wait for {container_id} in pod {}", self.id()),
    )
    .map(|outcome| platform::exit_status(&outcome))
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
  /// changes the container's process configuration, less its arguments,
  /// terminal and stdio. Swift calls it once, on its own thread.
  pub fn exec_in_container(
    &self,
    container_id: &str,
    process_id: &str,
    configuration: impl FnOnce(&mut LinuxProcessConfiguration) + Send + 'static,
  ) -> Result<LinuxProcess, Error> {
    platform::outcome(
      self.handle.exec_in_container(
        container_id,
        process_id,
        LinuxProcessConfiguration::default(),
        platform::ConfigureProcess::new(configuration),
      ),
      format!("exec {process_id} in {container_id} in pod {}", self.id()),
    )
    .map(|outcome| LinuxProcess {
      handle: outcome.linux_process(),
    })
  }

  /// `LinuxPod.listContainers()`.
  pub fn list_containers(&self) -> Vec<String> {
    self.handle.list_containers()
  }

  /// `LinuxPod.statistics(containerIDs:categories:)`. `None` is every
  /// container's.
  pub fn statistics(
    &self,
    container_ids: Option<&[&str]>,
    categories: StatCategory,
  ) -> Result<Vec<ContainerStatistics>, Error> {
    let ids = container_ids.unwrap_or_default();

    platform::outcome(
      self.handle.statistics(
        container_ids.is_some(),
        ids.iter().map(|id| id.to_string()).collect(),
        categories.raw_value,
      ),
      format!("read pod {}'s statistics", self.id()),
    )
    .map(|outcome| outcome.container_statistics_list())
  }

  /// `LinuxPod.dialVsock(port:)`. Swift hands over the connection's
  /// descriptor.
  pub fn dial_vsock(&self, port: u32) -> Result<OwnedFd, Error> {
    platform::outcome(
      self.handle.dial_vsock(port),
      format!("dial vsock port {port} in pod {}", self.id()),
    )
    // SAFETY: Swift's `FileHandle` doesn't close its descriptor, and
    // forgets it once it crosses.
    .map(|outcome| unsafe { OwnedFd::from_raw_fd(outcome.int32()) })
  }

  /// `LinuxPod.withVirtualMachineInstance(_:)`, as
  /// [`LinuxContainer::with_virtual_machine_instance`] is.
  pub fn with_virtual_machine_instance<T>(
    &self,
    body: impl FnOnce(&VzVirtualMachineInstance) -> Result<T, Error>,
  ) -> Result<T, Error> {
    let instance = platform::outcome(
      self.handle.virtual_machine_instance(),
      format!("reach pod {}'s virtual machine", self.id()),
    )?;

    body(&VzVirtualMachineInstance {
      handle: instance.virtual_machine_instance(),
    })
  }

  /// `LinuxPod.filesystemOperation(_:operation:path:)`.
  pub fn filesystem_operation(
    &self,
    container_id: &str,
    operation: FilesystemOperation,
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
      self.handle.relay_unix_socket(container_id, socket),
      format!("relay a socket for {container_id} in pod {}", self.id()),
    )
    .map(drop)
  }
}
