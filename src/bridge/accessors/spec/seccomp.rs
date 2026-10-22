//! The OCI runtime spec's seccomp profile and its syscall rules.

use crate::bridge::ffi;
use crate::containerization_oci::runtime;

impl ffi::CzOutcome {
  /// The `LinuxSeccomp` an outcome holds, read field by field.
  pub(crate) fn seccomp(&self) -> runtime::LinuxSeccomp {
    runtime::LinuxSeccomp {
      default_action: runtime::LinuxSeccompAction::from_swift(&self.seccomp_default_action()),
      default_errno_ret: self.seccomp_default_errno_ret(),
      architectures: self
        .seccomp_architectures()
        .iter()
        .map(|arch| runtime::Arch::from_swift(arch))
        .collect(),
      flags: self
        .seccomp_flags()
        .iter()
        .map(|flag| runtime::LinuxSeccompFlag::from_swift(flag))
        .collect(),
      listener_path: self.seccomp_listener_path(),
      listener_metadata: self.seccomp_listener_metadata(),
      syscalls: self.seccomp_syscalls().list(Self::syscall),
    }
  }

  fn syscall(&self) -> runtime::LinuxSyscall {
    runtime::LinuxSyscall {
      names: self.syscall_names(),
      action: runtime::LinuxSeccompAction::from_swift(&self.syscall_action()),
      errno_ret: self.syscall_errno_ret(),
      args: self.syscall_args().list(|arg| runtime::LinuxSeccompArg {
        index: arg.seccomp_arg_index(),
        value: arg.seccomp_arg_value(),
        value_two: arg.seccomp_arg_value_two(),
        op: runtime::LinuxSeccompOperator::from_swift(&arg.seccomp_arg_op()),
      }),
    }
  }
}

impl runtime::LinuxSeccomp {
  pub(crate) fn default_action(&self) -> &str {
    self.default_action.raw_value()
  }

  getters! {
    copy default_errno_ret: Option<usize>;
  }

  pub(crate) fn architectures_len(&self) -> usize {
    self.architectures.len()
  }

  pub(crate) fn architectures_at(&self, index: usize) -> &str {
    self.architectures[index].raw_value()
  }

  pub(crate) fn flags_len(&self) -> usize {
    self.flags.len()
  }

  pub(crate) fn flags_at(&self, index: usize) -> &str {
    self.flags[index].raw_value()
  }

  getters! {
    str listener_path;
    str listener_metadata;
    list syscalls: runtime::LinuxSyscall => syscalls_len, syscalls_at;
  }
}

impl runtime::LinuxSyscall {
  getters! {
    list names: str => names_len, names_at;
  }

  pub(crate) fn action(&self) -> &str {
    self.action.raw_value()
  }

  getters! {
    copy errno_ret: Option<usize>;
  }

  pub(crate) fn args_len(&self) -> usize {
    self.args.len()
  }

  pub(crate) fn arg_index_at(&self, index: usize) -> usize {
    self.args[index].index
  }

  pub(crate) fn arg_value_at(&self, index: usize) -> u64 {
    self.args[index].value
  }

  pub(crate) fn arg_value_two_at(&self, index: usize) -> u64 {
    self.args[index].value_two
  }

  pub(crate) fn arg_op_at(&self, index: usize) -> &str {
    self.args[index].op.raw_value()
  }
}
