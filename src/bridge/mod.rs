//! The Swift types Rust holds, and the Rust values Swift reads and builds.
//!
//! swift-bridge's parser refuses a `cfg` on the bridge module, so the gate is
//! on `mod bridge` in `lib.rs`.
//!
//! Containerization's objects cross as opaque Swift classes (`Cz`-prefixed, so
//! they don't shadow the types they hold). A method that throws returns a
//! `CzOutcome` holding its value or its error: swift-bridge 0.1.59's
//! `Result<_, String>` from Swift miscompiles with most argument and value
//! types.
//!
//! Value types cross as opaque Rust types (`Rust`-prefixed, to avoid
//! Containerization's `Mount`, `User`, etc.): Swift reads them through
//! `accessors`, and builds them through the constructors there when it hands
//! one back. swift-bridge can't put a `String`-holding struct in a `Vec`
//! (declined upstream, swift-bridge#305). (No doc comments inside the module:
//! swift-bridge can't parse them.)

mod accessors;

use crate::containerization::BootLog as RustBootLog;
use crate::containerization::Dns as RustDns;
use crate::containerization::Ext4Unpacker as RustExt4Unpacker;
use crate::containerization::Hosts as RustHosts;
use crate::containerization::Kernel as RustKernel;
use crate::containerization::LinuxCapabilities as RustLinuxCapabilities;
use crate::containerization::LinuxProcessConfiguration as RustLinuxProcessConfiguration;
use crate::containerization::Mount as RustMount;
use crate::containerization::NatInterface as RustNatInterface;
use crate::containerization::SystemPlatform as RustSystemPlatform;
use crate::containerization::UnixSocketConfiguration as RustUnixSocketConfiguration;
use crate::containerization::container_manager::CreateOptions as RustCreateOptions;
use crate::containerization::container_manager::RootfsCreateOptions as RustRootfsCreateOptions;
use crate::containerization::hosts::Entry as RustHostsEntry;
use crate::containerization::image::Description as RustImageDescription;
use crate::containerization::linux_container::Configuration as RustLinuxContainerConfiguration;
use crate::containerization_extras::IPv6Address as RustIPv6Address;
use crate::containerization_extras::IpAddress as RustIpAddress;
use crate::containerization_oci::Descriptor as RustDescriptor;
use crate::containerization_oci::Platform as RustPlatform;
use crate::containerization_oci::User as RustUser;
use crate::platform::Configure as RustConfigure;
use crate::platform::Progress as RustProgressHandler;

#[swift_bridge::bridge]
pub(crate) mod ffi {
  // `Mount.RuntimeOptions`, less its options. (`Any` is a Swift keyword.)
  enum RuntimeKind {
    Virtioblk,
    Virtiofs,
    Shared,
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

  // `BootLog`, less what it writes to.
  enum BootLogKind {
    File,
    FileHandle,
  }

  // `LinuxRLimit.Kind`.
  enum RlimitKind {
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

  // The sets of `LinuxCapabilities`.
  enum CapabilitySet {
    Bounding,
    Effective,
    Inheritable,
    Permitted,
    Ambient,
  }

  // `SystemPlatform.OS`.
  enum PlatformOs {
    Linux,
    Darwin,
  }

  // `SystemPlatform.Architecture`.
  enum PlatformArchitecture {
    Arm64,
    Amd64,
  }

  // `ProgressEvent`, less its value: `addItems` is `Items`, and so on.
  enum ProgressKind {
    Items,
    TotalItems,
    Size,
    TotalSize,
  }

  // `EXT4.JournalConfig.JournalMode`.
  enum JournalModeKind {
    Writeback,
    Ordered,
    Journal,
  }

