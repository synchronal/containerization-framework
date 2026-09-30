//! The Swift functions Rust calls, and the Rust values Swift reads.
//!
//! swift-bridge's parser refuses a `cfg` on the bridge module, so the gate is
//! on `mod bridge` in `lib.rs`.
//!
//! Model types cross as opaque Rust types read through `accessors`, not shared
//! structs: swift-bridge can't put a `String`-holding struct in a `Vec`
//! (declined upstream, swift-bridge#305). They're `Rust`-prefixed to avoid
//! Containerization's `Mount`, `User`, etc., and passed as owned clones since
//! swift-bridge can't pass them to Swift by reference. (No doc comments inside
//! the module: swift-bridge can't parse them.)

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
    fn options(self: &RustMount) -> Vec<String>;
    fn runtime_kind(self: &RustMount) -> RuntimeKind;
    fn runtime_options(self: &RustMount) -> Vec<String>;

    type RustUnixSocketConfiguration;
    fn source(self: &RustUnixSocketConfiguration) -> String;
    fn destination(self: &RustUnixSocketConfiguration) -> String;
    fn permissions(self: &RustUnixSocketConfiguration) -> Option<u32>;
    fn direction(self: &RustUnixSocketConfiguration) -> SocketDirection;

    type RustNatInterface;
    fn ipv4_address(self: &RustNatInterface) -> &str;
    fn ipv4_gateway(self: &RustNatInterface) -> Option<String>;
    fn ipv6_address(self: &RustNatInterface) -> Option<String>;
    fn ipv6_gateway(self: &RustNatInterface) -> Option<String>;
    fn mac_address(self: &RustNatInterface) -> Option<String>;
    fn mtu(self: &RustNatInterface) -> u32;

    type RustDns;
    fn nameservers(self: &RustDns) -> Vec<String>;
    fn domain(self: &RustDns) -> Option<String>;
    fn search_domains(self: &RustDns) -> Vec<String>;
    fn options(self: &RustDns) -> Vec<String>;

    type RustHostsEntry;
    fn ip_address(self: &RustHostsEntry) -> &str;
    fn hostnames(self: &RustHostsEntry) -> Vec<String>;
    fn comment(self: &RustHostsEntry) -> Option<String>;

    type RustHosts;
    fn entries(self: &RustHosts) -> Vec<RustHostsEntry>;
    fn comment(self: &RustHosts) -> Option<String>;

    type RustBootLog;
    fn path(self: &RustBootLog) -> String;
    fn append(self: &RustBootLog) -> bool;

    type RustUser;
    fn uid(self: &RustUser) -> u32;
    fn gid(self: &RustUser) -> u32;
    fn umask(self: &RustUser) -> Option<u32>;
    fn additional_gids(self: &RustUser) -> Vec<u32>;
    fn username(self: &RustUser) -> &str;

    type RustLinuxProcessConfiguration;
    fn has_arguments(self: &RustLinuxProcessConfiguration) -> bool;
    fn arguments(self: &RustLinuxProcessConfiguration) -> Vec<String>;
    fn environment_variables(self: &RustLinuxProcessConfiguration) -> Vec<String>;
    fn working_directory(self: &RustLinuxProcessConfiguration) -> Option<String>;
    fn has_user(self: &RustLinuxProcessConfiguration) -> bool;
    fn user(self: &RustLinuxProcessConfiguration) -> &RustUser;

    type RustLinuxContainerConfiguration;
    fn process(self: &RustLinuxContainerConfiguration) -> &RustLinuxProcessConfiguration;
    fn cpus(self: &RustLinuxContainerConfiguration) -> u32;
    fn memory_in_bytes(self: &RustLinuxContainerConfiguration) -> u64;
    fn hostname(self: &RustLinuxContainerConfiguration) -> Option<String>;
    fn sysctl_keys(self: &RustLinuxContainerConfiguration) -> Vec<String>;
    fn sysctl(self: &RustLinuxContainerConfiguration, key: &str) -> Option<String>;
    fn interfaces(self: &RustLinuxContainerConfiguration) -> Vec<RustNatInterface>;
    fn sockets(self: &RustLinuxContainerConfiguration) -> Vec<RustUnixSocketConfiguration>;
    fn mounts(self: &RustLinuxContainerConfiguration) -> Vec<RustMount>;
    fn masked_paths(self: &RustLinuxContainerConfiguration) -> Vec<String>;
    fn readonly_paths(self: &RustLinuxContainerConfiguration) -> Vec<String>;
    fn has_dns(self: &RustLinuxContainerConfiguration) -> bool;
    fn dns(self: &RustLinuxContainerConfiguration) -> &RustDns;
    fn has_hosts(self: &RustLinuxContainerConfiguration) -> bool;
    fn hosts(self: &RustLinuxContainerConfiguration) -> &RustHosts;
    fn virtualization(self: &RustLinuxContainerConfiguration) -> bool;
    fn has_boot_log(self: &RustLinuxContainerConfiguration) -> bool;
    fn boot_log(self: &RustLinuxContainerConfiguration) -> &RustBootLog;
    fn oci_runtime_path(self: &RustLinuxContainerConfiguration) -> Option<String>;
    fn seccomp_mode(self: &RustLinuxContainerConfiguration) -> SeccompMode;
    fn seccomp_profile(self: &RustLinuxContainerConfiguration) -> Option<String>;
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
    fn user(self: &RustBuildStep) -> Option<String>;
    fn cache_key(self: &RustBuildStep) -> &str;

    type RustBuildPlan;
    fn name(self: &RustBuildPlan) -> &str;
    fn base(self: &RustBuildPlan) -> &str;
    fn tag(self: &RustBuildPlan) -> &str;
    fn cpus(self: &RustBuildPlan) -> u32;
    fn memory_in_bytes(self: &RustBuildPlan) -> u64;
    fn vm_cpus(self: &RustBuildPlan) -> u32;
    fn vm_memory_in_bytes(self: &RustBuildPlan) -> u64;
    fn mounts(self: &RustBuildPlan) -> Vec<RustMount>;
    fn steps(self: &RustBuildPlan) -> Vec<RustBuildStep>;
    fn environment(self: &RustBuildPlan) -> Vec<String>;
    fn label_keys(self: &RustBuildPlan) -> Vec<String>;
    fn label(self: &RustBuildPlan, key: &str) -> Option<String>;
    fn user(self: &RustBuildPlan) -> Option<String>;
    fn working_directory(self: &RustBuildPlan) -> Option<String>;
    fn interface(self: &RustBuildPlan) -> &RustNatInterface;
    fn base_key(self: &RustBuildPlan) -> &str;
    fn rootfs_size_in_bytes(self: &RustBuildPlan) -> u64;
    fn cache_restore(self: &RustBuildPlan) -> bool;
    fn cache_keep(self: &RustBuildPlan) -> u64;
    fn cache_keep_for_seconds(self: &RustBuildPlan) -> u64;
    fn shell(self: &RustBuildPlan) -> Vec<String>;
    fn keepalive(self: &RustBuildPlan) -> Vec<String>;
    fn reclaim(self: &RustBuildPlan) -> bool;
  }

  extern "Swift" {
    fn czbridge_last_error() -> String;

    fn czbridge_boot(spec: RustBootSpec, store_root: &str, kernel_path: &str, initfs_reference: &str) -> i32;

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

    fn czbridge_build(plan: RustBuildPlan, store_root: &str, kernel_path: &str, initfs_reference: &str) -> i32;

    fn czbridge_provision(
      store_root: &str,
      kernel_path: &str,
      kernel_url: &str,
      kernel_in_archive: &str,
      initfs_reference: &str,
    ) -> i32;

    fn czbridge_resize(id: &str, terminal: i32) -> i32;

    fn czbridge_is_running(name: &str) -> bool;

    fn czbridge_is_unpacked(store_root: &str, image_reference: &str) -> i32;
  }
}
