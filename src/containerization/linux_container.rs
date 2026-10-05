//! `LinuxContainer`, and its nested `LinuxContainer.Configuration`.

use super::BootLog;
use super::Dns;
use super::ExitStatus;
use super::GIB;
use super::Hosts;
use super::LinuxProcess;
use super::LinuxProcessConfiguration;
use super::Mount;
use super::NatInterface;
use super::Signal;
use super::UnixSocketConfiguration;
use super::strings;
use crate::containerization_os::terminal;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::collections::BTreeMap;

/// `LinuxContainer.Configuration.SeccompProfile`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum SeccompProfile {
  #[default]
  Unconfined,
  Default,
  /// A `LinuxSeccomp`, as the JSON of an OCI runtime spec's `linux.seccomp`.
  Profile(String),
}

/// `LinuxContainer.Configuration`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Configuration {
  pub process: LinuxProcessConfiguration,
  pub cpus: u32,
  pub memory_in_bytes: u64,
  pub hostname: Option<String>,
  pub sysctl: BTreeMap<String, String>,
  /// Swift's `[any Interface]`, of which the bridge carries `NATInterface`.
  pub interfaces: Vec<NatInterface>,
  pub sockets: Vec<UnixSocketConfiguration>,
  pub mounts: Vec<Mount>,
  pub masked_paths: Vec<String>,
  pub readonly_paths: Vec<String>,
  pub dns: Option<Dns>,
  pub hosts: Option<Hosts>,
  pub virtualization: bool,
  pub boot_log: Option<BootLog>,
  pub oci_runtime_path: Option<String>,
  pub seccomp_profile: SeccompProfile,
  pub use_init: bool,
}

/// `LinuxContainer.Configuration()`.
impl Default for Configuration {
  fn default() -> Self {
    Self {
      process: LinuxProcessConfiguration::default(),
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

/// `LinuxContainer`. Made by [`super::ContainerManager::create`].
pub struct LinuxContainer {
  pub(crate) handle: ffi::CzLinuxContainer,
}

// Swift's `LinuxContainer` is `Sendable`.
unsafe impl Send for LinuxContainer {}
unsafe impl Sync for LinuxContainer {}

impl LinuxContainer {
  /// `LinuxContainer.maxIDLength`.
  pub const MAX_ID_LENGTH: usize = 64;

  /// `LinuxContainer.id`.
  pub fn id(&self) -> String {
    self.handle.id()
  }

  /// `LinuxContainer.create()`.
  pub fn create(&self) -> Result<(), Error> {
    platform::outcome(self.handle.create(), format!("create {}", self.id())).map(|_| ())
  }

  /// `LinuxContainer.start()`.
  pub fn start(&self) -> Result<(), Error> {
    platform::outcome(self.handle.start(), format!("start {}", self.id())).map(|_| ())
  }

  /// `LinuxContainer.stop()`.
  pub fn stop(&self) -> Result<(), Error> {
    platform::outcome(self.handle.stop(), format!("stop {}", self.id())).map(|_| ())
  }

  /// `LinuxContainer.kill(_:)`.
  pub fn kill(&self, signal: Signal) -> Result<(), Error> {
    platform::outcome(self.handle.kill(signal.raw_value), format!("signal {}", self.id())).map(|_| ())
  }

  /// `LinuxContainer.wait(timeoutInSeconds:)`.
  pub fn wait(&self, timeout_in_seconds: Option<i64>) -> Result<ExitStatus, Error> {
    platform::outcome(self.handle.wait(timeout_in_seconds), format!("wait for {}", self.id()))
      .map(|outcome| platform::exit_status(&outcome))
  }

  /// `LinuxContainer.resize(to:)`.
  pub fn resize(&self, to: terminal::Size) -> Result<(), Error> {
    platform::outcome(self.handle.resize(to.width, to.height), format!("resize {}", self.id())).map(|_| ())
  }

  /// `LinuxContainer.exec(_:configuration:)`.
  pub fn exec(&self, id: &str, configuration: LinuxProcessConfiguration) -> Result<LinuxProcess, Error> {
    platform::outcome(
      self.handle.exec(id, configuration),
      format!("exec {id} in {}", self.id()),
    )
    .map(|outcome| LinuxProcess {
      handle: outcome.linux_process(),
    })
  }

  /// `LinuxContainer.closeStdin()`.
  pub fn close_stdin(&self) -> Result<(), Error> {
    platform::outcome(self.handle.close_stdin(), format!("close {}'s stdin", self.id())).map(|_| ())
  }

  /// `LinuxContainer.defaultMounts()`.
  pub fn default_mounts() -> Vec<Mount> {
    let defaults = ["nosuid", "noexec", "nodev"];

    vec![
      Mount::any("proc", "proc", "/proc", &[]),
      Mount::any("sysfs", "sysfs", "/sys", &defaults),
      Mount::any("devtmpfs", "none", "/dev", &["nosuid", "mode=755"]),
      Mount::any("mqueue", "mqueue", "/dev/mqueue", &defaults),
      Mount::any(
        "tmpfs",
        "tmpfs",
        "/dev/shm",
        &["nosuid", "noexec", "nodev", "mode=1777", "size=65536k"],
      ),
      Mount::any("cgroup2", "none", "/sys/fs/cgroup", &defaults),
      Mount::any(
        "devpts",
        "devpts",
        "/dev/pts",
        &["nosuid", "noexec", "newinstance", "gid=5", "mode=0620", "ptmxmode=0666"],
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
      [format!("PATH={}", LinuxProcessConfiguration::DEFAULT_PATH)]
    );
  }
}
