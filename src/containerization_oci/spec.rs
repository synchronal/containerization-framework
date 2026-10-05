//! The OCI runtime spec, from Swift's `Spec.swift`: `Spec` and every type it
//! holds.

use super::ImageConfig;
use super::User;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::collections::BTreeMap;

/// `Spec`. Its `Default` is `Spec()`, which holds nothing.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Spec {
  pub version: String,
  pub hooks: Option<Hooks>,
  pub process: Option<Process>,
  pub hostname: String,
  pub domainname: String,
  pub mounts: Vec<Mount>,
  pub annotations: Option<BTreeMap<String, String>>,
  pub root: Option<Root>,
  pub linux: Option<Linux>,
}

/// `Process`. Its `Default` is `Process()`, which runs nothing in `/` as root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Process {
  pub cwd: String,
  pub env: Vec<String>,
  pub console_size: Option<Box>,
  pub selinux_label: String,
  pub no_new_privileges: bool,
  pub command_line: String,
  pub oom_score_adj: Option<isize>,
  pub capabilities: Option<LinuxCapabilities>,
  pub apparmor_profile: String,
  pub user: User,
  pub rlimits: Vec<POSIXRlimit>,
  pub args: Vec<String>,
  pub terminal: bool,
}

impl Default for Process {
  fn default() -> Self {
    Self {
      cwd: "/".to_string(),
      env: Vec::new(),
      console_size: None,
      selinux_label: String::new(),
      no_new_privileges: false,
      command_line: String::new(),
      oom_score_adj: None,
      capabilities: None,
      apparmor_profile: String::new(),
      user: User::default(),
      rlimits: Vec::new(),
      args: Vec::new(),
      terminal: false,
    }
  }
}

impl Process {
  /// `Process(from: ImageConfig)`.
  pub fn from_image_config(config: &ImageConfig) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_process_from_image_config(config.clone()),
      "make a process from an image config",
    )
    .map(|outcome| outcome.process())
  }

  /// `Process.description`, with the values of `env` redacted.
  pub fn description(&self) -> Result<String, Error> {
    platform::outcome(ffi::cz_process_description(self.clone()), "describe a process").map(|outcome| outcome.text())
  }
}

/// The OCI `LinuxCapabilities`, not
/// [`crate::containerization::LinuxCapabilities`]. Its `Default` holds nothing.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LinuxCapabilities {
  pub bounding: Option<Vec<String>>,
  pub effective: Option<Vec<String>>,
  pub inheritable: Option<Vec<String>>,
  pub permitted: Option<Vec<String>>,
  pub ambient: Option<Vec<String>>,
}

/// `Box`, a console's size. As in Swift, its fields are private.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Box {
  pub(crate) height: usize,
  pub(crate) width: usize,
}

impl Box {
  /// `Box(height:width:)`.
  pub fn new(height: usize, width: usize) -> Self {
    Self { height, width }
  }
}

/// `Root`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Root {
  pub path: String,
  pub readonly: bool,
}

impl Root {
  /// `Root(path:readonly:)`.
  pub fn new(path: impl Into<String>, readonly: bool) -> Self {
    Self {
      path: path.into(),
      readonly,
    }
  }
}

/// The OCI `Mount`, not [`crate::containerization::Mount`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mount {
  pub r#type: String,
  pub source: String,
  pub destination: String,
  pub options: Vec<String>,
  pub uid_mappings: Option<Vec<LinuxIDMapping>>,
  pub gid_mappings: Option<Vec<LinuxIDMapping>>,
}

impl Mount {
  /// `Mount(destination:)`, with every other field empty.
  pub fn new(destination: impl Into<String>) -> Self {
    Self {
      r#type: String::new(),
      source: String::new(),
      destination: destination.into(),
      options: Vec::new(),
      uid_mappings: None,
      gid_mappings: None,
    }
  }
}

/// `Hook`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hook {
  pub path: String,
  pub args: Vec<String>,
  pub env: Vec<String>,
  pub timeout: Option<isize>,
}

impl Hook {
  /// `Hook(path:args:env:timeout:)`.
  pub fn new(path: impl Into<String>, args: Vec<String>, env: Vec<String>, timeout: Option<isize>) -> Self {
    Self {
      path: path.into(),
      args,
      env,
      timeout,
    }
  }

  /// `Hook.description`, with the values of `env` redacted.
  pub fn description(&self) -> Result<String, Error> {
    platform::outcome(ffi::cz_hook_description(self.clone()), "describe a hook").map(|outcome| outcome.text())
  }
}

