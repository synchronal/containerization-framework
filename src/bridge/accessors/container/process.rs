//! Getters and setters for a process's configuration: its user, rlimits and
//! capabilities.

use crate::bridge::ffi;
use crate::containerization::process;
use crate::containerization::process::linux_rlimit;
use crate::containerization_oci::runtime;
use crate::containerization_os;

impl runtime::User {
  getters! {
    copy uid: u32;
    copy gid: u32;
    copy umask: Option<u32>;
  }

  pub(crate) fn additional_gids_len(&self) -> usize {
    self.additional_gids.len()
  }

  pub(crate) fn additional_gids_at(&self, index: usize) -> u32 {
    self.additional_gids[index]
  }

  getters! {
    str username;
  }
}

impl process::LinuxCapabilities {
  pub(crate) fn set_len(&self, set: ffi::CapabilitySet) -> usize {
    self.set(set).len()
  }

  pub(crate) fn set_at(&self, set: ffi::CapabilitySet, index: usize) -> &str {
    self.set(set)[index].description()
  }

  fn set(&self, set: ffi::CapabilitySet) -> &[containerization_os::linux::CapabilityName] {
    match set {
      ffi::CapabilitySet::Bounding => &self.bounding,
      ffi::CapabilitySet::Effective => &self.effective,
      ffi::CapabilitySet::Inheritable => &self.inheritable,
      ffi::CapabilitySet::Permitted => &self.permitted,
      ffi::CapabilitySet::Ambient => &self.ambient,
    }
  }
}

impl From<linux_rlimit::Kind> for ffi::RlimitKind {
  fn from(kind: linux_rlimit::Kind) -> Self {
    match kind {
      linux_rlimit::Kind::AddressSpace => Self::AddressSpace,
      linux_rlimit::Kind::CoreFileSize => Self::CoreFileSize,
      linux_rlimit::Kind::CpuTime => Self::CpuTime,
      linux_rlimit::Kind::DataSize => Self::DataSize,
      linux_rlimit::Kind::FileSize => Self::FileSize,
      linux_rlimit::Kind::Locks => Self::Locks,
      linux_rlimit::Kind::LockedMemory => Self::LockedMemory,
      linux_rlimit::Kind::MessageQueue => Self::MessageQueue,
      linux_rlimit::Kind::Nice => Self::Nice,
      linux_rlimit::Kind::OpenFiles => Self::OpenFiles,
      linux_rlimit::Kind::NumberOfProcesses => Self::NumberOfProcesses,
      linux_rlimit::Kind::ResidentSetSize => Self::ResidentSetSize,
      linux_rlimit::Kind::RealtimePriority => Self::RealtimePriority,
      linux_rlimit::Kind::RealtimeTimeout => Self::RealtimeTimeout,
      linux_rlimit::Kind::SignalsPending => Self::SignalsPending,
      linux_rlimit::Kind::StackSize => Self::StackSize,
    }
  }
}

impl From<ffi::RlimitKind> for linux_rlimit::Kind {
  fn from(kind: ffi::RlimitKind) -> Self {
    match kind {
      ffi::RlimitKind::AddressSpace => Self::AddressSpace,
      ffi::RlimitKind::CoreFileSize => Self::CoreFileSize,
      ffi::RlimitKind::CpuTime => Self::CpuTime,
      ffi::RlimitKind::DataSize => Self::DataSize,
      ffi::RlimitKind::FileSize => Self::FileSize,
      ffi::RlimitKind::Locks => Self::Locks,
      ffi::RlimitKind::LockedMemory => Self::LockedMemory,
      ffi::RlimitKind::MessageQueue => Self::MessageQueue,
      ffi::RlimitKind::Nice => Self::Nice,
      ffi::RlimitKind::OpenFiles => Self::OpenFiles,
      ffi::RlimitKind::NumberOfProcesses => Self::NumberOfProcesses,
      ffi::RlimitKind::ResidentSetSize => Self::ResidentSetSize,
      ffi::RlimitKind::RealtimePriority => Self::RealtimePriority,
      ffi::RlimitKind::RealtimeTimeout => Self::RealtimeTimeout,
      ffi::RlimitKind::SignalsPending => Self::SignalsPending,
      ffi::RlimitKind::StackSize => Self::StackSize,
    }
  }
}

impl process::LinuxProcessConfiguration {
  pub(crate) fn set_arguments(&mut self, arguments: Vec<String>) {
    self.arguments = arguments;
  }

  pub(crate) fn set_environment_variables(&mut self, environment_variables: Vec<String>) {
    self.environment_variables = environment_variables;
  }

  pub(crate) fn set_working_directory(&mut self, working_directory: String) {
    self.working_directory = working_directory;
  }