  extern "Rust" {
    // A `(inout LinuxContainer.Configuration) -> Void`: Swift calls it once,
    // with the configuration it seeded.
    type RustConfigure;
    fn call(self: &RustConfigure, configuration: &mut RustLinuxContainerConfiguration);

    // A `ProgressHandler?`. Swift asks `is_some` before calling it.
    type RustProgressHandler;
    #[swift_bridge(swift_name = "isSome")]
    fn is_some(self: &RustProgressHandler) -> bool;
    fn call(self: &RustProgressHandler, kinds: Vec<ProgressKind>, values: Vec<i64>);

    // Its `UInt128` value crosses as two halves.
    type RustIPv6Address;
    #[swift_bridge(swift_name = "valueHigh")]
    fn value_high(self: &RustIPv6Address) -> u64;
    #[swift_bridge(swift_name = "valueLow")]
    fn value_low(self: &RustIPv6Address) -> u64;
    fn zone(self: &RustIPv6Address) -> Option<&str>;

    type RustIpAddress;
    #[swift_bridge(swift_name = "holdsV6")]
    fn holds_v6(self: &RustIpAddress) -> bool;
    #[swift_bridge(swift_name = "v4Value")]
    fn v4_value(self: &RustIpAddress) -> u32;
    fn v6(self: &RustIpAddress) -> &RustIPv6Address;

    type RustPlatform;
    fn architecture(self: &RustPlatform) -> &str;
    fn os(self: &RustPlatform) -> &str;
    #[swift_bridge(swift_name = "osVersion")]
    fn os_version(self: &RustPlatform) -> Option<&str>;
    #[swift_bridge(swift_name = "hasOsFeatures")]
    fn has_os_features(self: &RustPlatform) -> bool;
    #[swift_bridge(swift_name = "osFeaturesLen")]
    fn os_features_len(self: &RustPlatform) -> usize;
    #[swift_bridge(swift_name = "osFeaturesAt")]
    fn os_features_at(self: &RustPlatform, index: usize) -> &str;
    fn variant(self: &RustPlatform) -> Option<&str>;

    type RustDescriptor;
    #[swift_bridge(swift_name = "mediaType")]
    fn media_type(self: &RustDescriptor) -> &str;
    fn digest(self: &RustDescriptor) -> &str;
    fn size(self: &RustDescriptor) -> i64;
    #[swift_bridge(swift_name = "hasUrls")]
    fn has_urls(self: &RustDescriptor) -> bool;
    #[swift_bridge(swift_name = "urlsLen")]
    fn urls_len(self: &RustDescriptor) -> usize;
    #[swift_bridge(swift_name = "urlsAt")]
    fn urls_at(self: &RustDescriptor, index: usize) -> &str;
    #[swift_bridge(swift_name = "hasAnnotations")]
    fn has_annotations(self: &RustDescriptor) -> bool;
    #[swift_bridge(swift_name = "annotationsLen")]
    fn annotations_len(self: &RustDescriptor) -> usize;
    #[swift_bridge(swift_name = "annotationKeyAt")]
    fn annotation_key_at(self: &RustDescriptor, index: usize) -> &str;
    #[swift_bridge(swift_name = "annotationValueAt")]
    fn annotation_value_at(self: &RustDescriptor, index: usize) -> &str;
    #[swift_bridge(swift_name = "hasPlatform")]
    fn has_platform(self: &RustDescriptor) -> bool;
    fn platform(self: &RustDescriptor) -> &RustPlatform;
    #[swift_bridge(swift_name = "artifactType")]
    fn artifact_type(self: &RustDescriptor) -> Option<&str>;

    type RustImageDescription;
    fn reference(self: &RustImageDescription) -> &str;
    fn descriptor(self: &RustImageDescription) -> &RustDescriptor;

    type RustExt4Unpacker;
    #[swift_bridge(swift_name = "capacityInBytes")]
    fn capacity_in_bytes(self: &RustExt4Unpacker) -> u64;
    #[swift_bridge(swift_name = "hasJournal")]
    fn has_journal(self: &RustExt4Unpacker) -> bool;
    #[swift_bridge(swift_name = "journalSize")]
    fn journal_size(self: &RustExt4Unpacker) -> Option<u64>;
    #[swift_bridge(swift_name = "hasJournalMode")]
    fn has_journal_mode(self: &RustExt4Unpacker) -> bool;
    #[swift_bridge(swift_name = "journalMode")]
    fn journal_mode(self: &RustExt4Unpacker) -> JournalModeKind;

    type RustRootfsCreateOptions;
    #[swift_bridge(swift_name = "hasWritableLayer")]
    fn has_writable_layer(self: &RustRootfsCreateOptions) -> bool;
    #[swift_bridge(swift_name = "writableLayer")]
    fn writable_layer(self: &RustRootfsCreateOptions) -> &RustMount;
    fn networking(self: &RustRootfsCreateOptions) -> bool;
    #[swift_bridge(swift_name = "vmCpus")]
    fn vm_cpus(self: &RustRootfsCreateOptions) -> u32;
    #[swift_bridge(swift_name = "vmMemoryInBytes")]
    fn vm_memory_in_bytes(self: &RustRootfsCreateOptions) -> u64;

    type RustMount;
    #[swift_bridge(swift_name = "mountType")]
    fn mount_type(self: &RustMount) -> &str;
    fn source(self: &RustMount) -> &str;
    fn destination(self: &RustMount) -> &str;
    #[swift_bridge(swift_name = "optionsLen")]
    fn options_len(self: &RustMount) -> usize;
    #[swift_bridge(swift_name = "optionsAt")]
    fn options_at(self: &RustMount, index: usize) -> &str;
    #[swift_bridge(swift_name = "runtimeKind")]
    fn runtime_kind(self: &RustMount) -> RuntimeKind;
    #[swift_bridge(swift_name = "runtimeOptionsLen")]
    fn runtime_options_len(self: &RustMount) -> usize;
    #[swift_bridge(swift_name = "runtimeOptionsAt")]
    fn runtime_options_at(self: &RustMount, index: usize) -> &str;

    type RustUnixSocketConfiguration;
    fn source(self: &RustUnixSocketConfiguration) -> String;
    fn destination(self: &RustUnixSocketConfiguration) -> String;
    fn permissions(self: &RustUnixSocketConfiguration) -> Option<u32>;
    fn direction(self: &RustUnixSocketConfiguration) -> SocketDirection;

    type RustNatInterface;
    #[swift_bridge(swift_name = "ipv4Address")]
    fn ipv4_address(self: &RustNatInterface) -> &str;
    #[swift_bridge(swift_name = "ipv4Gateway")]
    fn ipv4_gateway(self: &RustNatInterface) -> Option<&str>;
    #[swift_bridge(swift_name = "ipv6Address")]
    fn ipv6_address(self: &RustNatInterface) -> Option<&str>;
    #[swift_bridge(swift_name = "ipv6Gateway")]
    fn ipv6_gateway(self: &RustNatInterface) -> Option<&str>;
    #[swift_bridge(swift_name = "macAddress")]
    fn mac_address(self: &RustNatInterface) -> Option<&str>;
    fn mtu(self: &RustNatInterface) -> u32;

    type RustDns;
    #[swift_bridge(swift_name = "nameserversLen")]
    fn nameservers_len(self: &RustDns) -> usize;
    #[swift_bridge(swift_name = "nameserversAt")]
    fn nameservers_at(self: &RustDns, index: usize) -> &str;
    fn domain(self: &RustDns) -> Option<&str>;
    #[swift_bridge(swift_name = "searchDomainsLen")]
    fn search_domains_len(self: &RustDns) -> usize;
    #[swift_bridge(swift_name = "searchDomainsAt")]
    fn search_domains_at(self: &RustDns, index: usize) -> &str;
    #[swift_bridge(swift_name = "optionsLen")]
    fn options_len(self: &RustDns) -> usize;
    #[swift_bridge(swift_name = "optionsAt")]
    fn options_at(self: &RustDns, index: usize) -> &str;

    type RustHostsEntry;
    #[swift_bridge(swift_name = "ipAddress")]
    fn ip_address(self: &RustHostsEntry) -> &str;
    #[swift_bridge(swift_name = "hostnamesLen")]
    fn hostnames_len(self: &RustHostsEntry) -> usize;
    #[swift_bridge(swift_name = "hostnamesAt")]
    fn hostnames_at(self: &RustHostsEntry, index: usize) -> &str;
    fn comment(self: &RustHostsEntry) -> Option<&str>;

    type RustHosts;
    #[swift_bridge(swift_name = "entriesLen")]
    fn entries_len(self: &RustHosts) -> usize;
    #[swift_bridge(swift_name = "entriesAt")]
    fn entries_at(self: &RustHosts, index: usize) -> &RustHostsEntry;
    fn comment(self: &RustHosts) -> Option<&str>;

    type RustBootLog;
    fn kind(self: &RustBootLog) -> BootLogKind;
    fn path(self: &RustBootLog) -> String;
    fn append(self: &RustBootLog) -> bool;
    #[swift_bridge(swift_name = "fileHandle")]
    fn file_handle(self: &RustBootLog) -> i32;

    type RustUser;
    fn uid(self: &RustUser) -> u32;
    fn gid(self: &RustUser) -> u32;
    fn umask(self: &RustUser) -> Option<u32>;
    #[swift_bridge(swift_name = "additionalGidsLen")]
    fn additional_gids_len(self: &RustUser) -> usize;
    #[swift_bridge(swift_name = "additionalGidsAt")]
    fn additional_gids_at(self: &RustUser, index: usize) -> u32;
    fn username(self: &RustUser) -> &str;

    type RustLinuxCapabilities;
    #[swift_bridge(swift_name = "setLen")]
    fn set_len(self: &RustLinuxCapabilities, set: CapabilitySet) -> usize;
    #[swift_bridge(swift_name = "setAt")]
    fn set_at(self: &RustLinuxCapabilities, set: CapabilitySet, index: usize) -> &str;

    type RustLinuxProcessConfiguration;
    #[swift_bridge(swift_name = "setArguments")]
    fn set_arguments(self: &mut RustLinuxProcessConfiguration, arguments: Vec<String>);
    #[swift_bridge(swift_name = "setEnvironmentVariables")]
    fn set_environment_variables(self: &mut RustLinuxProcessConfiguration, environment_variables: Vec<String>);
    #[swift_bridge(swift_name = "setWorkingDirectory")]
    fn set_working_directory(self: &mut RustLinuxProcessConfiguration, working_directory: String);
    #[swift_bridge(swift_name = "setUser")]
    fn set_user(
      self: &mut RustLinuxProcessConfiguration,
      uid: u32,
      gid: u32,
      umask: Option<u32>,
      additional_gids: Vec<u32>,
      username: String,
    );
    #[swift_bridge(swift_name = "setNoNewPrivileges")]
    fn set_no_new_privileges(self: &mut RustLinuxProcessConfiguration, no_new_privileges: bool);
    #[swift_bridge(swift_name = "setCapabilities")]
    fn set_capabilities(
      self: &mut RustLinuxProcessConfiguration,
      bounding: Vec<String>,
      effective: Vec<String>,
      inheritable: Vec<String>,
      permitted: Vec<String>,
      ambient: Vec<String>,
    );
    #[swift_bridge(swift_name = "setTerminal")]
    fn set_terminal(self: &mut RustLinuxProcessConfiguration, terminal: bool);
    #[swift_bridge(swift_name = "clearRlimits")]
    fn clear_rlimits(self: &mut RustLinuxProcessConfiguration);
    #[swift_bridge(swift_name = "pushRlimit")]
    fn push_rlimit(self: &mut RustLinuxProcessConfiguration, kind: RlimitKind, hard: u64, soft: u64);
    #[swift_bridge(swift_name = "argumentsLen")]
    fn arguments_len(self: &RustLinuxProcessConfiguration) -> usize;
    #[swift_bridge(swift_name = "argumentsAt")]
    fn arguments_at(self: &RustLinuxProcessConfiguration, index: usize) -> &str;
    #[swift_bridge(swift_name = "environmentVariablesLen")]
    fn environment_variables_len(self: &RustLinuxProcessConfiguration) -> usize;
    #[swift_bridge(swift_name = "environmentVariablesAt")]
    fn environment_variables_at(self: &RustLinuxProcessConfiguration, index: usize) -> &str;
    #[swift_bridge(swift_name = "workingDirectory")]
    fn working_directory(self: &RustLinuxProcessConfiguration) -> &str;
    fn user(self: &RustLinuxProcessConfiguration) -> &RustUser;
    #[swift_bridge(swift_name = "rlimitsLen")]
    fn rlimits_len(self: &RustLinuxProcessConfiguration) -> usize;
    #[swift_bridge(swift_name = "rlimitKindAt")]
    fn rlimit_kind_at(self: &RustLinuxProcessConfiguration, index: usize) -> RlimitKind;
    #[swift_bridge(swift_name = "rlimitHardAt")]
    fn rlimit_hard_at(self: &RustLinuxProcessConfiguration, index: usize) -> u64;
    #[swift_bridge(swift_name = "rlimitSoftAt")]
    fn rlimit_soft_at(self: &RustLinuxProcessConfiguration, index: usize) -> u64;
    #[swift_bridge(swift_name = "noNewPrivileges")]
    fn no_new_privileges(self: &RustLinuxProcessConfiguration) -> bool;
    fn capabilities(self: &RustLinuxProcessConfiguration) -> &RustLinuxCapabilities;
    fn terminal(self: &RustLinuxProcessConfiguration) -> bool;
    fn stdin(self: &RustLinuxProcessConfiguration) -> Option<i32>;
    fn stdout(self: &RustLinuxProcessConfiguration) -> Option<i32>;
    fn stderr(self: &RustLinuxProcessConfiguration) -> Option<i32>;

    type RustLinuxContainerConfiguration;
    #[swift_bridge(swift_name = "processMut")]
    fn process_mut(self: &mut RustLinuxContainerConfiguration) -> &mut RustLinuxProcessConfiguration;
    #[swift_bridge(swift_name = "setCpus")]
    fn set_cpus(self: &mut RustLinuxContainerConfiguration, cpus: u32);
    #[swift_bridge(swift_name = "setMemoryInBytes")]
    fn set_memory_in_bytes(self: &mut RustLinuxContainerConfiguration, memory_in_bytes: u64);
    #[swift_bridge(swift_name = "setHostname")]
    fn set_hostname(self: &mut RustLinuxContainerConfiguration, hostname: Option<String>);
    #[swift_bridge(swift_name = "setMaskedPaths")]
    fn set_masked_paths(self: &mut RustLinuxContainerConfiguration, masked_paths: Vec<String>);
    #[swift_bridge(swift_name = "setReadonlyPaths")]
    fn set_readonly_paths(self: &mut RustLinuxContainerConfiguration, readonly_paths: Vec<String>);
    #[swift_bridge(swift_name = "setVirtualization")]
    fn set_virtualization(self: &mut RustLinuxContainerConfiguration, virtualization: bool);
    #[swift_bridge(swift_name = "setOciRuntimePath")]
    fn set_oci_runtime_path(self: &mut RustLinuxContainerConfiguration, oci_runtime_path: Option<String>);
    #[swift_bridge(swift_name = "setUseInit")]
    fn set_use_init(self: &mut RustLinuxContainerConfiguration, use_init: bool);
    #[swift_bridge(swift_name = "clearSysctl")]
    fn clear_sysctl(self: &mut RustLinuxContainerConfiguration);
    #[swift_bridge(swift_name = "clearInterfaces")]
    fn clear_interfaces(self: &mut RustLinuxContainerConfiguration);
    #[swift_bridge(swift_name = "clearSockets")]
    fn clear_sockets(self: &mut RustLinuxContainerConfiguration);
    #[swift_bridge(swift_name = "clearMounts")]
    fn clear_mounts(self: &mut RustLinuxContainerConfiguration);
    #[swift_bridge(swift_name = "clearDns")]
    fn clear_dns(self: &mut RustLinuxContainerConfiguration);
    #[swift_bridge(swift_name = "clearHosts")]
    fn clear_hosts(self: &mut RustLinuxContainerConfiguration);
    #[swift_bridge(swift_name = "clearBootLog")]
    fn clear_boot_log(self: &mut RustLinuxContainerConfiguration);
    #[swift_bridge(swift_name = "insertSysctl")]
    fn insert_sysctl(self: &mut RustLinuxContainerConfiguration, key: String, value: String);
    #[swift_bridge(swift_name = "pushInterface")]
    fn push_interface(
      self: &mut RustLinuxContainerConfiguration,
      ipv4_address: String,
      ipv4_gateway: Option<String>,
      ipv6_address: Option<String>,
      ipv6_gateway: Option<String>,
      mac_address: Option<String>,
      mtu: u32,
    );
    #[swift_bridge(swift_name = "pushSocket")]
    fn push_socket(
      self: &mut RustLinuxContainerConfiguration,
      source: String,
      destination: String,
      permissions: Option<u32>,
      direction: SocketDirection,
    );
    #[swift_bridge(swift_name = "pushMount")]
    fn push_mount(
      self: &mut RustLinuxContainerConfiguration,
      mount_type: String,
      source: String,
      destination: String,
      options: Vec<String>,
      kind: RuntimeKind,
      runtime_options: Vec<String>,
    );
    #[swift_bridge(swift_name = "setDns")]
    fn set_dns(
      self: &mut RustLinuxContainerConfiguration,
      nameservers: Vec<String>,
      domain: Option<String>,
      search_domains: Vec<String>,
      options: Vec<String>,
    );
    #[swift_bridge(swift_name = "setHosts")]
    fn set_hosts(self: &mut RustLinuxContainerConfiguration, comment: Option<String>);
    #[swift_bridge(swift_name = "pushHostsEntry")]
    fn push_hosts_entry(
      self: &mut RustLinuxContainerConfiguration,
      ip_address: String,
      hostnames: Vec<String>,
      comment: Option<String>,
    );
    #[swift_bridge(swift_name = "setBootLogFile")]
    fn set_boot_log_file(self: &mut RustLinuxContainerConfiguration, path: String, append: bool);
    #[swift_bridge(swift_name = "setBootLogFileHandle")]
    fn set_boot_log_file_handle(self: &mut RustLinuxContainerConfiguration, descriptor: i32);
    #[swift_bridge(swift_name = "setSeccompProfile")]
    fn set_seccomp_profile(self: &mut RustLinuxContainerConfiguration, mode: SeccompMode, profile: Option<String>);
    fn process(self: &RustLinuxContainerConfiguration) -> &RustLinuxProcessConfiguration;
    fn cpus(self: &RustLinuxContainerConfiguration) -> u32;
    #[swift_bridge(swift_name = "memoryInBytes")]
    fn memory_in_bytes(self: &RustLinuxContainerConfiguration) -> u64;
    fn hostname(self: &RustLinuxContainerConfiguration) -> Option<&str>;
    #[swift_bridge(swift_name = "sysctlLen")]
    fn sysctl_len(self: &RustLinuxContainerConfiguration) -> usize;
    #[swift_bridge(swift_name = "sysctlKeyAt")]
    fn sysctl_key_at(self: &RustLinuxContainerConfiguration, index: usize) -> &str;
    #[swift_bridge(swift_name = "sysctlValueAt")]
    fn sysctl_value_at(self: &RustLinuxContainerConfiguration, index: usize) -> &str;
    #[swift_bridge(swift_name = "interfacesLen")]
    fn interfaces_len(self: &RustLinuxContainerConfiguration) -> usize;
    #[swift_bridge(swift_name = "interfacesAt")]
    fn interfaces_at(self: &RustLinuxContainerConfiguration, index: usize) -> &RustNatInterface;
    #[swift_bridge(swift_name = "socketsLen")]
    fn sockets_len(self: &RustLinuxContainerConfiguration) -> usize;
    #[swift_bridge(swift_name = "socketsAt")]
    fn sockets_at(self: &RustLinuxContainerConfiguration, index: usize) -> &RustUnixSocketConfiguration;
    #[swift_bridge(swift_name = "mountsLen")]
    fn mounts_len(self: &RustLinuxContainerConfiguration) -> usize;
    #[swift_bridge(swift_name = "mountsAt")]
    fn mounts_at(self: &RustLinuxContainerConfiguration, index: usize) -> &RustMount;
    #[swift_bridge(swift_name = "maskedPathsLen")]
    fn masked_paths_len(self: &RustLinuxContainerConfiguration) -> usize;
    #[swift_bridge(swift_name = "maskedPathsAt")]
    fn masked_paths_at(self: &RustLinuxContainerConfiguration, index: usize) -> &str;
    #[swift_bridge(swift_name = "readonlyPathsLen")]
    fn readonly_paths_len(self: &RustLinuxContainerConfiguration) -> usize;
    #[swift_bridge(swift_name = "readonlyPathsAt")]
    fn readonly_paths_at(self: &RustLinuxContainerConfiguration, index: usize) -> &str;
    #[swift_bridge(swift_name = "hasDns")]
    fn has_dns(self: &RustLinuxContainerConfiguration) -> bool;
    fn dns(self: &RustLinuxContainerConfiguration) -> &RustDns;
    #[swift_bridge(swift_name = "hasHosts")]
    fn has_hosts(self: &RustLinuxContainerConfiguration) -> bool;
    fn hosts(self: &RustLinuxContainerConfiguration) -> &RustHosts;
    fn virtualization(self: &RustLinuxContainerConfiguration) -> bool;
    #[swift_bridge(swift_name = "hasBootLog")]
    fn has_boot_log(self: &RustLinuxContainerConfiguration) -> bool;
    #[swift_bridge(swift_name = "bootLog")]
    fn boot_log(self: &RustLinuxContainerConfiguration) -> &RustBootLog;
    #[swift_bridge(swift_name = "ociRuntimePath")]
    fn oci_runtime_path(self: &RustLinuxContainerConfiguration) -> Option<&str>;
    #[swift_bridge(swift_name = "seccompMode")]
    fn seccomp_mode(self: &RustLinuxContainerConfiguration) -> SeccompMode;
    #[swift_bridge(swift_name = "seccompProfile")]
    fn seccomp_profile(self: &RustLinuxContainerConfiguration) -> Option<&str>;
    #[swift_bridge(swift_name = "useInit")]
    fn use_init(self: &RustLinuxContainerConfiguration) -> bool;

    type RustCreateOptions;
    #[swift_bridge(swift_name = "rootfsSizeInBytes")]
    fn rootfs_size_in_bytes(self: &RustCreateOptions) -> u64;
    #[swift_bridge(swift_name = "writableLayerSizeInBytes")]
    fn writable_layer_size_in_bytes(self: &RustCreateOptions) -> Option<u64>;
    #[swift_bridge(swift_name = "readOnly")]
    fn read_only(self: &RustCreateOptions) -> bool;
    fn networking(self: &RustCreateOptions) -> bool;
    #[swift_bridge(swift_name = "vmCpus")]
    fn vm_cpus(self: &RustCreateOptions) -> u32;
    #[swift_bridge(swift_name = "vmMemoryInBytes")]
    fn vm_memory_in_bytes(self: &RustCreateOptions) -> u64;

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

  extern "Swift" {
    // What a throwing call returned, or why it failed. Rust asks `error`
    // first, then takes the one value the call makes, once.
    type CzOutcome;
    fn error(self: &CzOutcome) -> Option<String>;
    #[swift_bridge(swift_name = "errorCode")]
    fn error_code(self: &CzOutcome) -> Option<String>;
    #[swift_bridge(swift_name = "localContentStore")]
    fn local_content_store(self: &CzOutcome) -> CzLocalContentStore;
    fn content(self: &CzOutcome) -> CzContent;
    #[swift_bridge(swift_name = "imageStore")]
    fn image_store(self: &CzOutcome) -> CzImageStore;
    fn image(self: &CzOutcome) -> CzImage;
    fn images(self: &CzOutcome) -> CzImages;
    #[swift_bridge(swift_name = "initImage")]
    fn init_image(self: &CzOutcome) -> CzInitImage;
    #[swift_bridge(swift_name = "containerManager")]
    fn container_manager(self: &CzOutcome) -> CzContainerManager;
    #[swift_bridge(swift_name = "linuxContainer")]
    fn linux_container(self: &CzOutcome) -> CzLinuxContainer;
    #[swift_bridge(swift_name = "linuxProcess")]
    fn linux_process(self: &CzOutcome) -> CzLinuxProcess;
    #[swift_bridge(swift_name = "contentWriter")]
    fn content_writer(self: &CzOutcome) -> CzContentWriter;
    #[swift_bridge(swift_name = "writtenSize")]
    fn written_size(self: &CzOutcome) -> i64;
    #[swift_bridge(swift_name = "writtenDigest")]
    fn written_digest(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "ext4Reader")]
    fn ext4_reader(self: &CzOutcome) -> CzExt4Reader;

    #[swift_bridge(swift_name = "platformArchitecture")]
    fn platform_architecture(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "platformOs")]
    fn platform_os(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "platformOsVersion")]
    fn platform_os_version(self: &CzOutcome) -> Option<String>;
    #[swift_bridge(swift_name = "platformHasOsFeatures")]
    fn platform_has_os_features(self: &CzOutcome) -> bool;
    #[swift_bridge(swift_name = "platformOsFeatures")]
    fn platform_os_features(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "platformVariant")]
    fn platform_variant(self: &CzOutcome) -> Option<String>;

