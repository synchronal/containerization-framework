//! `LinuxRLimit`, and its nested `LinuxRLimit.Kind`.

use crate::containerization_oci;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::fmt;

macro_rules! kinds {
  ($($case:ident = $description:literal),* $(,)?) => {
    /// `LinuxRLimit.Kind`. Each case is one of Swift's static properties.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub enum Kind {
      $(#[doc = concat!("`", $description, "`.")] $case),*
    }

    impl Kind {
      /// Every case, in the order Swift declares them.
      pub(crate) const ALL: &[Self] = &[$(Self::$case),*];

      /// `description`, the kind's OCI name, such as `RLIMIT_NOFILE`.
      pub fn description(self) -> &'static str {
        match self {
          $(Self::$case => $description),*
        }
      }
    }
  };
}

kinds!(
  AddressSpace = "RLIMIT_AS",
  CoreFileSize = "RLIMIT_CORE",
  CpuTime = "RLIMIT_CPU",
  DataSize = "RLIMIT_DATA",
  FileSize = "RLIMIT_FSIZE",
  Locks = "RLIMIT_LOCKS",
  LockedMemory = "RLIMIT_MEMLOCK",
  MessageQueue = "RLIMIT_MSGQUEUE",
  Nice = "RLIMIT_NICE",
  OpenFiles = "RLIMIT_NOFILE",
  NumberOfProcesses = "RLIMIT_NPROC",
  ResidentSetSize = "RLIMIT_RSS",
  RealtimePriority = "RLIMIT_RTPRIO",
  RealtimeTimeout = "RLIMIT_RTTIME",
  SignalsPending = "RLIMIT_SIGPENDING",
  StackSize = "RLIMIT_STACK",
);

impl Kind {
  /// `LinuxRLimit.Kind(_:)`, from a kind's OCI name, such as `RLIMIT_NOFILE`.
  pub fn parse(string: &str) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_linux_rlimit_kind_parse(string),
      format!("parse the resource limit kind {string}"),
    )
    .map(|outcome| Self::from_description(&outcome.text()))
  }

  /// The case Swift described. It panics if Swift has a case Rust lacks.
  fn from_description(description: &str) -> Self {
    Self::ALL
      .iter()
      .copied()
      .find(|case| case.description() == description)
      .unwrap_or_else(|| panic!("Swift's LinuxRLimit.Kind has a case Rust lacks: {description:?}"))
  }
}

impl fmt::Display for Kind {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(self.description())
  }
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
  pub fn to_oci(&self) -> Result<containerization_oci::runtime::POSIXRlimit, Error> {
    platform::outcome(ffi::cz_linux_rlimit_to_oci(*self), "convert a resource limit to OCI's")
      .map(|outcome| outcome.posix_rlimit())
  }
}
