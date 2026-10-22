use crate::platform::ffi;

/// `VirtualMachineInstanceState`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VirtualMachineInstanceState {
  Starting,
  Running,
  Stopped,
  Stopping,
  Unknown,
}

same_variants!(
  VirtualMachineInstanceState,
  ffi::InstanceState {
    Starting,
    Running,
    Stopped,
    Stopping,
    Unknown,
  }
);