  pub(crate) fn set_user(
    &mut self,
    uid: u32,
    gid: u32,
    umask: Option<u32>,
    additional_gids: Vec<u32>,
    username: String,
  ) {
    self.user = runtime::User {
      uid,
      gid,
      umask,
      additional_gids,
      username,
    };
  }

  pub(crate) fn set_no_new_privileges(&mut self, no_new_privileges: bool) {
    self.no_new_privileges = no_new_privileges;
  }

  pub(crate) fn set_capabilities(
    &mut self,
    bounding: Vec<String>,
    effective: Vec<String>,
    inheritable: Vec<String>,
    permitted: Vec<String>,
    ambient: Vec<String>,
  ) {
    let names = |set: Vec<String>| {
      set
        .iter()
        .map(|description| containerization_os::linux::CapabilityName::from_description(description))
        .collect()
    };

    self.capabilities = process::LinuxCapabilities {
      bounding: names(bounding),
      effective: names(effective),
      inheritable: names(inheritable),
      permitted: names(permitted),
      ambient: names(ambient),
    };
  }

  pub(crate) fn set_terminal(&mut self, terminal: bool) {
    self.terminal = terminal;
  }

  pub(crate) fn clear_rlimits(&mut self) {
    self.rlimits.clear();
  }

  pub(crate) fn push_rlimit(&mut self, kind: ffi::RlimitKind, hard: u64, soft: u64) {
    self.rlimits.push(process::LinuxRLimit {
      kind: kind.into(),
      hard,
      soft,
    });
  }

  getters! {
    list arguments: str => arguments_len, arguments_at;
    list environment_variables: str => environment_variables_len, environment_variables_at;
    str working_directory;
    borrow user: runtime::User;
  }

  pub(crate) fn rlimits_len(&self) -> usize {
    self.rlimits.len()
  }

  pub(crate) fn rlimit_kind_at(&self, index: usize) -> ffi::RlimitKind {
    self.rlimits[index].kind.into()
  }

  pub(crate) fn rlimit_hard_at(&self, index: usize) -> u64 {
    self.rlimits[index].hard
  }

  pub(crate) fn rlimit_soft_at(&self, index: usize) -> u64 {
    self.rlimits[index].soft
  }

  getters! {
    copy no_new_privileges: bool;
    borrow capabilities: process::LinuxCapabilities;
    copy terminal: bool;
    copy stdin: Option<i32>;
    copy stdout: Option<i32>;
    copy stderr: Option<i32>;
  }
}

impl process::LinuxRLimit {
  pub(crate) fn kind(&self) -> ffi::RlimitKind {
    self.kind.into()
  }

  getters! {
    copy hard: u64;
    copy soft: u64;
  }
}

#[cfg(test)]
mod tests {
  use crate::bridge::ffi;
  use crate::containerization::process;
  use crate::containerization::process::linux_rlimit;
  use crate::containerization::process::signal;
  use crate::containerization_oci::runtime;
  use crate::containerization_os;

  #[test]
  fn copies_swifts_rlimit_kinds() {
    assert_eq!(
      ffi::cz_linux_rlimit_kind_descriptions(),
      linux_rlimit::Kind::ALL
        .iter()
        .map(|kind| kind.description())
        .collect::<Vec<_>>()
    );
  }

  #[test]
  fn copies_swifts_capability_presets() {
    let oci = |capabilities: process::LinuxCapabilities| {
      let set = |set: Vec<containerization_os::linux::CapabilityName>| {
        (!set.is_empty()).then(|| {
          set
            .iter()
            .map(|name| name.description().to_string())
            .collect()
        })
      };

      runtime::LinuxCapabilities {
        bounding: set(capabilities.bounding),
        effective: set(capabilities.effective),
        inheritable: set(capabilities.inheritable),
        permitted: set(capabilities.permitted),
        ambient: set(capabilities.ambient),
      }
    };

    assert_eq!(
      ffi::cz_linux_capabilities_presets().list(ffi::CzOutcome::oci_linux_capabilities),
      [
        oci(process::LinuxCapabilities::all_capabilities()),
        oci(process::LinuxCapabilities::default_oci_capabilities()),
      ]
    );
  }

  #[test]
  fn copies_swifts_signals() {
    let raw_values = |signals: &[process::Signal]| {
      signals
        .iter()
        .map(|signal| signal.raw_value)
        .collect::<Vec<_>>()
    };

    let mut linux = raw_values(signal::linux::ALL);
    linux.push(signal::linux::rtmin(0).raw_value);

    assert_eq!(ffi::cz_signal_linux_values(), linux);
    assert_eq!(ffi::cz_signal_darwin_values(), raw_values(signal::darwin::ALL));
  }
}