/// `Hooks`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hooks {
  pub prestart: Vec<Hook>,
  pub create_runtime: Vec<Hook>,
  pub create_container: Vec<Hook>,
  pub start_container: Vec<Hook>,
  pub poststart: Vec<Hook>,
  pub poststop: Vec<Hook>,
}

impl Hooks {
  /// `Hooks(prestart:createRuntime:createContainer:startContainer:poststart:poststop:)`.
  pub fn new(
    prestart: Vec<Hook>,
    create_runtime: Vec<Hook>,
    create_container: Vec<Hook>,
    start_container: Vec<Hook>,
    poststart: Vec<Hook>,
    poststop: Vec<Hook>,
  ) -> Self {
    Self {
      prestart,
      create_runtime,
      create_container,
      start_container,
      poststart,
      poststop,
    }
  }
}

/// `Linux`. Its `Default` is `Linux()`, which holds nothing.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Linux {
  pub uid_mappings: Vec<LinuxIDMapping>,
  pub gid_mappings: Vec<LinuxIDMapping>,
  pub sysctl: Option<BTreeMap<String, String>>,
  pub resources: Option<LinuxResources>,
  pub cgroups_path: String,
  pub namespaces: Vec<LinuxNamespace>,
  pub devices: Vec<LinuxDevice>,
  pub seccomp: Option<LinuxSeccomp>,
  pub rootfs_propagation: String,
  pub masked_paths: Vec<String>,
  pub readonly_paths: Vec<String>,
  pub mount_label: String,
  pub personality: Option<LinuxPersonality>,
}

/// `LinuxNamespace`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinuxNamespace {
  pub r#type: LinuxNamespaceType,
  pub path: String,
}

impl LinuxNamespace {
  /// `LinuxNamespace(type:)`, with no path.
  pub fn new(r#type: LinuxNamespaceType) -> Self {
    Self {
      r#type,
      path: String::new(),
    }
  }
}

raw_values!(
  /// `LinuxNamespaceType`.
  LinuxNamespaceType {
    Pid = "pid",
    Network = "network",
    Uts = "uts",
    Mount = "mount",
    Ipc = "ipc",
    User = "user",
    Cgroup = "cgroup",
  }
);

/// `LinuxIDMapping`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LinuxIDMapping {
  pub container_id: u32,
  pub host_id: u32,
  pub size: u32,
}

impl LinuxIDMapping {
  /// `LinuxIDMapping(containerID:hostID:size:)`.
  pub fn new(container_id: u32, host_id: u32, size: u32) -> Self {
    Self {
      container_id,
      host_id,
      size,
    }
  }
}

/// `POSIXRlimit`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct POSIXRlimit {
  pub r#type: String,
  pub hard: u64,
  pub soft: u64,
}

impl POSIXRlimit {
  /// `POSIXRlimit(type:hard:soft:)`.
  pub fn new(r#type: impl Into<String>, hard: u64, soft: u64) -> Self {
    Self {
      r#type: r#type.into(),
      hard,
      soft,
    }
  }
}

/// `LinuxHugepageLimit`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LinuxHugepageLimit {
  pub pagesize: String,
  pub limit: u64,
}

impl LinuxHugepageLimit {
  /// `LinuxHugepageLimit(pagesize:limit:)`.
  pub fn new(pagesize: impl Into<String>, limit: u64) -> Self {
    Self {
      pagesize: pagesize.into(),
      limit,
    }
  }
}

/// `LinuxInterfacePriority`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LinuxInterfacePriority {
  pub name: String,
  pub priority: u32,
}

impl LinuxInterfacePriority {
  /// `LinuxInterfacePriority(name:priority:)`.
  pub fn new(name: impl Into<String>, priority: u32) -> Self {
    Self {
      name: name.into(),
      priority,
    }
  }
}

/// `LinuxBlockIODevice`. No other type holds one.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LinuxBlockIODevice {
  pub major: i64,
  pub minor: i64,
}

impl LinuxBlockIODevice {
  /// `LinuxBlockIODevice(major:minor:)`.
  pub fn new(major: i64, minor: i64) -> Self {
    Self { major, minor }
  }
}

/// `LinuxWeightDevice`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LinuxWeightDevice {
  pub major: i64,
  pub minor: i64,
  pub weight: Option<u16>,
  pub leaf_weight: Option<u16>,
}

impl LinuxWeightDevice {
  /// `LinuxWeightDevice(major:minor:weight:leafWeight:)`.
  pub fn new(major: i64, minor: i64, weight: Option<u16>, leaf_weight: Option<u16>) -> Self {
    Self {
      major,
      minor,
      weight,
      leaf_weight,
    }
  }
}

