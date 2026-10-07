//! The OCI runtime spec, which Swift reads.

use crate::containerization_oci::runtime::Hook as RustHook;
use crate::containerization_oci::runtime::Hooks as RustHooks;
use crate::containerization_oci::runtime::Linux as RustLinux;
use crate::containerization_oci::runtime::LinuxBlockIO as RustLinuxBlockIO;
use crate::containerization_oci::runtime::LinuxCPU as RustLinuxCPU;
use crate::containerization_oci::runtime::LinuxCapabilities as RustOciLinuxCapabilities;
use crate::containerization_oci::runtime::LinuxDevice as RustLinuxDevice;
use crate::containerization_oci::runtime::LinuxDeviceCgroup as RustLinuxDeviceCgroup;
use crate::containerization_oci::runtime::LinuxIDMapping as RustLinuxIDMapping;
use crate::containerization_oci::runtime::LinuxMemory as RustLinuxMemory;
use crate::containerization_oci::runtime::LinuxResources as RustLinuxResources;
use crate::containerization_oci::runtime::LinuxSeccomp as RustLinuxSeccomp;
use crate::containerization_oci::runtime::LinuxSyscall as RustLinuxSyscall;
use crate::containerization_oci::runtime::Mount as RustOciMount;
use crate::containerization_oci::runtime::Process as RustProcess;
use crate::containerization_oci::runtime::Spec as RustSpec;
use crate::containerization_oci::runtime::User as RustUser;

use super::ffi::CapabilitySet;
use super::ffi::HookKind;
use super::ffi::ThrottleKind;

#[swift_bridge::bridge]
mod ffi {
  #[swift_bridge(already_declared)]
  enum CapabilitySet {}

  #[swift_bridge(already_declared)]
  enum HookKind {}

  #[swift_bridge(already_declared)]
  enum ThrottleKind {}

