//! The Swift functions Rust calls, and the Rust values Swift reads.
//!
//! swift-bridge's parser refuses a `cfg` on the bridge module, so the gate is
//! on `mod bridge` in `lib.rs`.
//!
//! Model types cross as opaque Rust types read through `accessors`, not shared
//! structs: swift-bridge can't put a `String`-holding struct in a `Vec`
//! (declined upstream, swift-bridge#305). They're `Rust`-prefixed to avoid
//! Containerization's `Mount`, `User`, etc., and passed as owned clones since
//! swift-bridge can't pass them to Swift by reference; Swift then reads them in
//! place. (No doc comments inside the module: swift-bridge can't parse them.)

mod accessors;

use crate::model::{
  BootLog as RustBootLog, BootSpec as RustBootSpec, BuildPlan as RustBuildPlan, BuildStep as RustBuildStep,
  Dns as RustDns, Hosts as RustHosts, HostsEntry as RustHostsEntry,
  LinuxContainerConfiguration as RustLinuxContainerConfiguration,
  LinuxProcessConfiguration as RustLinuxProcessConfiguration, Mount as RustMount, NatInterface as RustNatInterface,
  UnixSocketConfiguration as RustUnixSocketConfiguration, User as RustUser,
};

#[swift_bridge::bridge]
pub(crate) mod ffi {
  // `Mount.RuntimeOptions`, less its options. (`Any` is a Swift keyword.)
  enum RuntimeKind {
    Virtioblk,
    Virtiofs,
    Generic,
  }

  // `UnixSocketConfiguration.Direction`.
  enum SocketDirection {
    Into,
    OutOf,
  }

  // `SeccompProfile`, less a custom profile's JSON.
  enum SeccompMode {
    Unconfined,
    Default,
    Profile,
  }