/// `LinuxThrottleDevice`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LinuxThrottleDevice {
  pub major: i64,
  pub minor: i64,
  pub rate: u64,
}

impl LinuxThrottleDevice {
  /// `LinuxThrottleDevice(major:minor:rate:)`.
  pub fn new(major: i64, minor: i64, rate: u64) -> Self {
    Self { major, minor, rate }
  }
}

/// `LinuxBlockIO`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinuxBlockIO {
  pub weight: Option<u16>,
  pub leaf_weight: Option<u16>,
  pub weight_device: Vec<LinuxWeightDevice>,
  pub throttle_read_bps_device: Vec<LinuxThrottleDevice>,
  pub throttle_write_bps_device: Vec<LinuxThrottleDevice>,
  pub throttle_read_iops_device: Vec<LinuxThrottleDevice>,
  pub throttle_write_iops_device: Vec<LinuxThrottleDevice>,
}

impl LinuxBlockIO {
  /// `LinuxBlockIO(weight:leafWeight:weightDevice:throttleReadBpsDevice:throttleWriteBpsDevice:throttleReadIOPSDevice:throttleWriteIOPSDevice:)`.
  pub fn new(
    weight: Option<u16>,
    leaf_weight: Option<u16>,
    weight_device: Vec<LinuxWeightDevice>,
    throttle_read_bps_device: Vec<LinuxThrottleDevice>,
    throttle_write_bps_device: Vec<LinuxThrottleDevice>,
    throttle_read_iops_device: Vec<LinuxThrottleDevice>,
    throttle_write_iops_device: Vec<LinuxThrottleDevice>,
  ) -> Self {
    Self {
      weight,
      leaf_weight,
      weight_device,
      throttle_read_bps_device,
      throttle_write_bps_device,
      throttle_read_iops_device,
      throttle_write_iops_device,
    }
  }
}

/// `LinuxMemory`. Its `Default` is `LinuxMemory()`, which holds nothing.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct LinuxMemory {
  pub limit: Option<i64>,
  pub reservation: Option<i64>,
  pub swap: Option<i64>,
  pub kernel: Option<i64>,
  pub kernel_tcp: Option<i64>,
  pub swappiness: Option<u64>,
  pub disable_oom_killer: Option<bool>,
  pub use_hierarchy: Option<bool>,
  pub check_before_update: Option<bool>,
}

/// `LinuxCPU`. Its `Default` is `LinuxCPU()`, which holds nothing.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct LinuxCPU {
  pub shares: Option<u64>,
  pub quota: Option<i64>,
  pub burst: Option<u64>,
  pub period: Option<u64>,
  pub realtime_runtime: Option<i64>,
  pub realtime_period: Option<i64>,
  pub cpus: String,
  pub mems: String,
  pub idle: Option<i64>,
}

/// `LinuxPids`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LinuxPids {
  pub limit: i64,
}

impl LinuxPids {
  /// `LinuxPids(limit:)`.
  pub fn new(limit: i64) -> Self {
    Self { limit }
  }
}

/// `LinuxNetwork`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LinuxNetwork {
  pub class_id: Option<u32>,
  pub priorities: Vec<LinuxInterfacePriority>,
}

impl LinuxNetwork {
  /// `LinuxNetwork(classID:priorities:)`.
  pub fn new(class_id: Option<u32>, priorities: Vec<LinuxInterfacePriority>) -> Self {
    Self { class_id, priorities }
  }
}

/// `LinuxRdma`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LinuxRdma {
  pub hcs_handles: Option<u32>,
  pub hca_objects: Option<u32>,
}

impl LinuxRdma {
  /// `LinuxRdma(hcsHandles:hcaObjects:)`.
  pub fn new(hcs_handles: Option<u32>, hca_objects: Option<u32>) -> Self {
    Self {
      hcs_handles,
      hca_objects,
    }
  }
}

/// `LinuxResources`. Its `Default` is `LinuxResources()`, which holds nothing
/// but an empty `unified`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinuxResources {
  pub devices: Vec<LinuxDeviceCgroup>,
  pub memory: Option<LinuxMemory>,
  pub cpu: Option<LinuxCPU>,
  pub pids: Option<LinuxPids>,
  pub block_io: Option<LinuxBlockIO>,
  pub hugepage_limits: Vec<LinuxHugepageLimit>,
  pub network: Option<LinuxNetwork>,
  pub rdma: Option<BTreeMap<String, LinuxRdma>>,
  pub unified: Option<BTreeMap<String, String>>,
}