  extern "Rust" {
    #[swift_bridge(already_declared)]
    type RustUser;

    // The OCI runtime spec. An enum is its `rawValue`.
    type RustSpec;
    fn version(self: &RustSpec) -> &str;
    #[swift_bridge(swift_name = "hasHooks")]
    fn has_hooks(self: &RustSpec) -> bool;
    fn hooks(self: &RustSpec) -> &RustHooks;
    #[swift_bridge(swift_name = "hasProcess")]
    fn has_process(self: &RustSpec) -> bool;
    fn process(self: &RustSpec) -> &RustProcess;
    fn hostname(self: &RustSpec) -> &str;
    fn domainname(self: &RustSpec) -> &str;
    #[swift_bridge(swift_name = "mountsLen")]
    fn mounts_len(self: &RustSpec) -> usize;
    #[swift_bridge(swift_name = "mountsAt")]
    fn mounts_at(self: &RustSpec, index: usize) -> &RustOciMount;
    #[swift_bridge(swift_name = "hasAnnotations")]
    fn has_annotations(self: &RustSpec) -> bool;
    #[swift_bridge(swift_name = "annotationsLen")]
    fn annotations_len(self: &RustSpec) -> usize;
    #[swift_bridge(swift_name = "annotationKeyAt")]
    fn annotation_key_at(self: &RustSpec, index: usize) -> &str;
    #[swift_bridge(swift_name = "annotationValueAt")]
    fn annotation_value_at(self: &RustSpec, index: usize) -> &str;
    #[swift_bridge(swift_name = "hasRoot")]
    fn has_root(self: &RustSpec) -> bool;
    #[swift_bridge(swift_name = "rootPath")]
    fn root_path(self: &RustSpec) -> &str;
    #[swift_bridge(swift_name = "rootReadonly")]
    fn root_readonly(self: &RustSpec) -> bool;
    #[swift_bridge(swift_name = "hasLinux")]
    fn has_linux(self: &RustSpec) -> bool;
    fn linux(self: &RustSpec) -> &RustLinux;

    type RustProcess;
    fn cwd(self: &RustProcess) -> &str;
    #[swift_bridge(swift_name = "envLen")]
    fn env_len(self: &RustProcess) -> usize;
    #[swift_bridge(swift_name = "envAt")]
    fn env_at(self: &RustProcess, index: usize) -> &str;
    #[swift_bridge(swift_name = "hasConsoleSize")]
    fn has_console_size(self: &RustProcess) -> bool;
    #[swift_bridge(swift_name = "consoleHeight")]
    fn console_height(self: &RustProcess) -> usize;
    #[swift_bridge(swift_name = "consoleWidth")]
    fn console_width(self: &RustProcess) -> usize;
    #[swift_bridge(swift_name = "selinuxLabel")]
    fn selinux_label(self: &RustProcess) -> &str;
    #[swift_bridge(swift_name = "noNewPrivileges")]
    fn no_new_privileges(self: &RustProcess) -> bool;
    #[swift_bridge(swift_name = "commandLine")]
    fn command_line(self: &RustProcess) -> &str;
    #[swift_bridge(swift_name = "oomScoreAdj")]
    fn oom_score_adj(self: &RustProcess) -> Option<isize>;
    #[swift_bridge(swift_name = "hasCapabilities")]
    fn has_capabilities(self: &RustProcess) -> bool;
    fn capabilities(self: &RustProcess) -> &RustOciLinuxCapabilities;
    #[swift_bridge(swift_name = "apparmorProfile")]
    fn apparmor_profile(self: &RustProcess) -> &str;
    fn user(self: &RustProcess) -> &RustUser;
    #[swift_bridge(swift_name = "rlimitsLen")]
    fn rlimits_len(self: &RustProcess) -> usize;
    #[swift_bridge(swift_name = "rlimitTypeAt")]
    fn rlimit_type_at(self: &RustProcess, index: usize) -> &str;
    #[swift_bridge(swift_name = "rlimitHardAt")]
    fn rlimit_hard_at(self: &RustProcess, index: usize) -> u64;
    #[swift_bridge(swift_name = "rlimitSoftAt")]
    fn rlimit_soft_at(self: &RustProcess, index: usize) -> u64;
    #[swift_bridge(swift_name = "argsLen")]
    fn args_len(self: &RustProcess) -> usize;
    #[swift_bridge(swift_name = "argsAt")]
    fn args_at(self: &RustProcess, index: usize) -> &str;
    fn terminal(self: &RustProcess) -> bool;

    type RustOciLinuxCapabilities;
    #[swift_bridge(swift_name = "hasSet")]
    fn has_set(self: &RustOciLinuxCapabilities, set: CapabilitySet) -> bool;
    #[swift_bridge(swift_name = "setLen")]
    fn set_len(self: &RustOciLinuxCapabilities, set: CapabilitySet) -> usize;
    #[swift_bridge(swift_name = "setAt")]
    fn set_at(self: &RustOciLinuxCapabilities, set: CapabilitySet, index: usize) -> &str;

    type RustOciMount;
    #[swift_bridge(swift_name = "mountType")]
    fn mount_type(self: &RustOciMount) -> &str;
    fn source(self: &RustOciMount) -> &str;
    fn destination(self: &RustOciMount) -> &str;
    #[swift_bridge(swift_name = "optionsLen")]
    fn options_len(self: &RustOciMount) -> usize;
    #[swift_bridge(swift_name = "optionsAt")]
    fn options_at(self: &RustOciMount, index: usize) -> &str;
    #[swift_bridge(swift_name = "hasUidMappings")]
    fn has_uid_mappings(self: &RustOciMount) -> bool;
    #[swift_bridge(swift_name = "uidMappingsLen")]
    fn uid_mappings_len(self: &RustOciMount) -> usize;
    #[swift_bridge(swift_name = "uidMappingsAt")]
    fn uid_mappings_at(self: &RustOciMount, index: usize) -> &RustLinuxIDMapping;
    #[swift_bridge(swift_name = "hasGidMappings")]
    fn has_gid_mappings(self: &RustOciMount) -> bool;
    #[swift_bridge(swift_name = "gidMappingsLen")]
    fn gid_mappings_len(self: &RustOciMount) -> usize;
    #[swift_bridge(swift_name = "gidMappingsAt")]
    fn gid_mappings_at(self: &RustOciMount, index: usize) -> &RustLinuxIDMapping;

    type RustLinuxIDMapping;
    #[swift_bridge(swift_name = "containerID")]
    fn container_id(self: &RustLinuxIDMapping) -> u32;
    #[swift_bridge(swift_name = "hostID")]
    fn host_id(self: &RustLinuxIDMapping) -> u32;
    fn size(self: &RustLinuxIDMapping) -> u32;

    type RustHook;
    fn path(self: &RustHook) -> &str;
    #[swift_bridge(swift_name = "argsLen")]
    fn args_len(self: &RustHook) -> usize;
    #[swift_bridge(swift_name = "argsAt")]
    fn args_at(self: &RustHook, index: usize) -> &str;
    #[swift_bridge(swift_name = "envLen")]
    fn env_len(self: &RustHook) -> usize;
    #[swift_bridge(swift_name = "envAt")]
    fn env_at(self: &RustHook, index: usize) -> &str;
    fn timeout(self: &RustHook) -> Option<isize>;

    type RustHooks;
    #[swift_bridge(swift_name = "hooksLen")]
    fn hooks_len(self: &RustHooks, kind: HookKind) -> usize;
    #[swift_bridge(swift_name = "hooksAt")]
    fn hooks_at(self: &RustHooks, kind: HookKind, index: usize) -> &RustHook;

    type RustLinux;
    #[swift_bridge(swift_name = "uidMappingsLen")]
    fn uid_mappings_len(self: &RustLinux) -> usize;
    #[swift_bridge(swift_name = "uidMappingsAt")]
    fn uid_mappings_at(self: &RustLinux, index: usize) -> &RustLinuxIDMapping;
    #[swift_bridge(swift_name = "gidMappingsLen")]
    fn gid_mappings_len(self: &RustLinux) -> usize;
    #[swift_bridge(swift_name = "gidMappingsAt")]
    fn gid_mappings_at(self: &RustLinux, index: usize) -> &RustLinuxIDMapping;
    #[swift_bridge(swift_name = "hasSysctl")]
    fn has_sysctl(self: &RustLinux) -> bool;
    #[swift_bridge(swift_name = "sysctlLen")]
    fn sysctl_len(self: &RustLinux) -> usize;
    #[swift_bridge(swift_name = "sysctlKeyAt")]
    fn sysctl_key_at(self: &RustLinux, index: usize) -> &str;
    #[swift_bridge(swift_name = "sysctlValueAt")]
    fn sysctl_value_at(self: &RustLinux, index: usize) -> &str;
    #[swift_bridge(swift_name = "hasResources")]
    fn has_resources(self: &RustLinux) -> bool;
    fn resources(self: &RustLinux) -> &RustLinuxResources;
    #[swift_bridge(swift_name = "cgroupsPath")]
    fn cgroups_path(self: &RustLinux) -> &str;
    #[swift_bridge(swift_name = "namespacesLen")]
    fn namespaces_len(self: &RustLinux) -> usize;
    #[swift_bridge(swift_name = "namespaceTypeAt")]
    fn namespace_type_at(self: &RustLinux, index: usize) -> &str;
    #[swift_bridge(swift_name = "namespacePathAt")]
    fn namespace_path_at(self: &RustLinux, index: usize) -> &str;
    #[swift_bridge(swift_name = "devicesLen")]
    fn devices_len(self: &RustLinux) -> usize;
    #[swift_bridge(swift_name = "devicesAt")]
    fn devices_at(self: &RustLinux, index: usize) -> &RustLinuxDevice;
    #[swift_bridge(swift_name = "hasSeccomp")]
    fn has_seccomp(self: &RustLinux) -> bool;
    fn seccomp(self: &RustLinux) -> &RustLinuxSeccomp;
    #[swift_bridge(swift_name = "rootfsPropagation")]
    fn rootfs_propagation(self: &RustLinux) -> &str;
    #[swift_bridge(swift_name = "maskedPathsLen")]
    fn masked_paths_len(self: &RustLinux) -> usize;
    #[swift_bridge(swift_name = "maskedPathsAt")]
    fn masked_paths_at(self: &RustLinux, index: usize) -> &str;
    #[swift_bridge(swift_name = "readonlyPathsLen")]
    fn readonly_paths_len(self: &RustLinux) -> usize;
    #[swift_bridge(swift_name = "readonlyPathsAt")]
    fn readonly_paths_at(self: &RustLinux, index: usize) -> &str;
    #[swift_bridge(swift_name = "mountLabel")]
    fn mount_label(self: &RustLinux) -> &str;
    #[swift_bridge(swift_name = "hasPersonality")]
    fn has_personality(self: &RustLinux) -> bool;
    #[swift_bridge(swift_name = "personalityDomain")]
    fn personality_domain(self: &RustLinux) -> &str;
    #[swift_bridge(swift_name = "personalityFlagsLen")]
    fn personality_flags_len(self: &RustLinux) -> usize;
    #[swift_bridge(swift_name = "personalityFlagsAt")]
    fn personality_flags_at(self: &RustLinux, index: usize) -> &str;

    type RustLinuxResources;
    #[swift_bridge(swift_name = "devicesLen")]
    fn devices_len(self: &RustLinuxResources) -> usize;
    #[swift_bridge(swift_name = "devicesAt")]
    fn devices_at(self: &RustLinuxResources, index: usize) -> &RustLinuxDeviceCgroup;
    #[swift_bridge(swift_name = "hasMemory")]
    fn has_memory(self: &RustLinuxResources) -> bool;
    fn memory(self: &RustLinuxResources) -> &RustLinuxMemory;
    #[swift_bridge(swift_name = "hasCpu")]
    fn has_cpu(self: &RustLinuxResources) -> bool;
    fn cpu(self: &RustLinuxResources) -> &RustLinuxCPU;
    #[swift_bridge(swift_name = "pidsLimit")]
    fn pids_limit(self: &RustLinuxResources) -> Option<i64>;
    #[swift_bridge(swift_name = "hasBlockIO")]
    fn has_block_io(self: &RustLinuxResources) -> bool;
    #[swift_bridge(swift_name = "blockIO")]
    fn block_io(self: &RustLinuxResources) -> &RustLinuxBlockIO;
    #[swift_bridge(swift_name = "hugepageLimitsLen")]
    fn hugepage_limits_len(self: &RustLinuxResources) -> usize;
    #[swift_bridge(swift_name = "hugepageLimitPagesizeAt")]
    fn hugepage_limit_pagesize_at(self: &RustLinuxResources, index: usize) -> &str;
    #[swift_bridge(swift_name = "hugepageLimitLimitAt")]
    fn hugepage_limit_limit_at(self: &RustLinuxResources, index: usize) -> u64;
    #[swift_bridge(swift_name = "hasNetwork")]
    fn has_network(self: &RustLinuxResources) -> bool;
    #[swift_bridge(swift_name = "networkClassID")]
    fn network_class_id(self: &RustLinuxResources) -> Option<u32>;
    #[swift_bridge(swift_name = "networkPrioritiesLen")]
    fn network_priorities_len(self: &RustLinuxResources) -> usize;
    #[swift_bridge(swift_name = "networkPriorityNameAt")]
    fn network_priority_name_at(self: &RustLinuxResources, index: usize) -> &str;
    #[swift_bridge(swift_name = "networkPriorityAt")]
    fn network_priority_at(self: &RustLinuxResources, index: usize) -> u32;
    #[swift_bridge(swift_name = "hasRdma")]
    fn has_rdma(self: &RustLinuxResources) -> bool;
    #[swift_bridge(swift_name = "rdmaLen")]
    fn rdma_len(self: &RustLinuxResources) -> usize;
    #[swift_bridge(swift_name = "rdmaKeyAt")]
    fn rdma_key_at(self: &RustLinuxResources, index: usize) -> &str;
    #[swift_bridge(swift_name = "rdmaHcsHandlesAt")]
    fn rdma_hcs_handles_at(self: &RustLinuxResources, index: usize) -> Option<u32>;
    #[swift_bridge(swift_name = "rdmaHcaObjectsAt")]
    fn rdma_hca_objects_at(self: &RustLinuxResources, index: usize) -> Option<u32>;
    #[swift_bridge(swift_name = "hasUnified")]
    fn has_unified(self: &RustLinuxResources) -> bool;
    #[swift_bridge(swift_name = "unifiedLen")]
    fn unified_len(self: &RustLinuxResources) -> usize;
    #[swift_bridge(swift_name = "unifiedKeyAt")]
    fn unified_key_at(self: &RustLinuxResources, index: usize) -> &str;
    #[swift_bridge(swift_name = "unifiedValueAt")]
    fn unified_value_at(self: &RustLinuxResources, index: usize) -> &str;

    type RustLinuxMemory;
    fn limit(self: &RustLinuxMemory) -> Option<i64>;
    fn reservation(self: &RustLinuxMemory) -> Option<i64>;
    fn swap(self: &RustLinuxMemory) -> Option<i64>;
    fn kernel(self: &RustLinuxMemory) -> Option<i64>;
    #[swift_bridge(swift_name = "kernelTCP")]
    fn kernel_tcp(self: &RustLinuxMemory) -> Option<i64>;
    fn swappiness(self: &RustLinuxMemory) -> Option<u64>;
    #[swift_bridge(swift_name = "disableOOMKiller")]
    fn disable_oom_killer(self: &RustLinuxMemory) -> Option<bool>;
    #[swift_bridge(swift_name = "useHierarchy")]
    fn use_hierarchy(self: &RustLinuxMemory) -> Option<bool>;
    #[swift_bridge(swift_name = "checkBeforeUpdate")]
    fn check_before_update(self: &RustLinuxMemory) -> Option<bool>;

    type RustLinuxCPU;
    fn shares(self: &RustLinuxCPU) -> Option<u64>;
    fn quota(self: &RustLinuxCPU) -> Option<i64>;
    fn burst(self: &RustLinuxCPU) -> Option<u64>;
    fn period(self: &RustLinuxCPU) -> Option<u64>;
    #[swift_bridge(swift_name = "realtimeRuntime")]
    fn realtime_runtime(self: &RustLinuxCPU) -> Option<i64>;
    #[swift_bridge(swift_name = "realtimePeriod")]
    fn realtime_period(self: &RustLinuxCPU) -> Option<i64>;
    fn cpus(self: &RustLinuxCPU) -> &str;
    fn mems(self: &RustLinuxCPU) -> &str;
    fn idle(self: &RustLinuxCPU) -> Option<i64>;

    type RustLinuxBlockIO;
    fn weight(self: &RustLinuxBlockIO) -> Option<u16>;
    #[swift_bridge(swift_name = "leafWeight")]
    fn leaf_weight(self: &RustLinuxBlockIO) -> Option<u16>;
    #[swift_bridge(swift_name = "weightDeviceLen")]
    fn weight_device_len(self: &RustLinuxBlockIO) -> usize;
    #[swift_bridge(swift_name = "weightDeviceMajorAt")]
    fn weight_device_major_at(self: &RustLinuxBlockIO, index: usize) -> i64;
    #[swift_bridge(swift_name = "weightDeviceMinorAt")]
    fn weight_device_minor_at(self: &RustLinuxBlockIO, index: usize) -> i64;
    #[swift_bridge(swift_name = "weightDeviceWeightAt")]
    fn weight_device_weight_at(self: &RustLinuxBlockIO, index: usize) -> Option<u16>;
    #[swift_bridge(swift_name = "weightDeviceLeafWeightAt")]
    fn weight_device_leaf_weight_at(self: &RustLinuxBlockIO, index: usize) -> Option<u16>;
    #[swift_bridge(swift_name = "throttleLen")]
    fn throttle_len(self: &RustLinuxBlockIO, kind: ThrottleKind) -> usize;
    #[swift_bridge(swift_name = "throttleMajorAt")]
    fn throttle_major_at(self: &RustLinuxBlockIO, kind: ThrottleKind, index: usize) -> i64;
    #[swift_bridge(swift_name = "throttleMinorAt")]
    fn throttle_minor_at(self: &RustLinuxBlockIO, kind: ThrottleKind, index: usize) -> i64;
    #[swift_bridge(swift_name = "throttleRateAt")]
    fn throttle_rate_at(self: &RustLinuxBlockIO, kind: ThrottleKind, index: usize) -> u64;

    type RustLinuxDevice;
    fn path(self: &RustLinuxDevice) -> &str;
    #[swift_bridge(swift_name = "deviceType")]
    fn device_type(self: &RustLinuxDevice) -> &str;
    fn major(self: &RustLinuxDevice) -> i64;
    fn minor(self: &RustLinuxDevice) -> i64;
    #[swift_bridge(swift_name = "fileMode")]
    fn file_mode(self: &RustLinuxDevice) -> Option<u32>;
    fn uid(self: &RustLinuxDevice) -> Option<u32>;
    fn gid(self: &RustLinuxDevice) -> Option<u32>;

    type RustLinuxDeviceCgroup;
    fn allow(self: &RustLinuxDeviceCgroup) -> bool;
    #[swift_bridge(swift_name = "deviceType")]
    fn device_type(self: &RustLinuxDeviceCgroup) -> &str;
    fn major(self: &RustLinuxDeviceCgroup) -> Option<i64>;
    fn minor(self: &RustLinuxDeviceCgroup) -> Option<i64>;
    fn access(self: &RustLinuxDeviceCgroup) -> Option<&str>;

    type RustLinuxSeccomp;
    #[swift_bridge(swift_name = "defaultAction")]
    fn default_action(self: &RustLinuxSeccomp) -> &str;
    #[swift_bridge(swift_name = "defaultErrnoRet")]
    fn default_errno_ret(self: &RustLinuxSeccomp) -> Option<usize>;
    #[swift_bridge(swift_name = "architecturesLen")]
    fn architectures_len(self: &RustLinuxSeccomp) -> usize;
    #[swift_bridge(swift_name = "architecturesAt")]
    fn architectures_at(self: &RustLinuxSeccomp, index: usize) -> &str;
    #[swift_bridge(swift_name = "flagsLen")]
    fn flags_len(self: &RustLinuxSeccomp) -> usize;
    #[swift_bridge(swift_name = "flagsAt")]
    fn flags_at(self: &RustLinuxSeccomp, index: usize) -> &str;
    #[swift_bridge(swift_name = "listenerPath")]
    fn listener_path(self: &RustLinuxSeccomp) -> &str;
    #[swift_bridge(swift_name = "listenerMetadata")]
    fn listener_metadata(self: &RustLinuxSeccomp) -> &str;
    #[swift_bridge(swift_name = "syscallsLen")]
    fn syscalls_len(self: &RustLinuxSeccomp) -> usize;
    #[swift_bridge(swift_name = "syscallsAt")]
    fn syscalls_at(self: &RustLinuxSeccomp, index: usize) -> &RustLinuxSyscall;

    type RustLinuxSyscall;
    #[swift_bridge(swift_name = "namesLen")]
    fn names_len(self: &RustLinuxSyscall) -> usize;
    #[swift_bridge(swift_name = "namesAt")]
    fn names_at(self: &RustLinuxSyscall, index: usize) -> &str;
    fn action(self: &RustLinuxSyscall) -> &str;
    #[swift_bridge(swift_name = "errnoRet")]
    fn errno_ret(self: &RustLinuxSyscall) -> Option<usize>;
    #[swift_bridge(swift_name = "argsLen")]
    fn args_len(self: &RustLinuxSyscall) -> usize;
    #[swift_bridge(swift_name = "argIndexAt")]
    fn arg_index_at(self: &RustLinuxSyscall, index: usize) -> usize;
    #[swift_bridge(swift_name = "argValueAt")]
    fn arg_value_at(self: &RustLinuxSyscall, index: usize) -> u64;
    #[swift_bridge(swift_name = "argValueTwoAt")]
    fn arg_value_two_at(self: &RustLinuxSyscall, index: usize) -> u64;
    #[swift_bridge(swift_name = "argOpAt")]
    fn arg_op_at(self: &RustLinuxSyscall, index: usize) -> &str;
  }
}