  extern "Rust" {
    type RustMount;
    fn mount_type(self: &RustMount) -> &str;
    fn source(self: &RustMount) -> &str;
    fn destination(self: &RustMount) -> &str;
    fn options_len(self: &RustMount) -> usize;
    fn options_at(self: &RustMount, index: usize) -> &str;
    fn runtime_kind(self: &RustMount) -> RuntimeKind;
    fn runtime_options_len(self: &RustMount) -> usize;
    fn runtime_options_at(self: &RustMount, index: usize) -> &str;

    type RustUnixSocketConfiguration;
    fn source(self: &RustUnixSocketConfiguration) -> String;
    fn destination(self: &RustUnixSocketConfiguration) -> String;
    fn permissions(self: &RustUnixSocketConfiguration) -> Option<u32>;
    fn direction(self: &RustUnixSocketConfiguration) -> SocketDirection;

    type RustNatInterface;
    fn ipv4_address(self: &RustNatInterface) -> &str;
    fn ipv4_gateway(self: &RustNatInterface) -> Option<&str>;
    fn ipv6_address(self: &RustNatInterface) -> Option<&str>;
    fn ipv6_gateway(self: &RustNatInterface) -> Option<&str>;
    fn mac_address(self: &RustNatInterface) -> Option<&str>;
    fn mtu(self: &RustNatInterface) -> u32;

    type RustDns;
    fn nameservers_len(self: &RustDns) -> usize;
    fn nameservers_at(self: &RustDns, index: usize) -> &str;
    fn domain(self: &RustDns) -> Option<&str>;
    fn search_domains_len(self: &RustDns) -> usize;
    fn search_domains_at(self: &RustDns, index: usize) -> &str;
    fn options_len(self: &RustDns) -> usize;
    fn options_at(self: &RustDns, index: usize) -> &str;

    type RustHostsEntry;
    fn ip_address(self: &RustHostsEntry) -> &str;
    fn hostnames_len(self: &RustHostsEntry) -> usize;
    fn hostnames_at(self: &RustHostsEntry, index: usize) -> &str;
    fn comment(self: &RustHostsEntry) -> Option<&str>;

    type RustHosts;
    fn entries_len(self: &RustHosts) -> usize;
    fn entries_at(self: &RustHosts, index: usize) -> &RustHostsEntry;
    fn comment(self: &RustHosts) -> Option<&str>;

    type RustBootLog;
    fn path(self: &RustBootLog) -> String;
    fn append(self: &RustBootLog) -> bool;

    type RustUser;
    fn uid(self: &RustUser) -> u32;
    fn gid(self: &RustUser) -> u32;
    fn umask(self: &RustUser) -> Option<u32>;
    fn additional_gids_len(self: &RustUser) -> usize;
    fn additional_gids_at(self: &RustUser, index: usize) -> u32;
    fn username(self: &RustUser) -> &str;

    type RustLinuxProcessConfiguration;
    fn has_arguments(self: &RustLinuxProcessConfiguration) -> bool;
    fn arguments_len(self: &RustLinuxProcessConfiguration) -> usize;
    fn arguments_at(self: &RustLinuxProcessConfiguration, index: usize) -> &str;
    fn environment_variables_len(self: &RustLinuxProcessConfiguration) -> usize;
    fn environment_variables_at(self: &RustLinuxProcessConfiguration, index: usize) -> &str;
    fn working_directory(self: &RustLinuxProcessConfiguration) -> Option<&str>;
    fn has_user(self: &RustLinuxProcessConfiguration) -> bool;
    fn user(self: &RustLinuxProcessConfiguration) -> &RustUser;

    type RustLinuxContainerConfiguration;
    fn process(self: &RustLinuxContainerConfiguration) -> &RustLinuxProcessConfiguration;
    fn cpus(self: &RustLinuxContainerConfiguration) -> u32;
    fn memory_in_bytes(self: &RustLinuxContainerConfiguration) -> u64;
    fn hostname(self: &RustLinuxContainerConfiguration) -> Option<&str>;
    fn sysctl_len(self: &RustLinuxContainerConfiguration) -> usize;
    fn sysctl_key_at(self: &RustLinuxContainerConfiguration, index: usize) -> &str;
    fn sysctl_value_at(self: &RustLinuxContainerConfiguration, index: usize) -> &str;
    fn interfaces_len(self: &RustLinuxContainerConfiguration) -> usize;
    fn interfaces_at(self: &RustLinuxContainerConfiguration, index: usize) -> &RustNatInterface;
    fn sockets_len(self: &RustLinuxContainerConfiguration) -> usize;
    fn sockets_at(self: &RustLinuxContainerConfiguration, index: usize) -> &RustUnixSocketConfiguration;
    fn mounts_len(self: &RustLinuxContainerConfiguration) -> usize;
    fn mounts_at(self: &RustLinuxContainerConfiguration, index: usize) -> &RustMount;
    fn masked_paths_len(self: &RustLinuxContainerConfiguration) -> usize;
    fn masked_paths_at(self: &RustLinuxContainerConfiguration, index: usize) -> &str;
    fn readonly_paths_len(self: &RustLinuxContainerConfiguration) -> usize;
    fn readonly_paths_at(self: &RustLinuxContainerConfiguration, index: usize) -> &str;
    fn has_dns(self: &RustLinuxContainerConfiguration) -> bool;
    fn dns(self: &RustLinuxContainerConfiguration) -> &RustDns;
    fn has_hosts(self: &RustLinuxContainerConfiguration) -> bool;
    fn hosts(self: &RustLinuxContainerConfiguration) -> &RustHosts;
    fn virtualization(self: &RustLinuxContainerConfiguration) -> bool;
    fn has_boot_log(self: &RustLinuxContainerConfiguration) -> bool;
    fn boot_log(self: &RustLinuxContainerConfiguration) -> &RustBootLog;
    fn oci_runtime_path(self: &RustLinuxContainerConfiguration) -> Option<&str>;
    fn seccomp_mode(self: &RustLinuxContainerConfiguration) -> SeccompMode;
    fn seccomp_profile(self: &RustLinuxContainerConfiguration) -> Option<&str>;
    fn use_init(self: &RustLinuxContainerConfiguration) -> bool;

    type RustBootSpec;
    fn id(self: &RustBootSpec) -> &str;
    fn reference(self: &RustBootSpec) -> &str;
    fn rootfs_size_in_bytes(self: &RustBootSpec) -> u64;
    fn vm_cpus(self: &RustBootSpec) -> u32;
    fn vm_memory_in_bytes(self: &RustBootSpec) -> u64;
    fn configuration(self: &RustBootSpec) -> &RustLinuxContainerConfiguration;

    type RustBuildStep;
    fn name(self: &RustBuildStep) -> &str;
    fn script(self: &RustBuildStep) -> &str;
    fn user(self: &RustBuildStep) -> Option<&str>;
    fn cache_key(self: &RustBuildStep) -> &str;

    type RustBuildPlan;
    fn name(self: &RustBuildPlan) -> &str;
    fn base(self: &RustBuildPlan) -> &str;
    fn tag(self: &RustBuildPlan) -> &str;
    fn cpus(self: &RustBuildPlan) -> u32;
    fn memory_in_bytes(self: &RustBuildPlan) -> u64;
    fn vm_cpus(self: &RustBuildPlan) -> u32;
    fn vm_memory_in_bytes(self: &RustBuildPlan) -> u64;
    fn mounts_len(self: &RustBuildPlan) -> usize;
    fn mounts_at(self: &RustBuildPlan, index: usize) -> &RustMount;
    fn steps_len(self: &RustBuildPlan) -> usize;
    fn steps_at(self: &RustBuildPlan, index: usize) -> &RustBuildStep;
    fn environment_len(self: &RustBuildPlan) -> usize;
    fn environment_at(self: &RustBuildPlan, index: usize) -> &str;
    fn labels_len(self: &RustBuildPlan) -> usize;
    fn label_key_at(self: &RustBuildPlan, index: usize) -> &str;
    fn label_value_at(self: &RustBuildPlan, index: usize) -> &str;
    fn user(self: &RustBuildPlan) -> Option<&str>;
    fn working_directory(self: &RustBuildPlan) -> Option<String>;
    fn interface(self: &RustBuildPlan) -> &RustNatInterface;
    fn base_key(self: &RustBuildPlan) -> &str;
    fn rootfs_size_in_bytes(self: &RustBuildPlan) -> u64;
    fn cache_restore(self: &RustBuildPlan) -> bool;
    fn cache_keep(self: &RustBuildPlan) -> u64;
    fn cache_keep_for_seconds(self: &RustBuildPlan) -> u64;
    fn shell_len(self: &RustBuildPlan) -> usize;
    fn shell_at(self: &RustBuildPlan, index: usize) -> &str;
    fn keepalive_len(self: &RustBuildPlan) -> usize;
    fn keepalive_at(self: &RustBuildPlan, index: usize) -> &str;
    fn reclaim(self: &RustBuildPlan) -> bool;
  }

  extern "Swift" {
    fn czbridge_last_error() -> String;

    fn czbridge_boot(
      spec: RustBootSpec,
      store_root: &str,
      kernel_path: &str,
      initfs_reference: &str,
      initfs_path: &str,
    ) -> i32;

    // Each descriptor is `-1` when the caller leaves it unattached.
    fn czbridge_exec(
      name: &str,
      id: &str,
      configuration: RustLinuxProcessConfiguration,
      terminal: i32,
      stdin: i32,
      stdout: i32,
      stderr: i32,
    ) -> i32;

    fn czbridge_build(
      plan: RustBuildPlan,
      store_root: &str,
      kernel_path: &str,
      initfs_reference: &str,
      initfs_path: &str,
    ) -> i32;

    fn czbridge_provision(
      store_root: &str,
      kernel_path: &str,
      kernel_url: &str,
      kernel_in_archive: &str,
      initfs_reference: &str,
      initfs_path: &str,
    ) -> i32;

    fn czbridge_resize(id: &str, terminal: i32) -> i32;

    fn czbridge_is_running(name: &str) -> bool;

    fn czbridge_is_unpacked(store_root: &str, image_reference: &str) -> i32;
  }
}
