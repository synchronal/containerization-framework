//! The values a VM or pod is configured with, which Swift reads.

use crate::containerization::container::Mount as RustMount;
use crate::containerization::container::UnixSocketConfiguration as RustUnixSocketConfiguration;
use crate::containerization::container::linux_pod::ContainerConfiguration as RustPodContainerConfiguration;
use crate::containerization::container::linux_pod::PodVolume as RustPodVolume;
use crate::containerization::network::Dns as RustDns;
use crate::containerization::network::Hosts as RustHosts;
use crate::containerization::process::LinuxProcessConfiguration as RustLinuxProcessConfiguration;
use crate::containerization::vm::AttachedFilesystem as RustAttachedFilesystem;
use crate::containerization::vm::BootLog as RustBootLog;
use crate::containerization::vm::Kernel as RustKernel;
use crate::containerization::vm::SystemPlatform as RustSystemPlatform;
use crate::containerization_oci::runtime::LinuxSeccomp as RustLinuxSeccomp;

use super::ffi::BootLogKind;
use super::ffi::PlatformArchitecture;
use super::ffi::PlatformOs;
use super::ffi::PodVolumeKind;
use super::ffi::SeccompMode;

#[swift_bridge::bridge]
mod ffi {
  #[swift_bridge(already_declared)]
  enum SeccompMode {}

  #[swift_bridge(already_declared)]
  enum BootLogKind {}

  #[swift_bridge(already_declared)]
  enum PlatformOs {}

  #[swift_bridge(already_declared)]
  enum PlatformArchitecture {}

  #[swift_bridge(already_declared)]
  enum PodVolumeKind {}

