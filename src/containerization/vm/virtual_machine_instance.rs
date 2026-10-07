/// `VirtualMachineInstanceState`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VirtualMachineInstanceState {
  Starting,
  Running,
  Stopped,
  Stopping,
  Unknown,
}

/// `VirtiofsLayout`: how a VM shows its guest virtiofs devices.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VirtiofsLayout {
  /// One device, tagged `virtiofs`, with each share a directory in it.
  Unified,
  /// One device per share.
  PerTag,
}