impl Default for LinuxResources {
  fn default() -> Self {
    Self {
      devices: Vec::new(),
      memory: None,
      cpu: None,
      pids: None,
      block_io: None,
      hugepage_limits: Vec::new(),
      network: None,
      rdma: None,
      unified: Some(BTreeMap::new()),
    }
  }
}

/// `LinuxDevice`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LinuxDevice {
  pub path: String,
  pub r#type: String,
  pub major: i64,
  pub minor: i64,
  pub file_mode: Option<u32>,
  pub uid: Option<u32>,
  pub gid: Option<u32>,
}

impl LinuxDevice {
  /// `LinuxDevice(path:type:major:minor:fileMode:uid:gid:)`.
  pub fn new(
    path: impl Into<String>,
    r#type: impl Into<String>,
    major: i64,
    minor: i64,
    file_mode: Option<u32>,
    uid: Option<u32>,
    gid: Option<u32>,
  ) -> Self {
    Self {
      path: path.into(),
      r#type: r#type.into(),
      major,
      minor,
      file_mode,
      uid,
      gid,
    }
  }
}

/// `LinuxDeviceCgroup`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LinuxDeviceCgroup {
  pub allow: bool,
  pub r#type: String,
  pub major: Option<i64>,
  pub minor: Option<i64>,
  pub access: Option<String>,
}

impl LinuxDeviceCgroup {
  /// `LinuxDeviceCgroup(allow:type:major:minor:access:)`.
  pub fn new(
    allow: bool,
    r#type: impl Into<String>,
    major: Option<i64>,
    minor: Option<i64>,
    access: Option<String>,
  ) -> Self {
    Self {
      allow,
      r#type: r#type.into(),
      major,
      minor,
      access,
    }
  }
}

raw_values!(
  /// `LinuxPersonalityDomain`.
  LinuxPersonalityDomain {
    PerLinux = "LINUX",
    PerLinux32 = "LINUX32",
  }
);

/// `LinuxPersonality`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LinuxPersonality {
  pub domain: LinuxPersonalityDomain,
  pub flags: Vec<String>,
}

impl LinuxPersonality {
  /// `LinuxPersonality(domain:flags:)`.
  pub fn new(domain: LinuxPersonalityDomain, flags: Vec<String>) -> Self {
    Self { domain, flags }
  }
}

/// `LinuxSeccomp`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LinuxSeccomp {
  pub default_action: LinuxSeccompAction,
  pub default_errno_ret: Option<usize>,
  pub architectures: Vec<Arch>,
  pub flags: Vec<LinuxSeccompFlag>,
  pub listener_path: String,
  pub listener_metadata: String,
  pub syscalls: Vec<LinuxSyscall>,
}

impl LinuxSeccomp {
  /// `LinuxSeccomp(defaultAction:defaultErrnoRet:architectures:flags:listenerPath:listenerMetadata:syscalls:)`.
  pub fn new(
    default_action: LinuxSeccompAction,
    default_errno_ret: Option<usize>,
    architectures: Vec<Arch>,
    flags: Vec<LinuxSeccompFlag>,
    listener_path: impl Into<String>,
    listener_metadata: impl Into<String>,
    syscalls: Vec<LinuxSyscall>,
  ) -> Self {
    Self {
      default_action,
      default_errno_ret,
      architectures,
      flags,
      listener_path: listener_path.into(),
      listener_metadata: listener_metadata.into(),
      syscalls,
    }
  }

  /// `LinuxSeccomp.decode(from:)`, which reads the JSON of an OCI runtime
  /// spec's `linux.seccomp` and rejects Docker's profile format.
  pub fn decode(data: &[u8]) -> Result<Self, Error> {
    platform::outcome(ffi::cz_linux_seccomp_decode(data.to_vec()), "decode a seccomp profile")
      .map(|outcome| outcome.seccomp())
  }

  /// `LinuxSeccomp.defaultProfile(capabilities:arch:)`.
  pub fn default_profile(capabilities: Option<&LinuxCapabilities>, arch: Arch) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_linux_seccomp_default_profile(
        capabilities.is_some(),
        capabilities.cloned().unwrap_or_default(),
        arch.raw_value(),
      ),
      "make the default seccomp profile",
    )
    .map(|outcome| outcome.seccomp())
  }
}

raw_values!(
  /// `LinuxSeccompFlag`.
  LinuxSeccompFlag {
    FlagLog = "SECCOMP_FILTER_FLAG_LOG",
    FlagSpecAllow = "SECCOMP_FILTER_FLAG_SPEC_ALLOW",
    FlagWaitKillableRecv = "SECCOMP_FILTER_FLAG_WAIT_KILLABLE_RECV",
  }
);