  extern "Rust" {
    #[swift_bridge(already_declared)]
    type RustMount;
    #[swift_bridge(already_declared)]
    type RustUnixSocketConfiguration;
    #[swift_bridge(already_declared)]
    type RustLinuxProcessConfiguration;
    #[swift_bridge(already_declared)]
    type RustDns;
    #[swift_bridge(already_declared)]
    type RustHosts;
    #[swift_bridge(already_declared)]
    type RustLinuxSeccomp;

    type RustBootLog;
    fn kind(self: &RustBootLog) -> BootLogKind;
    fn path(self: &RustBootLog) -> String;
    fn append(self: &RustBootLog) -> bool;
    #[swift_bridge(swift_name = "fileHandle")]
    fn file_handle(self: &RustBootLog) -> i32;

    // `location` is an `nbd` source's URL or a `diskImage`'s path.
    type RustPodVolume;
    fn name(self: &RustPodVolume) -> &str;
    fn format(self: &RustPodVolume) -> &str;
    #[swift_bridge(swift_name = "sourceKind")]
    fn source_kind(self: &RustPodVolume) -> PodVolumeKind;
    fn location(self: &RustPodVolume) -> String;
    fn timeout(self: &RustPodVolume) -> Option<f64>;
    #[swift_bridge(swift_name = "readOnly")]
    fn read_only(self: &RustPodVolume) -> bool;
    #[swift_bridge(swift_name = "sizeBytes")]
    fn size_bytes(self: &RustPodVolume) -> Option<u64>;

    // Read as `RustLinuxContainerConfiguration` is. Its seccomp profile is
    // `nil` unless `has_seccomp_profile`.
    type RustPodContainerConfiguration;
    fn process(self: &RustPodContainerConfiguration) -> &RustLinuxProcessConfiguration;
    fn cpus(self: &RustPodContainerConfiguration) -> u32;
    #[swift_bridge(swift_name = "memoryInBytes")]
    fn memory_in_bytes(self: &RustPodContainerConfiguration) -> u64;
    fn hostname(self: &RustPodContainerConfiguration) -> Option<&str>;
    #[swift_bridge(swift_name = "sysctlLen")]
    fn sysctl_len(self: &RustPodContainerConfiguration) -> usize;
    #[swift_bridge(swift_name = "sysctlKeyAt")]
    fn sysctl_key_at(self: &RustPodContainerConfiguration, index: usize) -> &str;
    #[swift_bridge(swift_name = "sysctlValueAt")]
    fn sysctl_value_at(self: &RustPodContainerConfiguration, index: usize) -> &str;
    #[swift_bridge(swift_name = "mountsLen")]
    fn mounts_len(self: &RustPodContainerConfiguration) -> usize;
    #[swift_bridge(swift_name = "mountsAt")]
    fn mounts_at(self: &RustPodContainerConfiguration, index: usize) -> &RustMount;
    #[swift_bridge(swift_name = "maskedPathsLen")]
    fn masked_paths_len(self: &RustPodContainerConfiguration) -> usize;
    #[swift_bridge(swift_name = "maskedPathsAt")]
    fn masked_paths_at(self: &RustPodContainerConfiguration, index: usize) -> &str;
    #[swift_bridge(swift_name = "readonlyPathsLen")]
    fn readonly_paths_len(self: &RustPodContainerConfiguration) -> usize;
    #[swift_bridge(swift_name = "readonlyPathsAt")]
    fn readonly_paths_at(self: &RustPodContainerConfiguration, index: usize) -> &str;
    #[swift_bridge(swift_name = "socketsLen")]
    fn sockets_len(self: &RustPodContainerConfiguration) -> usize;
    #[swift_bridge(swift_name = "socketsAt")]
    fn sockets_at(self: &RustPodContainerConfiguration, index: usize) -> &RustUnixSocketConfiguration;
    #[swift_bridge(swift_name = "hasDns")]
    fn has_dns(self: &RustPodContainerConfiguration) -> bool;
    fn dns(self: &RustPodContainerConfiguration) -> &RustDns;
    #[swift_bridge(swift_name = "hasHosts")]
    fn has_hosts(self: &RustPodContainerConfiguration) -> bool;
    fn hosts(self: &RustPodContainerConfiguration) -> &RustHosts;
    #[swift_bridge(swift_name = "hasSeccompProfile")]
    fn has_seccomp_profile(self: &RustPodContainerConfiguration) -> bool;
    #[swift_bridge(swift_name = "seccompMode")]
    fn seccomp_mode(self: &RustPodContainerConfiguration) -> SeccompMode;
    #[swift_bridge(swift_name = "seccompProfile")]
    fn seccomp_profile(self: &RustPodContainerConfiguration) -> &RustLinuxSeccomp;
    #[swift_bridge(swift_name = "useInit")]
    fn use_init(self: &RustPodContainerConfiguration) -> bool;

    type RustAttachedFilesystem;
    #[swift_bridge(swift_name = "filesystemType")]
    fn filesystem_type(self: &RustAttachedFilesystem) -> &str;
    fn source(self: &RustAttachedFilesystem) -> &str;
    fn destination(self: &RustAttachedFilesystem) -> &str;
    #[swift_bridge(swift_name = "optionsLen")]
    fn options_len(self: &RustAttachedFilesystem) -> usize;
    #[swift_bridge(swift_name = "optionsAt")]
    fn options_at(self: &RustAttachedFilesystem, index: usize) -> &str;

    type RustSystemPlatform;
    fn os(self: &RustSystemPlatform) -> PlatformOs;
    fn architecture(self: &RustSystemPlatform) -> PlatformArchitecture;

    type RustKernel;
    fn path(self: &RustKernel) -> String;
    fn platform(self: &RustKernel) -> &RustSystemPlatform;
    #[swift_bridge(swift_name = "kernelArgsLen")]
    fn kernel_args_len(self: &RustKernel) -> usize;
    #[swift_bridge(swift_name = "kernelArgsAt")]
    fn kernel_args_at(self: &RustKernel, index: usize) -> &str;
    #[swift_bridge(swift_name = "initArgsLen")]
    fn init_args_len(self: &RustKernel) -> usize;
    #[swift_bridge(swift_name = "initArgsAt")]
    fn init_args_at(self: &RustKernel, index: usize) -> &str;
  }
}
