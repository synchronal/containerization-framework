//! Getters and setters for a process's configuration: its user, rlimits and
//! capabilities.

use crate::bridge::ffi;
use crate::containerization::process;
use crate::containerization::process::linux_rlimit;
use crate::containerization_oci::runtime;
use crate::containerization_os;

impl runtime::User {
  pub(crate) fn uid(&self) -> u32 {
    self.uid
  }

  pub(crate) fn gid(&self) -> u32 {
    self.gid
  }

  pub(crate) fn umask(&self) -> Option<u32> {
    self.umask
  }

  pub(crate) fn additional_gids_len(&self) -> usize {
    self.additional_gids.len()
  }

  pub(crate) fn additional_gids_at(&self, index: usize) -> u32 {
    self.additional_gids[index]
  }

  pub(crate) fn username(&self) -> &str {
    &self.username
  }
}

impl process::LinuxCapabilities {
  pub(crate) fn set_len(&self, set: ffi::CapabilitySet) -> usize {
    self.set(set).len()
  }

  pub(crate) fn set_at(&self, set: ffi::CapabilitySet, index: usize) -> &str {
    self.set(set)[index].description()
  }

  fn set(&self, set: ffi::CapabilitySet) -> &[containerization_os::CapabilityName] {
    match set {
      ffi::CapabilitySet::Bounding => &self.bounding,
      ffi::CapabilitySet::Effective => &self.effective,
      ffi::CapabilitySet::Inheritable => &self.inheritable,
      ffi::CapabilitySet::Permitted => &self.permitted,
      ffi::CapabilitySet::Ambient => &self.ambient,
    }
  }
}

fn rlimit_kind(kind: linux_rlimit::Kind) -> ffi::RlimitKind {
  match kind {
    linux_rlimit::Kind::AddressSpace => ffi::RlimitKind::AddressSpace,
    linux_rlimit::Kind::CoreFileSize => ffi::RlimitKind::CoreFileSize,
    linux_rlimit::Kind::CpuTime => ffi::RlimitKind::CpuTime,
    linux_rlimit::Kind::DataSize => ffi::RlimitKind::DataSize,
    linux_rlimit::Kind::FileSize => ffi::RlimitKind::FileSize,
    linux_rlimit::Kind::Locks => ffi::RlimitKind::Locks,
    linux_rlimit::Kind::LockedMemory => ffi::RlimitKind::LockedMemory,
    linux_rlimit::Kind::MessageQueue => ffi::RlimitKind::MessageQueue,
    linux_rlimit::Kind::Nice => ffi::RlimitKind::Nice,
    linux_rlimit::Kind::OpenFiles => ffi::RlimitKind::OpenFiles,
    linux_rlimit::Kind::NumberOfProcesses => ffi::RlimitKind::NumberOfProcesses,
    linux_rlimit::Kind::ResidentSetSize => ffi::RlimitKind::ResidentSetSize,
    linux_rlimit::Kind::RealtimePriority => ffi::RlimitKind::RealtimePriority,
    linux_rlimit::Kind::RealtimeTimeout => ffi::RlimitKind::RealtimeTimeout,
    linux_rlimit::Kind::SignalsPending => ffi::RlimitKind::SignalsPending,
    linux_rlimit::Kind::StackSize => ffi::RlimitKind::StackSize,
  }
}

fn linux_rlimit_kind(kind: ffi::RlimitKind) -> linux_rlimit::Kind {
  match kind {
    ffi::RlimitKind::AddressSpace => linux_rlimit::Kind::AddressSpace,
    ffi::RlimitKind::CoreFileSize => linux_rlimit::Kind::CoreFileSize,
    ffi::RlimitKind::CpuTime => linux_rlimit::Kind::CpuTime,
    ffi::RlimitKind::DataSize => linux_rlimit::Kind::DataSize,
    ffi::RlimitKind::FileSize => linux_rlimit::Kind::FileSize,
    ffi::RlimitKind::Locks => linux_rlimit::Kind::Locks,
    ffi::RlimitKind::LockedMemory => linux_rlimit::Kind::LockedMemory,
    ffi::RlimitKind::MessageQueue => linux_rlimit::Kind::MessageQueue,
    ffi::RlimitKind::Nice => linux_rlimit::Kind::Nice,
    ffi::RlimitKind::OpenFiles => linux_rlimit::Kind::OpenFiles,
    ffi::RlimitKind::NumberOfProcesses => linux_rlimit::Kind::NumberOfProcesses,
    ffi::RlimitKind::ResidentSetSize => linux_rlimit::Kind::ResidentSetSize,
    ffi::RlimitKind::RealtimePriority => linux_rlimit::Kind::RealtimePriority,
    ffi::RlimitKind::RealtimeTimeout => linux_rlimit::Kind::RealtimeTimeout,
    ffi::RlimitKind::SignalsPending => linux_rlimit::Kind::SignalsPending,
    ffi::RlimitKind::StackSize => linux_rlimit::Kind::StackSize,
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
        .map(|description| containerization_os::CapabilityName::from_description(description))
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
      kind: linux_rlimit_kind(kind),
      hard,
      soft,
    });
  }

  pub(crate) fn arguments_len(&self) -> usize {
    self.arguments.len()
  }

  pub(crate) fn arguments_at(&self, index: usize) -> &str {
    &self.arguments[index]
  }

  pub(crate) fn environment_variables_len(&self) -> usize {
    self.environment_variables.len()
  }

  pub(crate) fn environment_variables_at(&self, index: usize) -> &str {
    &self.environment_variables[index]
  }

  pub(crate) fn working_directory(&self) -> &str {
    &self.working_directory
  }

  pub(crate) fn user(&self) -> &runtime::User {
    &self.user
  }

  pub(crate) fn rlimits_len(&self) -> usize {
    self.rlimits.len()
  }

  pub(crate) fn rlimit_kind_at(&self, index: usize) -> ffi::RlimitKind {
    rlimit_kind(self.rlimits[index].kind)
  }

  pub(crate) fn rlimit_hard_at(&self, index: usize) -> u64 {
    self.rlimits[index].hard
  }

  pub(crate) fn rlimit_soft_at(&self, index: usize) -> u64 {
    self.rlimits[index].soft
  }

  pub(crate) fn no_new_privileges(&self) -> bool {
    self.no_new_privileges
  }

  pub(crate) fn capabilities(&self) -> &process::LinuxCapabilities {
    &self.capabilities
  }

  pub(crate) fn terminal(&self) -> bool {
    self.terminal
  }

  pub(crate) fn stdin(&self) -> Option<i32> {
    self.stdin
  }

  pub(crate) fn stdout(&self) -> Option<i32> {
    self.stdout
  }

  pub(crate) fn stderr(&self) -> Option<i32> {
    self.stderr
  }
}

impl process::LinuxRLimit {
  pub(crate) fn kind(&self) -> ffi::RlimitKind {
    rlimit_kind(self.kind)
  }

  pub(crate) fn hard(&self) -> u64 {
    self.hard
  }

  pub(crate) fn soft(&self) -> u64 {
    self.soft
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
      let set = |set: Vec<containerization_os::CapabilityName>| {
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