raw_values!(
  /// `Arch`, a seccomp architecture.
  Arch {
    ArchX86 = "SCMP_ARCH_X86",
    ArchX86_64 = "SCMP_ARCH_X86_64",
    ArchX32 = "SCMP_ARCH_X32",
    ArchARM = "SCMP_ARCH_ARM",
    ArchAARCH64 = "SCMP_ARCH_AARCH64",
    ArchMIPS = "SCMP_ARCH_MIPS",
    ArchMIPS64 = "SCMP_ARCH_MIPS64",
    ArchMIPS64N32 = "SCMP_ARCH_MIPS64N32",
    ArchMIPSEL = "SCMP_ARCH_MIPSEL",
    ArchMIPSEL64 = "SCMP_ARCH_MIPSEL64",
    ArchMIPSEL64N32 = "SCMP_ARCH_MIPSEL64N32",
    ArchPPC = "SCMP_ARCH_PPC",
    ArchPPC64 = "SCMP_ARCH_PPC64",
    ArchPPC64LE = "SCMP_ARCH_PPC64LE",
    ArchS390 = "SCMP_ARCH_S390",
    ArchS390X = "SCMP_ARCH_S390X",
    ArchPARISC = "SCMP_ARCH_PARISC",
    ArchPARISC64 = "SCMP_ARCH_PARISC64",
    ArchRISCV64 = "SCMP_ARCH_RISCV64",
  }
);

impl Arch {
  /// `Arch.current`: the architecture Swift was built for, or `None` if
  /// `Arch` has no case for it.
  pub fn current() -> Result<Option<Self>, Error> {
    platform::outcome(ffi::cz_arch_current(), "read the current seccomp architecture").map(|outcome| {
      outcome
        .optional_text()
        .map(|raw_value| Self::from_swift(&raw_value))
    })
  }

  /// `Arch.currentVerified()`, which also fails when `current` is `None` or
  /// isn't the architecture running.
  pub fn current_verified() -> Result<Self, Error> {
    platform::outcome(ffi::cz_arch_current_verified(), "read the current seccomp architecture")
      .map(|outcome| Self::from_swift(&outcome.text()))
  }
}

raw_values!(
  /// `LinuxSeccompAction`.
  LinuxSeccompAction {
    ActKill = "SCMP_ACT_KILL",
    ActKillProcess = "SCMP_ACT_KILL_PROCESS",
    ActKillThread = "SCMP_ACT_KILL_THREAD",
    ActTrap = "SCMP_ACT_TRAP",
    ActErrno = "SCMP_ACT_ERRNO",
    ActTrace = "SCMP_ACT_TRACE",
    ActAllow = "SCMP_ACT_ALLOW",
    ActLog = "SCMP_ACT_LOG",
    ActNotify = "SCMP_ACT_NOTIFY",
  }
);

raw_values!(
  /// `LinuxSeccompOperator`.
  LinuxSeccompOperator {
    OpNotEqual = "SCMP_CMP_NE",
    OpLessThan = "SCMP_CMP_LT",
    OpLessEqual = "SCMP_CMP_LE",
    OpEqualTo = "SCMP_CMP_EQ",
    OpGreaterEqual = "SCMP_CMP_GE",
    OpGreaterThan = "SCMP_CMP_GT",
    OpMaskedEqual = "SCMP_CMP_MASKED_EQ",
  }
);

/// `LinuxSeccompArg`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LinuxSeccompArg {
  pub index: usize,
  pub value: u64,
  pub value_two: u64,
  pub op: LinuxSeccompOperator,
}

impl LinuxSeccompArg {
  /// `LinuxSeccompArg(index:value:valueTwo:op:)`.
  pub fn new(index: usize, value: u64, value_two: u64, op: LinuxSeccompOperator) -> Self {
    Self {
      index,
      value,
      value_two,
      op,
    }
  }
}

/// `LinuxSyscall`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LinuxSyscall {
  pub names: Vec<String>,
  pub action: LinuxSeccompAction,
  pub errno_ret: Option<usize>,
  pub args: Vec<LinuxSeccompArg>,
}

impl LinuxSyscall {
  /// `LinuxSyscall(names:action:errnoRet:args:)`.
  pub fn new(
    names: Vec<String>,
    action: LinuxSeccompAction,
    errno_ret: Option<usize>,
    args: Vec<LinuxSeccompArg>,
  ) -> Self {
    Self {
      names,
      action,
      errno_ret,
      args,
    }
  }
}