    #[swift_bridge(swift_name = "currentPlatform")]
    fn cz_platform_current() -> CzOutcome;

    // Addresses, field by field: Rust builds its own. Each address getter
    // also reads the address in an `IPAddress`, a CIDR block or a `CIDR`, and
    // `prefixLength` the prefix of a CIDR block or a `CIDR`.
    fn boolean(self: &CzOutcome) -> bool;
    // Whether an optional result is there.
    #[swift_bridge(swift_name = "isSome")]
    fn is_some(self: &CzOutcome) -> bool;
    #[swift_bridge(swift_name = "isIPv6")]
    fn is_ipv6(self: &CzOutcome) -> bool;
    #[swift_bridge(swift_name = "ipv4Value")]
    fn ipv4_value(self: &CzOutcome) -> u32;
    #[swift_bridge(swift_name = "ipv6High")]
    fn ipv6_high(self: &CzOutcome) -> u64;
    #[swift_bridge(swift_name = "ipv6Low")]
    fn ipv6_low(self: &CzOutcome) -> u64;
    #[swift_bridge(swift_name = "ipv6Zone")]
    fn ipv6_zone(self: &CzOutcome) -> Option<String>;
    #[swift_bridge(swift_name = "prefixLength")]
    fn prefix_length(self: &CzOutcome) -> u8;
    #[swift_bridge(swift_name = "macValue")]
    fn mac_value(self: &CzOutcome) -> u64;
    // A `UInt128`, in two halves.
    #[swift_bridge(swift_name = "wideHigh")]
    fn wide_high(self: &CzOutcome) -> u64;
    #[swift_bridge(swift_name = "wideLow")]
    fn wide_low(self: &CzOutcome) -> u64;

