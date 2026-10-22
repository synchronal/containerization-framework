//! The OCI runtime spec's process, its user, capabilities and rlimits. The
//! getters Swift reads a `User` through are in `container/process.rs`, beside
//! the `LinuxProcessConfiguration` that holds one.

use crate::bridge::accessors::present;
use crate::bridge::ffi;
use crate::containerization_oci::runtime;

impl ffi::CzOutcome {
  pub(crate) fn process(&self) -> runtime::Process {
    runtime::Process {
      cwd: self.process_cwd(),
      env: self.process_env(),
      console_size: self
        .process_console_size()
        .optional(|size| runtime::Box::new(size.box_height(), size.box_width())),
      selinux_label: self.process_selinux_label(),
      no_new_privileges: self.process_no_new_privileges(),
      command_line: self.process_command_line(),
      oom_score_adj: self.process_oom_score_adj(),
      capabilities: self
        .process_capabilities()
        .optional(Self::oci_linux_capabilities),
      apparmor_profile: self.process_apparmor_profile(),
      user: self.process_user().user(),
      rlimits: self.process_rlimits().list(Self::posix_rlimit),
      args: self.process_args(),
      terminal: self.process_terminal(),
    }
  }

  fn user(&self) -> runtime::User {
    runtime::User {
      uid: self.user_uid(),
      gid: self.user_gid(),
      umask: self.user_umask(),
      additional_gids: self.user_additional_gids(),
      username: self.user_username(),
    }
  }

  pub(crate) fn oci_linux_capabilities(&self) -> runtime::LinuxCapabilities {
    let set = |set| self.capabilities_set(set).optional(Self::strings);

    runtime::LinuxCapabilities {
      bounding: set(ffi::CapabilitySet::Bounding),
      effective: set(ffi::CapabilitySet::Effective),
      inheritable: set(ffi::CapabilitySet::Inheritable),
      permitted: set(ffi::CapabilitySet::Permitted),
      ambient: set(ffi::CapabilitySet::Ambient),
    }
  }

  pub(crate) fn posix_rlimit(&self) -> runtime::POSIXRlimit {
    runtime::POSIXRlimit {
      r#type: self.rlimit_type(),
      hard: self.rlimit_hard(),
      soft: self.rlimit_soft(),
    }
  }
}

impl runtime::Process {
  getters! {
    str cwd;
    list env: str => env_len, env_at;
  }

  pub(crate) fn has_console_size(&self) -> bool {
    self.console_size.is_some()
  }

  pub(crate) fn console_height(&self) -> usize {
    present(&self.console_size).height
  }

  pub(crate) fn console_width(&self) -> usize {
    present(&self.console_size).width
  }

  getters! {
    str selinux_label;
    copy no_new_privileges: bool;
    str command_line;
    copy oom_score_adj: Option<isize>;
    present capabilities: runtime::LinuxCapabilities => has_capabilities;
    str apparmor_profile;
    borrow user: runtime::User;
  }

  pub(crate) fn rlimits_len(&self) -> usize {
    self.rlimits.len()
  }

  pub(crate) fn rlimit_type_at(&self, index: usize) -> &str {
    &self.rlimits[index].r#type
  }

  pub(crate) fn rlimit_hard_at(&self, index: usize) -> u64 {
    self.rlimits[index].hard
  }

  pub(crate) fn rlimit_soft_at(&self, index: usize) -> u64 {
    self.rlimits[index].soft
  }

  getters! {
    list args: str => args_len, args_at;
    copy terminal: bool;
  }
}

impl runtime::LinuxCapabilities {
  pub(crate) fn has_set(&self, set: ffi::CapabilitySet) -> bool {
    self.set(set).is_some()
  }

  pub(crate) fn set_len(&self, set: ffi::CapabilitySet) -> usize {
    self.set(set).as_ref().map_or(0, Vec::len)
  }

  pub(crate) fn set_at(&self, set: ffi::CapabilitySet, index: usize) -> &str {
    &present(self.set(set))[index]
  }

  fn set(&self, set: ffi::CapabilitySet) -> &Option<Vec<String>> {
    match set {
      ffi::CapabilitySet::Bounding => &self.bounding,
      ffi::CapabilitySet::Effective => &self.effective,
      ffi::CapabilitySet::Inheritable => &self.inheritable,
      ffi::CapabilitySet::Permitted => &self.permitted,
      ffi::CapabilitySet::Ambient => &self.ambient,
    }
  }
}
