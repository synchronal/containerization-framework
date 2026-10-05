//! `LinuxRLimit`, and its nested `LinuxRLimit.Kind`.

use crate::containerization_oci;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

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

  /// `LinuxRLimit.toOCI()`.
  pub fn to_oci(&self) -> Result<containerization_oci::POSIXRlimit, Error> {
    platform::outcome(ffi::cz_linux_rlimit_to_oci(*self), "convert a resource limit to OCI's")
      .map(|outcome| outcome.posix_rlimit())
  }
}