    // `IPv4Address`, as its value.
    #[swift_bridge(swift_name = "ipv4AddressFromBytes")]
    fn cz_ipv4_address_from_bytes(bytes: Vec<u8>) -> CzOutcome;
    #[swift_bridge(swift_name = "parseIPv4Address")]
    fn cz_ipv4_address_parse(string: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv4AddressBytes")]
    fn cz_ipv4_address_bytes(value: u32) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv4AddressDescription")]
    fn cz_ipv4_address_description(value: u32) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv4AddressIsUnspecified")]
    fn cz_ipv4_address_is_unspecified(value: u32) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv4AddressIsLoopback")]
    fn cz_ipv4_address_is_loopback(value: u32) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv4AddressIsMulticast")]
    fn cz_ipv4_address_is_multicast(value: u32) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv4AddressIsLinkLocal")]
    fn cz_ipv4_address_is_link_local(value: u32) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv4AddressIsBroadcast")]
    fn cz_ipv4_address_is_broadcast(value: u32) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv4AddressLessThan")]
    fn cz_ipv4_address_less_than(lhs: u32, rhs: u32) -> CzOutcome;

    #[swift_bridge(swift_name = "parseIPv6Address")]
    fn cz_ipv6_address_parse(address: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv6AddressFromBytes")]
    fn cz_ipv6_address_from_bytes(bytes: Vec<u8>, zone: Option<String>) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv6AddressUnspecified")]
    fn cz_ipv6_address_unspecified() -> CzOutcome;
    #[swift_bridge(swift_name = "ipv6AddressLoopback")]
    fn cz_ipv6_address_loopback() -> CzOutcome;
    #[swift_bridge(swift_name = "ipv6AddressBytes")]
    fn cz_ipv6_address_bytes(address: RustIPv6Address) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv6AddressDescription")]
    fn cz_ipv6_address_description(address: RustIPv6Address) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv6AddressIsUnspecified")]
    fn cz_ipv6_address_is_unspecified(address: RustIPv6Address) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv6AddressIsLoopback")]
    fn cz_ipv6_address_is_loopback(address: RustIPv6Address) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv6AddressIsMulticast")]
    fn cz_ipv6_address_is_multicast(address: RustIPv6Address) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv6AddressIsLinkLocal")]
    fn cz_ipv6_address_is_link_local(address: RustIPv6Address) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv6AddressIsUniqueLocal")]
    fn cz_ipv6_address_is_unique_local(address: RustIPv6Address) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv6AddressIsGlobalUnicast")]
    fn cz_ipv6_address_is_global_unicast(address: RustIPv6Address) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv6AddressIsDocumentation")]
    fn cz_ipv6_address_is_documentation(address: RustIPv6Address) -> CzOutcome;
    #[swift_bridge(swift_name = "ipv6AddressLessThan")]
    fn cz_ipv6_address_less_than(lhs: RustIPv6Address, rhs: RustIPv6Address) -> CzOutcome;

    #[swift_bridge(swift_name = "parseIPAddress")]
    fn cz_ip_address_parse(string: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "ipAddressDescription")]
    fn cz_ip_address_description(address: RustIpAddress) -> CzOutcome;
    #[swift_bridge(swift_name = "ipAddressIsV4")]
    fn cz_ip_address_is_v4(address: RustIpAddress) -> CzOutcome;
    #[swift_bridge(swift_name = "ipAddressIsV6")]
    fn cz_ip_address_is_v6(address: RustIpAddress) -> CzOutcome;
    #[swift_bridge(swift_name = "ipAddressIPv4")]
    fn cz_ip_address_ipv4(address: RustIpAddress) -> CzOutcome;
    #[swift_bridge(swift_name = "ipAddressIPv6")]
    fn cz_ip_address_ipv6(address: RustIpAddress) -> CzOutcome;
    #[swift_bridge(swift_name = "ipAddressIsLoopback")]
    fn cz_ip_address_is_loopback(address: RustIpAddress) -> CzOutcome;
    #[swift_bridge(swift_name = "ipAddressIsMulticast")]
    fn cz_ip_address_is_multicast(address: RustIpAddress) -> CzOutcome;
    #[swift_bridge(swift_name = "ipAddressIsUnspecified")]
    fn cz_ip_address_is_unspecified(address: RustIpAddress) -> CzOutcome;

    // `Prefix`, as its length.
    #[swift_bridge(swift_name = "prefixWithLength")]
    fn cz_prefix_new(length: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "prefixIPv4")]
    fn cz_prefix_ipv4(length: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "prefixIPv6")]
    fn cz_prefix_ipv6(length: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "prefixDescription")]
    fn cz_prefix_description(length: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "prefixSuffixMask32")]
    fn cz_prefix_suffix_mask32(length: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "prefixPrefixMask32")]
    fn cz_prefix_prefix_mask32(length: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "prefixSuffixMask128")]
    fn cz_prefix_suffix_mask128(length: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "prefixPrefixMask128")]
    fn cz_prefix_prefix_mask128(length: u8) -> CzOutcome;

    // `CIDRv4`, as its address's value and its prefix's length.
    #[swift_bridge(swift_name = "parseCIDRv4")]
    fn cz_cidr_v4_parse(cidr: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrV4")]
    fn cz_cidr_v4_new(address: u32, prefix: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrV4FromRange")]
    fn cz_cidr_v4_from_range(lower: u32, upper: u32) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrV4Lower")]
    fn cz_cidr_v4_lower(address: u32, prefix: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrV4Upper")]
    fn cz_cidr_v4_upper(address: u32, prefix: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrV4Gateway")]
    fn cz_cidr_v4_gateway(address: u32, prefix: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrV4Contains")]
    fn cz_cidr_v4_contains(address: u32, prefix: u8, ip: u32) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrV4Description")]
    fn cz_cidr_v4_description(address: u32, prefix: u8) -> CzOutcome;

    // `CIDRv6`, as its address and its prefix's length.
    #[swift_bridge(swift_name = "parseCIDRv6")]
    fn cz_cidr_v6_parse(cidr: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrV6")]
    fn cz_cidr_v6_new(address: RustIPv6Address, prefix: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrV6FromRange")]
    fn cz_cidr_v6_from_range(lower: RustIPv6Address, upper: RustIPv6Address) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrV6Lower")]
    fn cz_cidr_v6_lower(address: RustIPv6Address, prefix: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrV6Upper")]
    fn cz_cidr_v6_upper(address: RustIPv6Address, prefix: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrV6Gateway")]
    fn cz_cidr_v6_gateway(address: RustIPv6Address, prefix: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrV6Contains")]
    fn cz_cidr_v6_contains(address: RustIPv6Address, prefix: u8, ip: RustIPv6Address) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrV6Description")]
    fn cz_cidr_v6_description(address: RustIPv6Address, prefix: u8) -> CzOutcome;

    // `CIDR`, as its case's address and its prefix's length.
    #[swift_bridge(swift_name = "parseCIDR")]
    fn cz_cidr_parse(cidr: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "cidr")]
    fn cz_cidr_new(address: RustIpAddress, prefix: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrFromRange")]
    fn cz_cidr_from_range(lower: RustIpAddress, upper: RustIpAddress) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrAddress")]
    fn cz_cidr_address(address: RustIpAddress, prefix: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrPrefix")]
    fn cz_cidr_prefix(address: RustIpAddress, prefix: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrLower")]
    fn cz_cidr_lower(address: RustIpAddress, prefix: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrUpper")]
    fn cz_cidr_upper(address: RustIpAddress, prefix: u8) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrContains")]
    fn cz_cidr_contains(address: RustIpAddress, prefix: u8, ip: RustIpAddress) -> CzOutcome;
    #[swift_bridge(swift_name = "cidrDescription")]
    fn cz_cidr_description(address: RustIpAddress, prefix: u8) -> CzOutcome;

    // `MACAddress`, as its value.
    #[swift_bridge(swift_name = "macAddress")]
    fn cz_mac_address_new(value: u64) -> CzOutcome;
    #[swift_bridge(swift_name = "macAddressFromBytes")]
    fn cz_mac_address_from_bytes(bytes: Vec<u8>) -> CzOutcome;
    #[swift_bridge(swift_name = "parseMACAddress")]
    fn cz_mac_address_parse(string: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "macAddressBytes")]
    fn cz_mac_address_bytes(value: u64) -> CzOutcome;
    #[swift_bridge(swift_name = "macAddressDescription")]
    fn cz_mac_address_description(value: u64) -> CzOutcome;
    #[swift_bridge(swift_name = "macAddressIsLocallyAdministered")]
    fn cz_mac_address_is_locally_administered(value: u64) -> CzOutcome;
    #[swift_bridge(swift_name = "macAddressIsMulticast")]
    fn cz_mac_address_is_multicast(value: u64) -> CzOutcome;
    #[swift_bridge(swift_name = "macAddressIPv6Address")]
    fn cz_mac_address_ipv6_address(value: u64, network: RustIPv6Address) -> CzOutcome;
    #[swift_bridge(swift_name = "macAddressLessThan")]
    fn cz_mac_address_less_than(lhs: u64, rhs: u64) -> CzOutcome;

    type CzExt4Reader;
    #[swift_bridge(swift_name = "openExt4Reader")]
    fn cz_ext4_reader_new(#[swift_bridge(label = "blockDevice")] block_device: &str) -> CzOutcome;
    fn export(self: &CzExt4Reader, archive: &str) -> CzOutcome;

    // `EXT4Unpacker.unpack(_:for:at:progress:)`, with the image as a
    // `duplicate()`.
    #[swift_bridge(swift_name = "unpackExt4")]
    fn cz_ext4_unpack(
      unpacker: RustExt4Unpacker,
      image: CzImage,
      platform: RustPlatform,
      at: &str,
      progress: RustProgressHandler,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "mountType")]
    fn mount_type(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "mountSource")]
    fn mount_source(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "mountDestination")]
    fn mount_destination(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "mountOptions")]
    fn mount_options(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "mountRuntimeKind")]
    fn mount_runtime_kind(self: &CzOutcome) -> RuntimeKind;
    #[swift_bridge(swift_name = "mountRuntimeOptions")]
    fn mount_runtime_options(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "exitCode")]
    fn exit_code(self: &CzOutcome) -> i32;
    #[swift_bridge(swift_name = "exitedAt")]
    fn exited_at(self: &CzOutcome) -> f64;
    fn number(self: &CzOutcome) -> u64;
    fn text(self: &CzOutcome) -> String;
    fn strings(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "hasBytes")]
    fn has_bytes(self: &CzOutcome) -> bool;
    fn bytes(self: &CzOutcome) -> Vec<u8>;

    type CzLocalContentStore;
    #[swift_bridge(swift_name = "openLocalContentStore")]
    fn cz_local_content_store_new(path: &str) -> CzOutcome;
    fn get(self: &CzLocalContentStore, digest: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "deleteDigests")]
    fn delete_digests(self: &CzLocalContentStore, digests: Vec<String>) -> CzOutcome;
    #[swift_bridge(swift_name = "deleteKeeping")]
    fn delete_keeping(self: &CzLocalContentStore, keeping: Vec<String>) -> CzOutcome;
    #[swift_bridge(swift_name = "totalAllocatedSize")]
    fn total_allocated_size(self: &CzLocalContentStore) -> CzOutcome;
    // `ImageStore(path:contentStore:)`, on the store it takes.
    #[swift_bridge(swift_name = "imageStore")]
    fn image_store(self: &CzLocalContentStore, path: &str) -> CzOutcome;
    // `body` gets the ingest directory, and returns whether it succeeded.
    fn ingest(self: &CzLocalContentStore, body: Box<dyn FnOnce(String) -> bool>) -> CzOutcome;

    type CzContentWriter;
    #[swift_bridge(swift_name = "openContentWriter")]
    fn cz_content_writer_new(base: &str) -> CzOutcome;
    fn create(self: &CzContentWriter, from: &str) -> CzOutcome;

    // A `Content?`: Rust asks `is_some` before anything else.
    type CzContent;
    #[swift_bridge(swift_name = "isSome")]
    fn is_some(self: &CzContent) -> bool;
    fn path(self: &CzContent) -> String;
    fn digest(self: &CzContent) -> CzOutcome;
    fn size(self: &CzContent) -> CzOutcome;
    fn data(self: &CzContent) -> CzOutcome;
    #[swift_bridge(swift_name = "dataRange")]
    fn data_range(self: &CzContent, offset: u64, length: usize) -> CzOutcome;

    type CzImageStore;
    #[swift_bridge(swift_name = "openImageStore")]
    fn cz_image_store_new(path: &str) -> CzOutcome;
    fn path(self: &CzImageStore) -> String;
    fn get(self: &CzImageStore, reference: &str, pull: bool) -> CzOutcome;
    fn list(self: &CzImageStore) -> CzOutcome;
    fn delete(
      self: &CzImageStore,
      reference: &str,
      #[swift_bridge(label = "performCleanup")] perform_cleanup: bool,
    ) -> CzOutcome;
    fn tag(self: &CzImageStore, existing: &str, new: &str) -> CzOutcome;
    fn pull(self: &CzImageStore, reference: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "getInitImage")]
    fn get_init_image(self: &CzImageStore, reference: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "cleanUpOrphanedBlobs")]
    fn clean_up_orphaned_blobs(self: &CzImageStore) -> CzOutcome;
    #[swift_bridge(swift_name = "calculateOrphanedBlobsSize")]
    fn calculate_orphaned_blobs_size(self: &CzImageStore) -> CzOutcome;
    fn create(self: &CzImageStore, description: RustImageDescription) -> CzOutcome;
    fn load(self: &CzImageStore, from: &str, progress: RustProgressHandler) -> CzOutcome;
    // `ContainerManager`'s inits, on the store they take: swift-bridge can't
    // pass a `&` Swift type as an argument.
    #[swift_bridge(swift_name = "containerManager")]
    fn container_manager(
      self: &CzImageStore,
      kernel: RustKernel,
      initfs: RustMount,
      rosetta: bool,
      #[swift_bridge(label = "nestedVirtualization")] nested_virtualization: bool,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "containerManager")]
    fn container_manager_with_initfs_reference(
      self: &CzImageStore,
      kernel: RustKernel,
      #[swift_bridge(label = "initfsReference")] initfs_reference: &str,
      rosetta: bool,
      #[swift_bridge(label = "nestedVirtualization")] nested_virtualization: bool,
    ) -> CzOutcome;

    type CzImages;
    fn len(self: &CzImages) -> usize;
    fn at(self: &CzImages, index: usize) -> CzImage;

    type CzImage;
    fn duplicate(self: &CzImage) -> CzImage;
    fn reference(self: &CzImage) -> String;
    fn digest(self: &CzImage) -> String;
    #[swift_bridge(swift_name = "mediaType")]
    fn media_type(self: &CzImage) -> String;
    #[swift_bridge(swift_name = "referencedDigests")]
    fn referenced_digests(self: &CzImage) -> CzOutcome;
    #[swift_bridge(swift_name = "getContent")]
    fn get_content(self: &CzImage, digest: &str) -> CzOutcome;

    type CzInitImage;
    fn name(self: &CzInitImage) -> String;
    #[swift_bridge(swift_name = "initBlock")]
    fn init_block(self: &CzInitImage, at: &str, platform: RustSystemPlatform) -> CzOutcome;

    // `create` takes the image as a `duplicate()`, for the same reason, and a
    // `seed` to fill with the configuration it seeds before `configuration`
    // changes it.
    type CzContainerManager;
    fn create(
      self: &CzContainerManager,
      id: &str,
      image: CzImage,
      options: RustCreateOptions,
      seed: RustLinuxContainerConfiguration,
      configuration: RustConfigure,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "createWithRootfs")]
    fn create_with_rootfs(
      self: &CzContainerManager,
      id: &str,
      image: CzImage,
      rootfs: RustMount,
      options: RustRootfsCreateOptions,
      seed: RustLinuxContainerConfiguration,
      configuration: RustConfigure,
    ) -> CzOutcome;
    fn delete(self: &CzContainerManager, id: &str) -> CzOutcome;

    type CzLinuxContainer;
    fn id(self: &CzLinuxContainer) -> String;
    fn create(self: &CzLinuxContainer) -> CzOutcome;
    fn start(self: &CzLinuxContainer) -> CzOutcome;
    fn stop(self: &CzLinuxContainer) -> CzOutcome;
    fn kill(self: &CzLinuxContainer, signal: i32) -> CzOutcome;
    fn wait(
      self: &CzLinuxContainer,
      #[swift_bridge(label = "timeoutInSeconds")] timeout_in_seconds: Option<i64>,
    ) -> CzOutcome;
    fn resize(self: &CzLinuxContainer, width: u16, height: u16) -> CzOutcome;
    fn exec(self: &CzLinuxContainer, id: &str, configuration: RustLinuxProcessConfiguration) -> CzOutcome;
    #[swift_bridge(swift_name = "closeStdin")]
    fn close_stdin(self: &CzLinuxContainer) -> CzOutcome;

    type CzLinuxProcess;
    fn id(self: &CzLinuxProcess) -> String;
    fn pid(self: &CzLinuxProcess) -> i32;
    fn start(self: &CzLinuxProcess) -> CzOutcome;
    fn kill(self: &CzLinuxProcess, signal: i32) -> CzOutcome;
    fn resize(self: &CzLinuxProcess, width: u16, height: u16) -> CzOutcome;
    #[swift_bridge(swift_name = "closeStdin")]
    fn close_stdin(self: &CzLinuxProcess) -> CzOutcome;
    fn wait(
      self: &CzLinuxProcess,
      #[swift_bridge(label = "timeoutInSeconds")] timeout_in_seconds: Option<i64>,
    ) -> CzOutcome;
    fn delete(self: &CzLinuxProcess) -> CzOutcome;
  }
}
