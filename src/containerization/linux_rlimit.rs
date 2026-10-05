//! `LinuxRLimit`, and its nested `LinuxRLimit.Kind`.

/// `LinuxRLimit.Kind`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Kind {
  AddressSpace,
  CoreFileSize,
  CpuTime,
  DataSize,
  FileSize,
  Locks,
  LockedMemory,
  MessageQueue,
  Nice,
  OpenFiles,
  NumberOfProcesses,
  ResidentSetSize,
  RealtimePriority,
  RealtimeTimeout,
  SignalsPending,
  StackSize,
}

/// `LinuxRLimit`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LinuxRLimit {
  pub kind: Kind,
  pub hard: u64,
  pub soft: u64,
}

impl LinuxRLimit {
  /// `LinuxRLimit(kind:limit:)`.
  pub fn new(kind: Kind, limit: u64) -> Self {
    Self {
      kind,
      hard: limit,
      soft: limit,
    }
  }
}
