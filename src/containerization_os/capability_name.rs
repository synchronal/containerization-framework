use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::fmt;

macro_rules! capability_names {
  ($($case:ident = $description:literal),* $(,)?) => {
    /// `CapabilityName`. Each case is one of Swift's static properties, and
    /// its discriminant is its `capValue`.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub enum CapabilityName {
      $(#[doc = concat!("`", $description, "`.")] $case),*
    }

    impl CapabilityName {
      /// `allCases`.
      pub const ALL_CASES: &[Self] = &[$(Self::$case),*];

      /// `description`, such as `CAP_CHOWN`.
      pub fn description(self) -> &'static str {
        match self {
          $(Self::$case => $description),*
        }
      }
    }
  };
}

capability_names!(
  Chown = "CAP_CHOWN",
  DacOverride = "CAP_DAC_OVERRIDE",
  DacReadSearch = "CAP_DAC_READ_SEARCH",
  Fowner = "CAP_FOWNER",
  Fsetid = "CAP_FSETID",
  Kill = "CAP_KILL",
  Setgid = "CAP_SETGID",
  Setuid = "CAP_SETUID",
  Setpcap = "CAP_SETPCAP",
  LinuxImmutable = "CAP_LINUX_IMMUTABLE",
  NetBindService = "CAP_NET_BIND_SERVICE",
  NetBroadcast = "CAP_NET_BROADCAST",
  NetAdmin = "CAP_NET_ADMIN",
  NetRaw = "CAP_NET_RAW",
  IpcLock = "CAP_IPC_LOCK",
  IpcOwner = "CAP_IPC_OWNER",
  SysModule = "CAP_SYS_MODULE",
  SysRawio = "CAP_SYS_RAWIO",
  SysChroot = "CAP_SYS_CHROOT",
  SysPtrace = "CAP_SYS_PTRACE",
  SysPacct = "CAP_SYS_PACCT",
  SysAdmin = "CAP_SYS_ADMIN",
  SysBoot = "CAP_SYS_BOOT",
  SysNice = "CAP_SYS_NICE",
  SysResource = "CAP_SYS_RESOURCE",
  SysTime = "CAP_SYS_TIME",
  SysTtyConfig = "CAP_SYS_TTY_CONFIG",
  Mknod = "CAP_MKNOD",
  Lease = "CAP_LEASE",
  AuditWrite = "CAP_AUDIT_WRITE",
  AuditControl = "CAP_AUDIT_CONTROL",
  Setfcap = "CAP_SETFCAP",
  MacOverride = "CAP_MAC_OVERRIDE",
  MacAdmin = "CAP_MAC_ADMIN",
  Syslog = "CAP_SYSLOG",
  WakeAlarm = "CAP_WAKE_ALARM",
  BlockSuspend = "CAP_BLOCK_SUSPEND",
  AuditRead = "CAP_AUDIT_READ",
  Perfmon = "CAP_PERFMON",
  Bpf = "CAP_BPF",
  CheckpointRestore = "CAP_CHECKPOINT_RESTORE",
);

impl CapabilityName {
  /// `CapabilityName(rawValue:)`, which ignores case and accepts a name with
  /// or without its `CAP_` prefix, such as `chown`.
  pub fn parse(raw_value: &str) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_capability_name_parse(raw_value),
      format!("parse the capability {raw_value}"),
    )
    .map(|outcome| Self::from_description(&outcome.text()))
  }

  /// `capValue`.
  pub fn cap_value(self) -> u32 {
    self as u32
  }

  /// The case Swift described. It panics if Swift has a case Rust lacks.
  pub(crate) fn from_description(description: &str) -> Self {
    Self::ALL_CASES
      .iter()
      .copied()
      .find(|case| case.description() == description)
      .unwrap_or_else(|| panic!("Swift's CapabilityName has a case Rust lacks: {description:?}"))
  }
}

impl fmt::Display for CapabilityName {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(self.description())
  }
}
