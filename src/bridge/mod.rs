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
use crate::containerization::LinuxRLimit as RustLinuxRLimit;
use crate::containerization::Mount as RustMount;
use crate::containerization::NatInterface as RustNatInterface;
use crate::containerization::SystemPlatform as RustSystemPlatform;
use crate::containerization::UnixSocketConfiguration as RustUnixSocketConfiguration;
use crate::containerization::container_manager::CreateOptions as RustCreateOptions;
use crate::containerization::container_manager::RootfsCreateOptions as RustRootfsCreateOptions;
use crate::containerization::hosts::Entry as RustHostsEntry;
use crate::containerization::image::Description as RustImageDescription;
use crate::containerization::linux_container::Configuration as RustLinuxContainerConfiguration;
use crate::containerization_archive::ArchiveWriterConfiguration as RustArchiveWriterConfiguration;
use crate::containerization_ext4::ext4::formatter::FormatterOptions as RustFormatterOptions;
use crate::containerization_extras::IPv6Address as RustIPv6Address;
use crate::containerization_extras::IpAddress as RustIpAddress;
use crate::containerization_oci::Descriptor as RustDescriptor;
use crate::containerization_oci::Hook as RustHook;
use crate::containerization_oci::Hooks as RustHooks;
use crate::containerization_oci::ImageConfig as RustImageConfig;
use crate::containerization_oci::Linux as RustLinux;
use crate::containerization_oci::LinuxBlockIO as RustLinuxBlockIO;
use crate::containerization_oci::LinuxCPU as RustLinuxCPU;
use crate::containerization_oci::LinuxCapabilities as RustOciLinuxCapabilities;
use crate::containerization_oci::LinuxDevice as RustLinuxDevice;
use crate::containerization_oci::LinuxDeviceCgroup as RustLinuxDeviceCgroup;
use crate::containerization_oci::LinuxIDMapping as RustLinuxIDMapping;
use crate::containerization_oci::LinuxMemory as RustLinuxMemory;
use crate::containerization_oci::LinuxResources as RustLinuxResources;
use crate::containerization_oci::LinuxSeccomp as RustLinuxSeccomp;
use crate::containerization_oci::LinuxSyscall as RustLinuxSyscall;
use crate::containerization_oci::Mount as RustOciMount;
use crate::containerization_oci::Platform as RustPlatform;
use crate::containerization_oci::Process as RustProcess;
use crate::containerization_oci::Spec as RustSpec;
use crate::containerization_oci::User as RustUser;
use crate::platform::ConfigureContainer as RustConfigure;
use crate::platform::ConfigureProcess as RustConfigureProcess;
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

  // `SeccompProfile`, less a custom profile.
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

  // `Options`, less its payload, with `compression`'s cases spelled out.
  enum ArchiveOptionKind {
    CompressionLevel,
    CompressionStore,
    CompressionDeflate,
    Xattrformat,
  }

  // The lists of `Hooks`.
  enum HookKind {
    Prestart,
    CreateRuntime,
    CreateContainer,
    StartContainer,
    Poststart,
    Poststop,
  }

  // The throttle lists of `LinuxBlockIO`.
  enum ThrottleKind {
    ReadBps,
    WriteBps,
    ReadIops,
    WriteIops,
  }

  // swift-log's `Logger.Level`.
  enum LogLevel {
    Trace,
    Debug,
    Info,
    Notice,
    Warning,
    Error,
    Critical,
  }

  // `Hosts.Entry`'s static constructors.
  enum HostsEntryName {
    LocalHostIpv4,
    LocalHostIpv6,
    Ipv6LocalNet,
    Ipv6MulticastPrefix,
    Ipv6AllNodes,
    Ipv6AllRouters,
  }

  extern "Rust" {
    // A `(inout LinuxContainer.Configuration) -> Void`: Swift calls it once,
    // with the configuration it seeded.
    type RustConfigure;
    fn call(self: &RustConfigure, configuration: &mut RustLinuxContainerConfiguration);

    // A `(inout LinuxProcessConfiguration) -> Void`: Swift calls it once,
    // with the configuration it filled.
    type RustConfigureProcess;
    fn call(self: &RustConfigureProcess, process: &mut RustLinuxProcessConfiguration);

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

    type RustFormatterOptions;
    #[swift_bridge(swift_name = "blockSize")]
    fn block_size(self: &RustFormatterOptions) -> u32;
    #[swift_bridge(swift_name = "minDiskSize")]
    fn min_disk_size(self: &RustFormatterOptions) -> u64;
    #[swift_bridge(swift_name = "hasJournal")]
    fn has_journal(self: &RustFormatterOptions) -> bool;
    #[swift_bridge(swift_name = "journalSize")]
    fn journal_size(self: &RustFormatterOptions) -> Option<u64>;
    #[swift_bridge(swift_name = "hasJournalMode")]
    fn has_journal_mode(self: &RustFormatterOptions) -> bool;
    #[swift_bridge(swift_name = "journalMode")]
    fn journal_mode(self: &RustFormatterOptions) -> JournalModeKind;

    // Its enums are their `rawValue`s. An option's level and format are
    // read only for the kinds that have them.
    type RustArchiveWriterConfiguration;
    fn format(self: &RustArchiveWriterConfiguration) -> &str;
    fn filter(self: &RustArchiveWriterConfiguration) -> &str;
    #[swift_bridge(swift_name = "optionsLen")]
    fn options_len(self: &RustArchiveWriterConfiguration) -> usize;
    #[swift_bridge(swift_name = "optionKindAt")]
    fn option_kind_at(self: &RustArchiveWriterConfiguration, index: usize) -> ArchiveOptionKind;
    #[swift_bridge(swift_name = "optionCompressionLevelAt")]
    fn option_compression_level_at(self: &RustArchiveWriterConfiguration, index: usize) -> u32;
    #[swift_bridge(swift_name = "optionXattrFormatAt")]
    fn option_xattr_format_at(self: &RustArchiveWriterConfiguration, index: usize) -> &str;
    #[swift_bridge(swift_name = "localesLen")]
    fn locales_len(self: &RustArchiveWriterConfiguration) -> usize;
    #[swift_bridge(swift_name = "localesAt")]
    fn locales_at(self: &RustArchiveWriterConfiguration, index: usize) -> &str;

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
    #[swift_bridge(swift_name = "ipv4AddressValue")]
    fn ipv4_address_value(self: &RustNatInterface) -> u32;
    #[swift_bridge(swift_name = "ipv4Prefix")]
    fn ipv4_prefix(self: &RustNatInterface) -> u8;
    #[swift_bridge(swift_name = "ipv4Gateway")]
    fn ipv4_gateway(self: &RustNatInterface) -> Option<u32>;
    #[swift_bridge(swift_name = "hasIpv6Address")]
    fn has_ipv6_address(self: &RustNatInterface) -> bool;
    #[swift_bridge(swift_name = "ipv6Address")]
    fn ipv6_address(self: &RustNatInterface) -> &RustIPv6Address;
    #[swift_bridge(swift_name = "ipv6Prefix")]
    fn ipv6_prefix(self: &RustNatInterface) -> u8;
    #[swift_bridge(swift_name = "hasIpv6Gateway")]
    fn has_ipv6_gateway(self: &RustNatInterface) -> bool;
    #[swift_bridge(swift_name = "ipv6Gateway")]
    fn ipv6_gateway(self: &RustNatInterface) -> &RustIPv6Address;
    #[swift_bridge(swift_name = "macAddress")]
    fn mac_address(self: &RustNatInterface) -> Option<u64>;
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
      ipv4_address: u32,
      ipv4_prefix: u8,
      ipv4_gateway: Option<u32>,
      mac_address: Option<u64>,
      mtu: u32,
    );
    #[swift_bridge(swift_name = "setInterfaceIpv6Address")]
    fn set_interface_ipv6_address(
      self: &mut RustLinuxContainerConfiguration,
      high: u64,
      low: u64,
      zone: Option<String>,
      prefix: u8,
    );
    #[swift_bridge(swift_name = "setInterfaceIpv6Gateway")]
    fn set_interface_ipv6_gateway(
      self: &mut RustLinuxContainerConfiguration,
      high: u64,
      low: u64,
      zone: Option<String>,
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
    fn set_seccomp_profile(self: &mut RustLinuxContainerConfiguration, mode: SeccompMode, profile: CzOutcome);
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
    fn seccomp_profile(self: &RustLinuxContainerConfiguration) -> &RustLinuxSeccomp;
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

    type RustLinuxRLimit;
    fn kind(self: &RustLinuxRLimit) -> RlimitKind;
    fn hard(self: &RustLinuxRLimit) -> u64;
    fn soft(self: &RustLinuxRLimit) -> u64;

    type RustImageConfig;
    fn user(self: &RustImageConfig) -> Option<&str>;
    #[swift_bridge(swift_name = "hasEnv")]
    fn has_env(self: &RustImageConfig) -> bool;
    #[swift_bridge(swift_name = "envLen")]
    fn env_len(self: &RustImageConfig) -> usize;
    #[swift_bridge(swift_name = "envAt")]
    fn env_at(self: &RustImageConfig, index: usize) -> &str;
    #[swift_bridge(swift_name = "hasEntrypoint")]
    fn has_entrypoint(self: &RustImageConfig) -> bool;
    #[swift_bridge(swift_name = "entrypointLen")]
    fn entrypoint_len(self: &RustImageConfig) -> usize;
    #[swift_bridge(swift_name = "entrypointAt")]
    fn entrypoint_at(self: &RustImageConfig, index: usize) -> &str;
    #[swift_bridge(swift_name = "hasCmd")]
    fn has_cmd(self: &RustImageConfig) -> bool;
    #[swift_bridge(swift_name = "cmdLen")]
    fn cmd_len(self: &RustImageConfig) -> usize;
    #[swift_bridge(swift_name = "cmdAt")]
    fn cmd_at(self: &RustImageConfig, index: usize) -> &str;
    #[swift_bridge(swift_name = "workingDir")]
    fn working_dir(self: &RustImageConfig) -> Option<&str>;
    #[swift_bridge(swift_name = "hasLabels")]
    fn has_labels(self: &RustImageConfig) -> bool;
    #[swift_bridge(swift_name = "labelsLen")]
    fn labels_len(self: &RustImageConfig) -> usize;
    #[swift_bridge(swift_name = "labelKeyAt")]
    fn label_key_at(self: &RustImageConfig, index: usize) -> &str;
    #[swift_bridge(swift_name = "labelValueAt")]
    fn label_value_at(self: &RustImageConfig, index: usize) -> &str;
    #[swift_bridge(swift_name = "stopSignal")]
    fn stop_signal(self: &RustImageConfig) -> Option<&str>;

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
    #[swift_bridge(swift_name = "archiveWriter")]
    fn archive_writer(self: &CzOutcome) -> CzArchiveWriter;
    #[swift_bridge(swift_name = "archiveReader")]
    fn archive_reader(self: &CzOutcome) -> CzArchiveReader;
    // A `WriteEntry`, alone or with the data or reader an iterator returned
    // it with.
    #[swift_bridge(swift_name = "writeEntry")]
    fn write_entry(self: &CzOutcome) -> CzWriteEntry;
    #[swift_bridge(swift_name = "entryData")]
    fn entry_data(self: &CzOutcome) -> Vec<u8>;
    #[swift_bridge(swift_name = "archiveEntryReader")]
    fn archive_entry_reader(self: &CzOutcome) -> CzArchiveEntryReader;
    // A held `[String: Data]`'s keys, and the value of one.
    #[swift_bridge(swift_name = "dataMapKeys")]
    fn data_map_keys(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "dataMapValue")]
    fn data_map_value(self: &CzOutcome, key: &str) -> Vec<u8>;

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

    // A held list's length and elements, and a held `[String: String]`'s keys
    // and values in the same order.
    fn len(self: &CzOutcome) -> usize;
    fn at(self: &CzOutcome, index: usize) -> CzOutcome;
    #[swift_bridge(swift_name = "mapKeys")]
    fn map_keys(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "mapValues")]
    fn map_values(self: &CzOutcome) -> Vec<String>;

    // The OCI image types, field by field. A list, a map, a nested value or
    // an optional one of those is an outcome of its own.
    #[swift_bridge(swift_name = "descriptorMediaType")]
    fn descriptor_media_type(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "descriptorDigest")]
    fn descriptor_digest(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "descriptorSize")]
    fn descriptor_size(self: &CzOutcome) -> i64;
    #[swift_bridge(swift_name = "descriptorUrls")]
    fn descriptor_urls(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "descriptorAnnotations")]
    fn descriptor_annotations(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "descriptorPlatform")]
    fn descriptor_platform(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "descriptorArtifactType")]
    fn descriptor_artifact_type(self: &CzOutcome) -> Option<String>;

    #[swift_bridge(swift_name = "indexSchemaVersion")]
    fn index_schema_version(self: &CzOutcome) -> isize;
    #[swift_bridge(swift_name = "indexMediaType")]
    fn index_media_type(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "indexManifests")]
    fn index_manifests(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "indexAnnotations")]
    fn index_annotations(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "indexSubject")]
    fn index_subject(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "indexArtifactType")]
    fn index_artifact_type(self: &CzOutcome) -> Option<String>;

    #[swift_bridge(swift_name = "manifestSchemaVersion")]
    fn manifest_schema_version(self: &CzOutcome) -> isize;
    #[swift_bridge(swift_name = "manifestMediaType")]
    fn manifest_media_type(self: &CzOutcome) -> Option<String>;
    #[swift_bridge(swift_name = "manifestConfig")]
    fn manifest_config(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "manifestLayers")]
    fn manifest_layers(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "manifestAnnotations")]
    fn manifest_annotations(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "manifestSubject")]
    fn manifest_subject(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "manifestArtifactType")]
    fn manifest_artifact_type(self: &CzOutcome) -> Option<String>;

    #[swift_bridge(swift_name = "imageConfigUser")]
    fn image_config_user(self: &CzOutcome) -> Option<String>;
    #[swift_bridge(swift_name = "imageConfigEnv")]
    fn image_config_env(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "imageConfigEntrypoint")]
    fn image_config_entrypoint(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "imageConfigCmd")]
    fn image_config_cmd(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "imageConfigWorkingDir")]
    fn image_config_working_dir(self: &CzOutcome) -> Option<String>;
    #[swift_bridge(swift_name = "imageConfigLabels")]
    fn image_config_labels(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "imageConfigStopSignal")]
    fn image_config_stop_signal(self: &CzOutcome) -> Option<String>;

    #[swift_bridge(swift_name = "rootfsType")]
    fn rootfs_type(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "rootfsDiffIDs")]
    fn rootfs_diff_ids(self: &CzOutcome) -> Vec<String>;

    #[swift_bridge(swift_name = "historyCreated")]
    fn history_created(self: &CzOutcome) -> Option<String>;
    #[swift_bridge(swift_name = "historyCreatedBy")]
    fn history_created_by(self: &CzOutcome) -> Option<String>;
    #[swift_bridge(swift_name = "historyAuthor")]
    fn history_author(self: &CzOutcome) -> Option<String>;
    #[swift_bridge(swift_name = "historyComment")]
    fn history_comment(self: &CzOutcome) -> Option<String>;
    #[swift_bridge(swift_name = "historyEmptyLayer")]
    fn history_empty_layer(self: &CzOutcome) -> Option<bool>;

    #[swift_bridge(swift_name = "ociImageCreated")]
    fn oci_image_created(self: &CzOutcome) -> Option<String>;
    #[swift_bridge(swift_name = "ociImageAuthor")]
    fn oci_image_author(self: &CzOutcome) -> Option<String>;
    #[swift_bridge(swift_name = "ociImageArchitecture")]
    fn oci_image_architecture(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "ociImageOs")]
    fn oci_image_os(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "ociImageOsVersion")]
    fn oci_image_os_version(self: &CzOutcome) -> Option<String>;
    #[swift_bridge(swift_name = "ociImageOsFeatures")]
    fn oci_image_os_features(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "ociImageVariant")]
    fn oci_image_variant(self: &CzOutcome) -> Option<String>;
    #[swift_bridge(swift_name = "ociImageConfig")]
    fn oci_image_config(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "ociImageRootfs")]
    fn oci_image_rootfs(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "ociImageHistory")]
    fn oci_image_history(self: &CzOutcome) -> CzOutcome;

    // A held `[String: T]`'s keys, sorted, and an outcome holding its values
    // in the same order.
    #[swift_bridge(swift_name = "entryKeys")]
    fn entry_keys(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "entryValues")]
    fn entry_values(self: &CzOutcome) -> CzOutcome;

    // The OCI runtime spec, field by field. An enum is its `rawValue`.
    #[swift_bridge(swift_name = "specVersion")]
    fn spec_version(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "specHooks")]
    fn spec_hooks(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "specProcess")]
    fn spec_process(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "specHostname")]
    fn spec_hostname(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "specDomainname")]
    fn spec_domainname(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "specMounts")]
    fn spec_mounts(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "specAnnotations")]
    fn spec_annotations(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "specRoot")]
    fn spec_root(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "specLinux")]
    fn spec_linux(self: &CzOutcome) -> CzOutcome;

    #[swift_bridge(swift_name = "processCwd")]
    fn process_cwd(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "processEnv")]
    fn process_env(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "processConsoleSize")]
    fn process_console_size(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "processSelinuxLabel")]
    fn process_selinux_label(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "processNoNewPrivileges")]
    fn process_no_new_privileges(self: &CzOutcome) -> bool;
    #[swift_bridge(swift_name = "processCommandLine")]
    fn process_command_line(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "processOomScoreAdj")]
    fn process_oom_score_adj(self: &CzOutcome) -> Option<isize>;
    #[swift_bridge(swift_name = "processCapabilities")]
    fn process_capabilities(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "processApparmorProfile")]
    fn process_apparmor_profile(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "processUser")]
    fn process_user(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "processRlimits")]
    fn process_rlimits(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "processArgs")]
    fn process_args(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "processTerminal")]
    fn process_terminal(self: &CzOutcome) -> bool;

    // `Box`'s fields are internal, so Swift reads them by reflection.
    #[swift_bridge(swift_name = "boxHeight")]
    fn box_height(self: &CzOutcome) -> usize;
    #[swift_bridge(swift_name = "boxWidth")]
    fn box_width(self: &CzOutcome) -> usize;

    #[swift_bridge(swift_name = "userUid")]
    fn user_uid(self: &CzOutcome) -> u32;
    #[swift_bridge(swift_name = "userGid")]
    fn user_gid(self: &CzOutcome) -> u32;
    #[swift_bridge(swift_name = "userUmask")]
    fn user_umask(self: &CzOutcome) -> Option<u32>;
    #[swift_bridge(swift_name = "userAdditionalGids")]
    fn user_additional_gids(self: &CzOutcome) -> Vec<u32>;
    #[swift_bridge(swift_name = "userUsername")]
    fn user_username(self: &CzOutcome) -> String;

    #[swift_bridge(swift_name = "capabilitiesSet")]
    fn capabilities_set(self: &CzOutcome, set: CapabilitySet) -> CzOutcome;

    #[swift_bridge(swift_name = "rootPath")]
    fn root_path(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "rootReadonly")]
    fn root_readonly(self: &CzOutcome) -> bool;

    #[swift_bridge(swift_name = "ociMountType")]
    fn oci_mount_type(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "ociMountSource")]
    fn oci_mount_source(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "ociMountDestination")]
    fn oci_mount_destination(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "ociMountOptions")]
    fn oci_mount_options(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "ociMountUidMappings")]
    fn oci_mount_uid_mappings(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "ociMountGidMappings")]
    fn oci_mount_gid_mappings(self: &CzOutcome) -> CzOutcome;

    #[swift_bridge(swift_name = "hookPath")]
    fn hook_path(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "hookArgs")]
    fn hook_args(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "hookEnv")]
    fn hook_env(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "hookTimeout")]
    fn hook_timeout(self: &CzOutcome) -> Option<isize>;
    #[swift_bridge(swift_name = "hooksOf")]
    fn hooks_of(self: &CzOutcome, kind: HookKind) -> CzOutcome;

    #[swift_bridge(swift_name = "linuxUidMappings")]
    fn linux_uid_mappings(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "linuxGidMappings")]
    fn linux_gid_mappings(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "linuxSysctl")]
    fn linux_sysctl(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "linuxResources")]
    fn linux_resources(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "linuxCgroupsPath")]
    fn linux_cgroups_path(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "linuxNamespaces")]
    fn linux_namespaces(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "linuxDevices")]
    fn linux_devices(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "linuxSeccomp")]
    fn linux_seccomp(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "linuxRootfsPropagation")]
    fn linux_rootfs_propagation(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "linuxMaskedPaths")]
    fn linux_masked_paths(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "linuxReadonlyPaths")]
    fn linux_readonly_paths(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "linuxMountLabel")]
    fn linux_mount_label(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "linuxPersonality")]
    fn linux_personality(self: &CzOutcome) -> CzOutcome;

    #[swift_bridge(swift_name = "namespaceType")]
    fn namespace_type(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "namespacePath")]
    fn namespace_path(self: &CzOutcome) -> String;

    #[swift_bridge(swift_name = "idMappingContainerID")]
    fn id_mapping_container_id(self: &CzOutcome) -> u32;
    #[swift_bridge(swift_name = "idMappingHostID")]
    fn id_mapping_host_id(self: &CzOutcome) -> u32;
    #[swift_bridge(swift_name = "idMappingSize")]
    fn id_mapping_size(self: &CzOutcome) -> u32;

    #[swift_bridge(swift_name = "rlimitType")]
    fn rlimit_type(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "rlimitHard")]
    fn rlimit_hard(self: &CzOutcome) -> u64;
    #[swift_bridge(swift_name = "rlimitSoft")]
    fn rlimit_soft(self: &CzOutcome) -> u64;

    #[swift_bridge(swift_name = "resourcesDevices")]
    fn resources_devices(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "resourcesMemory")]
    fn resources_memory(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "resourcesCpu")]
    fn resources_cpu(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "resourcesPids")]
    fn resources_pids(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "resourcesBlockIO")]
    fn resources_block_io(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "resourcesHugepageLimits")]
    fn resources_hugepage_limits(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "resourcesNetwork")]
    fn resources_network(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "resourcesRdma")]
    fn resources_rdma(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "resourcesUnified")]
    fn resources_unified(self: &CzOutcome) -> CzOutcome;

    #[swift_bridge(swift_name = "memoryLimit")]
    fn memory_limit(self: &CzOutcome) -> Option<i64>;
    #[swift_bridge(swift_name = "memoryReservation")]
    fn memory_reservation(self: &CzOutcome) -> Option<i64>;
    #[swift_bridge(swift_name = "memorySwap")]
    fn memory_swap(self: &CzOutcome) -> Option<i64>;
    #[swift_bridge(swift_name = "memoryKernel")]
    fn memory_kernel(self: &CzOutcome) -> Option<i64>;
    #[swift_bridge(swift_name = "memoryKernelTCP")]
    fn memory_kernel_tcp(self: &CzOutcome) -> Option<i64>;
    #[swift_bridge(swift_name = "memorySwappiness")]
    fn memory_swappiness(self: &CzOutcome) -> Option<u64>;
    #[swift_bridge(swift_name = "memoryDisableOOMKiller")]
    fn memory_disable_oom_killer(self: &CzOutcome) -> Option<bool>;
    #[swift_bridge(swift_name = "memoryUseHierarchy")]
    fn memory_use_hierarchy(self: &CzOutcome) -> Option<bool>;
    #[swift_bridge(swift_name = "memoryCheckBeforeUpdate")]
    fn memory_check_before_update(self: &CzOutcome) -> Option<bool>;

    #[swift_bridge(swift_name = "cpuShares")]
    fn cpu_shares(self: &CzOutcome) -> Option<u64>;
    #[swift_bridge(swift_name = "cpuQuota")]
    fn cpu_quota(self: &CzOutcome) -> Option<i64>;
    #[swift_bridge(swift_name = "cpuBurst")]
    fn cpu_burst(self: &CzOutcome) -> Option<u64>;
    #[swift_bridge(swift_name = "cpuPeriod")]
    fn cpu_period(self: &CzOutcome) -> Option<u64>;
    #[swift_bridge(swift_name = "cpuRealtimeRuntime")]
    fn cpu_realtime_runtime(self: &CzOutcome) -> Option<i64>;
    #[swift_bridge(swift_name = "cpuRealtimePeriod")]
    fn cpu_realtime_period(self: &CzOutcome) -> Option<i64>;
    #[swift_bridge(swift_name = "cpuCpus")]
    fn cpu_cpus(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "cpuMems")]
    fn cpu_mems(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "cpuIdle")]
    fn cpu_idle(self: &CzOutcome) -> Option<i64>;

    #[swift_bridge(swift_name = "pidsLimit")]
    fn pids_limit(self: &CzOutcome) -> i64;

    #[swift_bridge(swift_name = "blockIOWeight")]
    fn block_io_weight(self: &CzOutcome) -> Option<u16>;
    #[swift_bridge(swift_name = "blockIOLeafWeight")]
    fn block_io_leaf_weight(self: &CzOutcome) -> Option<u16>;
    #[swift_bridge(swift_name = "blockIOWeightDevice")]
    fn block_io_weight_device(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "blockIOThrottle")]
    fn block_io_throttle(self: &CzOutcome, kind: ThrottleKind) -> CzOutcome;
    #[swift_bridge(swift_name = "weightDeviceMajor")]
    fn weight_device_major(self: &CzOutcome) -> i64;
    #[swift_bridge(swift_name = "weightDeviceMinor")]
    fn weight_device_minor(self: &CzOutcome) -> i64;
    #[swift_bridge(swift_name = "weightDeviceWeight")]
    fn weight_device_weight(self: &CzOutcome) -> Option<u16>;
    #[swift_bridge(swift_name = "weightDeviceLeafWeight")]
    fn weight_device_leaf_weight(self: &CzOutcome) -> Option<u16>;
    #[swift_bridge(swift_name = "throttleDeviceMajor")]
    fn throttle_device_major(self: &CzOutcome) -> i64;
    #[swift_bridge(swift_name = "throttleDeviceMinor")]
    fn throttle_device_minor(self: &CzOutcome) -> i64;
    #[swift_bridge(swift_name = "throttleDeviceRate")]
    fn throttle_device_rate(self: &CzOutcome) -> u64;

    #[swift_bridge(swift_name = "hugepageLimitPagesize")]
    fn hugepage_limit_pagesize(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "hugepageLimitLimit")]
    fn hugepage_limit_limit(self: &CzOutcome) -> u64;

    #[swift_bridge(swift_name = "networkClassID")]
    fn network_class_id(self: &CzOutcome) -> Option<u32>;
    #[swift_bridge(swift_name = "networkPriorities")]
    fn network_priorities(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "interfacePriorityName")]
    fn interface_priority_name(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "interfacePriorityPriority")]
    fn interface_priority_priority(self: &CzOutcome) -> u32;

    #[swift_bridge(swift_name = "rdmaHcsHandles")]
    fn rdma_hcs_handles(self: &CzOutcome) -> Option<u32>;
    #[swift_bridge(swift_name = "rdmaHcaObjects")]
    fn rdma_hca_objects(self: &CzOutcome) -> Option<u32>;

    #[swift_bridge(swift_name = "devicePath")]
    fn device_path(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "deviceType")]
    fn device_type(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "deviceMajor")]
    fn device_major(self: &CzOutcome) -> i64;
    #[swift_bridge(swift_name = "deviceMinor")]
    fn device_minor(self: &CzOutcome) -> i64;
    #[swift_bridge(swift_name = "deviceFileMode")]
    fn device_file_mode(self: &CzOutcome) -> Option<u32>;
    #[swift_bridge(swift_name = "deviceUid")]
    fn device_uid(self: &CzOutcome) -> Option<u32>;
    #[swift_bridge(swift_name = "deviceGid")]
    fn device_gid(self: &CzOutcome) -> Option<u32>;

    #[swift_bridge(swift_name = "deviceCgroupAllow")]
    fn device_cgroup_allow(self: &CzOutcome) -> bool;
    #[swift_bridge(swift_name = "deviceCgroupType")]
    fn device_cgroup_type(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "deviceCgroupMajor")]
    fn device_cgroup_major(self: &CzOutcome) -> Option<i64>;
    #[swift_bridge(swift_name = "deviceCgroupMinor")]
    fn device_cgroup_minor(self: &CzOutcome) -> Option<i64>;
    #[swift_bridge(swift_name = "deviceCgroupAccess")]
    fn device_cgroup_access(self: &CzOutcome) -> Option<String>;

    #[swift_bridge(swift_name = "personalityDomain")]
    fn personality_domain(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "personalityFlags")]
    fn personality_flags(self: &CzOutcome) -> Vec<String>;

    #[swift_bridge(swift_name = "seccompDefaultAction")]
    fn seccomp_default_action(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "seccompDefaultErrnoRet")]
    fn seccomp_default_errno_ret(self: &CzOutcome) -> Option<usize>;
    #[swift_bridge(swift_name = "seccompArchitectures")]
    fn seccomp_architectures(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "seccompFlags")]
    fn seccomp_flags(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "seccompListenerPath")]
    fn seccomp_listener_path(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "seccompListenerMetadata")]
    fn seccomp_listener_metadata(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "seccompSyscalls")]
    fn seccomp_syscalls(self: &CzOutcome) -> CzOutcome;

    #[swift_bridge(swift_name = "syscallNames")]
    fn syscall_names(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "syscallAction")]
    fn syscall_action(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "syscallErrnoRet")]
    fn syscall_errno_ret(self: &CzOutcome) -> Option<usize>;
    #[swift_bridge(swift_name = "syscallArgs")]
    fn syscall_args(self: &CzOutcome) -> CzOutcome;
    #[swift_bridge(swift_name = "seccompArgIndex")]
    fn seccomp_arg_index(self: &CzOutcome) -> usize;
    #[swift_bridge(swift_name = "seccompArgValue")]
    fn seccomp_arg_value(self: &CzOutcome) -> u64;
    #[swift_bridge(swift_name = "seccompArgValueTwo")]
    fn seccomp_arg_value_two(self: &CzOutcome) -> u64;
    #[swift_bridge(swift_name = "seccompArgOp")]
    fn seccomp_arg_op(self: &CzOutcome) -> String;

    #[swift_bridge(swift_name = "runtimeSpecVersionMajor")]
    fn runtime_spec_version_major(self: &CzOutcome) -> isize;
    #[swift_bridge(swift_name = "runtimeSpecVersionMinor")]
    fn runtime_spec_version_minor(self: &CzOutcome) -> isize;
    #[swift_bridge(swift_name = "runtimeSpecVersionPatch")]
    fn runtime_spec_version_patch(self: &CzOutcome) -> isize;
    #[swift_bridge(swift_name = "runtimeSpecVersionDev")]
    fn runtime_spec_version_dev(self: &CzOutcome) -> String;

    #[swift_bridge(swift_name = "processFromImageConfig")]
    fn cz_process_from_image_config(config: RustImageConfig) -> CzOutcome;
    #[swift_bridge(swift_name = "processDescription")]
    fn cz_process_description(process: RustProcess) -> CzOutcome;
    #[swift_bridge(swift_name = "hookDescription")]
    fn cz_hook_description(hook: RustHook) -> CzOutcome;
    #[swift_bridge(swift_name = "decodeLinuxSeccomp")]
    fn cz_linux_seccomp_decode(data: Vec<u8>) -> CzOutcome;
    // `capabilities` stands for `nil` when `has_capabilities` is false.
    #[swift_bridge(swift_name = "defaultSeccompProfile")]
    fn cz_linux_seccomp_default_profile(
      #[swift_bridge(label = "hasCapabilities")] has_capabilities: bool,
      capabilities: RustOciLinuxCapabilities,
      arch: &str,
    ) -> CzOutcome;
    // The outcome holds the case's `rawValue`, or `Absent`.
    #[swift_bridge(swift_name = "currentArch")]
    fn cz_arch_current() -> CzOutcome;
    #[swift_bridge(swift_name = "currentVerifiedArch")]
    fn cz_arch_current_verified() -> CzOutcome;
    #[swift_bridge(swift_name = "currentRuntimeSpecVersion")]
    fn cz_runtime_spec_version_current() -> CzOutcome;
    #[swift_bridge(swift_name = "linuxRLimitToOCI")]
    fn cz_linux_rlimit_to_oci(rlimit: RustLinuxRLimit) -> CzOutcome;
    #[swift_bridge(swift_name = "linuxCapabilitiesToOCI")]
    fn cz_linux_capabilities_to_oci(capabilities: RustLinuxCapabilities) -> CzOutcome;
    #[swift_bridge(swift_name = "systemPlatformOCIPlatform")]
    fn cz_system_platform_oci_platform(platform: RustSystemPlatform) -> CzOutcome;

    // `Bundle`, as its path. Each outcome holds a path, except `loadConfig`'s,
    // which holds a `Spec`.
    #[swift_bridge(swift_name = "createBundle")]
    fn cz_bundle_create(path: &str, spec: RustSpec) -> CzOutcome;
    #[swift_bridge(swift_name = "createBundleFromData")]
    fn cz_bundle_create_from_data(path: &str, spec: Vec<u8>) -> CzOutcome;
    #[swift_bridge(swift_name = "loadBundle")]
    fn cz_bundle_load(path: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "bundleConfigPath")]
    fn cz_bundle_config_path(path: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "bundleRootfsPath")]
    fn cz_bundle_rootfs_path(path: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "deleteBundle")]
    fn cz_bundle_delete(path: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "bundleLoadConfig")]
    fn cz_bundle_load_config(path: &str) -> CzOutcome;

    // For unit tests: an enum's raw values in the order of Rust's `ALL`,
    // `seccompFdName`, and what a type's initializer makes with no arguments.
    #[swift_bridge(swift_name = "rawValues")]
    fn cz_raw_values(name: &str) -> Vec<String>;
    #[swift_bridge(swift_name = "seccompFdName")]
    fn cz_seccomp_fd_name() -> String;
    #[swift_bridge(swift_name = "defaultValue")]
    fn cz_default(name: &str) -> CzOutcome;

    // Swift's `MediaTypes` and `AnnotationKeys`, in the order of Rust's
    // `ALL`, for a test that compares Rust's copies with them.
    #[swift_bridge(swift_name = "mediaTypes")]
    fn cz_media_types() -> Vec<String>;
    #[swift_bridge(swift_name = "annotationKeys")]
    fn cz_annotation_keys() -> Vec<String>;

    type CzReference;
    #[swift_bridge(swift_name = "newReference")]
    fn cz_reference_new(path: &str, domain: Option<String>, tag: Option<String>, digest: Option<String>) -> CzOutcome;
    #[swift_bridge(swift_name = "parseReference")]
    fn cz_reference_parse(string: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "referenceWithName")]
    fn cz_reference_with_name(name: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "referenceResolveDomain")]
    fn cz_reference_resolve_domain(domain: &str) -> CzOutcome;
    fn reference(self: &CzOutcome) -> CzReference;
    fn domain(self: &CzReference) -> Option<String>;
    #[swift_bridge(swift_name = "resolvedDomain")]
    fn resolved_domain(self: &CzReference) -> Option<String>;
    fn path(self: &CzReference) -> String;
    fn tag(self: &CzReference) -> Option<String>;
    fn digest(self: &CzReference) -> Option<String>;
    fn name(self: &CzReference) -> String;
    fn description(self: &CzReference) -> String;
    #[swift_bridge(swift_name = "withTag")]
    fn with_tag(self: &CzReference, tag: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "withDigest")]
    fn with_digest(self: &CzReference, digest: &str) -> CzOutcome;
    fn normalize(self: &CzReference);

    // `ParsedDigest`, as its `encoded`.
    #[swift_bridge(swift_name = "parsedDigestEncoded")]
    fn parsed_digest_encoded(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "parseDigest")]
    fn cz_parsed_digest_parse(digest: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "parseDigestPathComponent")]
    fn cz_parsed_digest_parse_path_component(component: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "digestIsValid")]
    fn cz_parsed_digest_is_valid(digest: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "digestDescription")]
    fn cz_parsed_digest_description(encoded: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "digestPath")]
    fn cz_parsed_digest_path(encoded: &str, root: &str) -> CzOutcome;
    // For a test that compares Rust's copy with it.
    #[swift_bridge(swift_name = "digestAlgorithm")]
    fn cz_parsed_digest_algorithm() -> String;

    #[swift_bridge(swift_name = "currentPlatform")]
    fn cz_platform_current() -> CzOutcome;
    #[swift_bridge(swift_name = "parsePlatform")]
    fn cz_platform_parse(platform: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "platformDescription")]
    fn cz_platform_description(platform: RustPlatform) -> CzOutcome;
    #[swift_bridge(swift_name = "platformEquals")]
    fn cz_platform_equals(lhs: RustPlatform, rhs: RustPlatform) -> CzOutcome;
    #[swift_bridge(swift_name = "platformMatches")]
    fn cz_platform_matches(lhs: RustPlatform, rhs: RustPlatform) -> CzOutcome;

    // `ProgressEvent.event`, of the case and value Rust holds.
    #[swift_bridge(swift_name = "progressEventEvent")]
    fn cz_progress_event_event(kind: ProgressKind, value: i64) -> CzOutcome;

    // `ProxyUtils.proxyFromEnvironment(scheme:host:env:)`, with `env` as its
    // keys and values, or Swift's default when `has_env` is false. The
    // outcome holds the URL's `absoluteString`, or nothing.
    #[swift_bridge(swift_name = "proxyFromEnvironment")]
    fn cz_proxy_from_environment(
      scheme: Option<String>,
      host: &str,
      #[swift_bridge(label = "hasEnv")] has_env: bool,
      #[swift_bridge(label = "envKeys")] env_keys: Vec<String>,
      #[swift_bridge(label = "envValues")] env_values: Vec<String>,
    ) -> CzOutcome;

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

    // ContainerizationEXT4. A `SuperBlock` or an `Inode` crosses as its bytes,
    // and an archive's format and filter as their `rawValue`s.
    type CzExt4Reader;
    #[swift_bridge(swift_name = "openExt4Reader")]
    fn cz_ext4_reader_new(#[swift_bridge(label = "blockDevice")] block_device: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "superBlock")]
    fn super_block(self: &CzExt4Reader) -> Vec<u8>;
    fn exists(self: &CzExt4Reader, path: &str, #[swift_bridge(label = "followSymlinks")] follow_symlinks: bool)
    -> bool;
    // The outcome holds the inode's number and the inode.
    fn stat(
      self: &CzExt4Reader,
      path: &str,
      #[swift_bridge(label = "followSymlinks")] follow_symlinks: bool,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "listDirectory")]
    fn list_directory(self: &CzExt4Reader, path: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "readFile")]
    fn read_file(
      self: &CzExt4Reader,
      at: &str,
      offset: u64,
      count: Option<usize>,
      #[swift_bridge(label = "followSymlinks")] follow_symlinks: bool,
    ) -> CzOutcome;
    fn export(self: &CzExt4Reader, archive: &str) -> CzOutcome;
    // Each outcome holds the attributes, which Rust only counts.
    #[swift_bridge(swift_name = "readInlineExtendedAttributes")]
    fn cz_ext4_reader_read_inline_extended_attributes(buffer: Vec<u8>) -> CzOutcome;
    #[swift_bridge(swift_name = "readBlockExtendedAttributes")]
    fn cz_ext4_reader_read_block_extended_attributes(buffer: Vec<u8>) -> CzOutcome;
    #[swift_bridge(swift_name = "inodeNumber")]
    fn inode_number(self: &CzOutcome) -> u32;
    #[swift_bridge(swift_name = "inodeBytes")]
    fn inode_bytes(self: &CzOutcome) -> Vec<u8>;
    #[swift_bridge(swift_name = "rootInode")]
    fn cz_ext4_inode_root() -> CzOutcome;

    // `compressName`'s outcome holds the prefix id and the rest of the name.
    #[swift_bridge(swift_name = "compressExtendedAttributeName")]
    fn cz_ext4_extended_attribute_compress_name(name: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "compressedNameId")]
    fn compressed_name_id(self: &CzOutcome) -> u8;
    #[swift_bridge(swift_name = "compressedNameStr")]
    fn compressed_name_str(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "decompressExtendedAttributeName")]
    fn cz_ext4_extended_attribute_decompress_name(id: isize, suffix: &str) -> CzOutcome;

    // `create`'s timestamps cross as seconds since 1970, `buf` stands for
    // `nil` when `has_buf` is false, and so do the xattrs when `has_xattrs`
    // is. Their values are joined into one, with each one's length.
    type CzExt4Formatter;
    #[swift_bridge(swift_name = "newExt4Formatter")]
    fn cz_ext4_formatter_new(
      #[swift_bridge(label = "devicePath")] device_path: &str,
      options: RustFormatterOptions,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "ext4Formatter")]
    fn ext4_formatter(self: &CzOutcome) -> CzExt4Formatter;
    fn link(self: &CzExt4Formatter, link: &str, target: &str) -> CzOutcome;
    fn unlink(
      self: &CzExt4Formatter,
      path: &str,
      #[swift_bridge(label = "directoryWhiteout")] directory_whiteout: bool,
    ) -> CzOutcome;
    fn create(
      self: &CzExt4Formatter,
      path: &str,
      link: Option<String>,
      mode: u16,
      access: f64,
      modification: f64,
      creation: f64,
      now: f64,
      #[swift_bridge(label = "hasBuf")] has_buf: bool,
      buf: Vec<u8>,
      uid: Option<u32>,
      gid: Option<u32>,
      #[swift_bridge(label = "hasXattrs")] has_xattrs: bool,
      #[swift_bridge(label = "xattrNames")] xattr_names: Vec<String>,
      #[swift_bridge(label = "xattrLengths")] xattr_lengths: Vec<u64>,
      #[swift_bridge(label = "xattrValues")] xattr_values: Vec<u8>,
      recursion: bool,
    ) -> CzOutcome;
    fn close(self: &CzExt4Formatter) -> CzOutcome;
    fn unpack(
      self: &CzExt4Formatter,
      source: &str,
      format: &str,
      compression: &str,
      progress: RustProgressHandler,
    ) -> CzOutcome;
    // `unpack(reader:progress:)`, with the reader as a `duplicate()`.
    #[swift_bridge(swift_name = "unpackReader")]
    fn unpack_reader(self: &CzExt4Formatter, reader: CzArchiveReader, progress: RustProgressHandler) -> CzOutcome;
    // The outcome holds the size and the number of items.
    #[swift_bridge(swift_name = "scanArchiveHeaders")]
    fn cz_ext4_formatter_scan_archive_headers(format: &str, filter: &str, file: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "scannedSize")]
    fn scanned_size(self: &CzOutcome) -> i64;
    #[swift_bridge(swift_name = "scannedItems")]
    fn scanned_items(self: &CzOutcome) -> isize;

    // For unit tests: each `FileModeFlag` as `Inode.Mode(flag, 0)`, in the
    // order of Rust's `ALL`; `ExtendedAttribute.prefixMap`, sorted by key;
    // `SuperBlockMagic`; the sizes of `SuperBlock` and `Inode` and the
    // offsets of their last fields; and a `FileTimestamps`' `accessLo` and
    // `accessHi` for a date.
    #[swift_bridge(swift_name = "ext4FileModeFlags")]
    fn cz_ext4_file_mode_flags() -> Vec<u16>;
    #[swift_bridge(swift_name = "ext4PrefixMapKeys")]
    fn cz_ext4_prefix_map_keys() -> Vec<isize>;
    #[swift_bridge(swift_name = "ext4PrefixMapValues")]
    fn cz_ext4_prefix_map_values() -> Vec<String>;
    #[swift_bridge(swift_name = "ext4SuperBlockMagic")]
    fn cz_ext4_super_block_magic() -> u16;
    #[swift_bridge(swift_name = "ext4Layout")]
    fn cz_ext4_layout() -> Vec<u64>;
    #[swift_bridge(swift_name = "fileTimestampsAccess")]
    fn cz_file_timestamps_access(seconds: f64) -> Vec<u32>;

    // ContainerizationIO. `next`'s outcome holds the next chunk, or `Absent`.
    type CzReadStream;
    #[swift_bridge(swift_name = "newReadStream")]
    fn cz_read_stream_new() -> CzOutcome;
    #[swift_bridge(swift_name = "readStreamWithURL")]
    fn cz_read_stream_with_url(url: &str, #[swift_bridge(label = "bufferSize")] buffer_size: usize) -> CzOutcome;
    #[swift_bridge(swift_name = "readStreamWithData")]
    fn cz_read_stream_with_data(data: Vec<u8>, #[swift_bridge(label = "bufferSize")] buffer_size: usize) -> CzOutcome;
    #[swift_bridge(swift_name = "readStream")]
    fn read_stream(self: &CzOutcome) -> CzReadStream;
    fn reset(self: &CzReadStream) -> CzOutcome;
    #[swift_bridge(swift_name = "dataStream")]
    fn data_stream(self: &CzReadStream) -> CzDataStream;
    type CzDataStream;
    fn next(self: &CzDataStream) -> CzOutcome;
    // For a unit test: `ReadStream.bufferSize`.
    #[swift_bridge(swift_name = "readStreamBufferSize")]
    fn cz_read_stream_buffer_size() -> isize;

    // `EXT4Unpacker.unpack(archive:compression:at:)`, with the filter as its
    // `rawValue`.
    #[swift_bridge(swift_name = "unpackExt4Archive")]
    fn cz_ext4_unpack_archive(unpacker: RustExt4Unpacker, archive: &str, compression: &str, at: &str) -> CzOutcome;

    // ContainerizationArchive. Enums cross as their `rawValue`s, and a
    // `WriteEntry` argument as a `duplicate()`.
    #[swift_bridge(swift_name = "xattrFormatDescription")]
    fn cz_xattr_format_description(format: &str) -> CzOutcome;
    // For a unit test: an enum's raw values in the order of Rust's `ALL`, and
    // `ArchiveWriterConfiguration.defaultLocales`.
    #[swift_bridge(swift_name = "archiveRawValues")]
    fn cz_archive_raw_values(name: &str) -> Vec<String>;
    #[swift_bridge(swift_name = "archiveDefaultLocales")]
    fn cz_archive_default_locales() -> Vec<String>;

    type CzWriteEntry;
    #[swift_bridge(swift_name = "newWriteEntry")]
    fn cz_write_entry_new() -> CzOutcome;
    fn duplicate(self: &CzWriteEntry) -> CzWriteEntry;
    #[swift_bridge(swift_name = "hasSize")]
    fn has_size(self: &CzWriteEntry) -> bool;
    fn size(self: &CzWriteEntry) -> i64;
    #[swift_bridge(swift_name = "setSize")]
    fn set_size(self: &CzWriteEntry, #[swift_bridge(label = "isSet")] is_set: bool, size: i64);
    fn permissions(self: &CzWriteEntry) -> u16;
    #[swift_bridge(swift_name = "setPermissions")]
    fn set_permissions(self: &CzWriteEntry, permissions: u16);
    #[swift_bridge(swift_name = "hasOwner")]
    fn has_owner(self: &CzWriteEntry) -> bool;
    fn owner(self: &CzWriteEntry) -> u32;
    #[swift_bridge(swift_name = "setOwner")]
    fn set_owner(self: &CzWriteEntry, #[swift_bridge(label = "isSet")] is_set: bool, owner: u32);
    #[swift_bridge(swift_name = "hasGroup")]
    fn has_group(self: &CzWriteEntry) -> bool;
    fn group(self: &CzWriteEntry) -> u32;
    #[swift_bridge(swift_name = "setGroup")]
    fn set_group(self: &CzWriteEntry, #[swift_bridge(label = "isSet")] is_set: bool, group: u32);
    fn hardlink(self: &CzWriteEntry) -> Option<String>;
    #[swift_bridge(swift_name = "setHardlink")]
    fn set_hardlink(self: &CzWriteEntry, hardlink: Option<String>);
    #[swift_bridge(swift_name = "hardlinkUtf8")]
    fn hardlink_utf8(self: &CzWriteEntry) -> Option<String>;
    #[swift_bridge(swift_name = "setHardlinkUtf8")]
    fn set_hardlink_utf8(self: &CzWriteEntry, hardlink: Option<String>);
    fn strmode(self: &CzWriteEntry) -> Option<String>;
    #[swift_bridge(swift_name = "fileType")]
    fn file_type(self: &CzWriteEntry) -> String;
    #[swift_bridge(swift_name = "setFileType")]
    fn set_file_type(self: &CzWriteEntry, #[swift_bridge(label = "fileType")] file_type: &str);
    // Dates cross as seconds since 1970.
    #[swift_bridge(swift_name = "hasContentAccessDate")]
    fn has_content_access_date(self: &CzWriteEntry) -> bool;
    #[swift_bridge(swift_name = "contentAccessDate")]
    fn content_access_date(self: &CzWriteEntry) -> f64;
    #[swift_bridge(swift_name = "setContentAccessDate")]
    fn set_content_access_date(self: &CzWriteEntry, #[swift_bridge(label = "isSet")] is_set: bool, seconds: f64);
    #[swift_bridge(swift_name = "hasCreationDate")]
    fn has_creation_date(self: &CzWriteEntry) -> bool;
    #[swift_bridge(swift_name = "creationDate")]
    fn creation_date(self: &CzWriteEntry) -> f64;
    #[swift_bridge(swift_name = "setCreationDate")]
    fn set_creation_date(self: &CzWriteEntry, #[swift_bridge(label = "isSet")] is_set: bool, seconds: f64);
    #[swift_bridge(swift_name = "hasModificationDate")]
    fn has_modification_date(self: &CzWriteEntry) -> bool;
    #[swift_bridge(swift_name = "modificationDate")]
    fn modification_date(self: &CzWriteEntry) -> f64;
    #[swift_bridge(swift_name = "setModificationDate")]
    fn set_modification_date(self: &CzWriteEntry, #[swift_bridge(label = "isSet")] is_set: bool, seconds: f64);
    fn path(self: &CzWriteEntry) -> Option<String>;
    #[swift_bridge(swift_name = "setPath")]
    fn set_path(self: &CzWriteEntry, path: Option<String>);
    #[swift_bridge(swift_name = "pathUtf8")]
    fn path_utf8(self: &CzWriteEntry) -> Option<String>;
    #[swift_bridge(swift_name = "setPathUtf8")]
    fn set_path_utf8(self: &CzWriteEntry, path: Option<String>);
    #[swift_bridge(swift_name = "symlinkTarget")]
    fn symlink_target(self: &CzWriteEntry) -> Option<String>;
    #[swift_bridge(swift_name = "setSymlinkTarget")]
    fn set_symlink_target(self: &CzWriteEntry, target: Option<String>);
    // The outcome holds the `[String: Data]`. The setter's values are joined
    // into one, with each one's length.
    fn xattrs(self: &CzWriteEntry) -> CzOutcome;
    #[swift_bridge(swift_name = "setXattrs")]
    fn set_xattrs(self: &CzWriteEntry, names: Vec<String>, lengths: Vec<u64>, values: Vec<u8>);

    type CzArchiveWriter;
    #[swift_bridge(swift_name = "newArchiveWriter")]
    fn cz_archive_writer_new(configuration: RustArchiveWriterConfiguration) -> CzOutcome;
    // `ArchiveWriter(format:filter:options:locales:file:)`, with its
    // arguments in a configuration.
    #[swift_bridge(swift_name = "archiveWriterWithFile")]
    fn cz_archive_writer_with_file(configuration: RustArchiveWriterConfiguration, file: &str) -> CzOutcome;
    // `WriteEntry(_:)`, on the writer it takes.
    #[swift_bridge(swift_name = "newEntry")]
    fn new_entry(self: &CzArchiveWriter) -> CzWriteEntry;
    fn open(self: &CzArchiveWriter, file: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "openWithFileDescriptor")]
    fn open_with_file_descriptor(
      self: &CzArchiveWriter,
      #[swift_bridge(label = "fileDescriptor")] file_descriptor: i32,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "finishEncoding")]
    fn finish_encoding(self: &CzArchiveWriter) -> CzOutcome;
    #[swift_bridge(swift_name = "makeTransactionWriter")]
    fn make_transaction_writer(self: &CzArchiveWriter) -> CzArchiveWriterTransaction;
    // `data` stands for `nil` when `has_data` is false.
    #[swift_bridge(swift_name = "writeEntry")]
    fn write_entry(
      self: &CzArchiveWriter,
      entry: CzWriteEntry,
      #[swift_bridge(label = "hasData")] has_data: bool,
      data: Vec<u8>,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "archiveDirectory")]
    fn archive_directory(self: &CzArchiveWriter, dir: &str) -> CzOutcome;
    fn archive(self: &CzArchiveWriter, paths: Vec<String>, base: &str) -> CzOutcome;

    type CzArchiveWriterTransaction;
    #[swift_bridge(swift_name = "writeHeader")]
    fn write_header(self: &CzArchiveWriterTransaction, entry: CzWriteEntry) -> CzOutcome;
    #[swift_bridge(swift_name = "writeChunk")]
    fn write_chunk(self: &CzArchiveWriterTransaction, data: Vec<u8>) -> CzOutcome;
    fn finish(self: &CzArchiveWriterTransaction) -> CzOutcome;

    type CzArchiveReader;
    #[swift_bridge(swift_name = "openArchiveReader")]
    fn cz_archive_reader_new(file: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "archiveReaderWithFormat")]
    fn cz_archive_reader_with_format(format: &str, filter: &str, file: &str) -> CzOutcome;
    // The reader owns the descriptor, and closes it.
    #[swift_bridge(swift_name = "archiveReaderWithFileHandle")]
    fn cz_archive_reader_with_file_handle(
      format: &str,
      filter: &str,
      #[swift_bridge(label = "fileHandle")] file_handle: i32,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "archiveReaderWithBundle")]
    fn cz_archive_reader_with_bundle(
      name: &str,
      bundle: Vec<u8>,
      #[swift_bridge(label = "tempDirectoryBaseName")] temp_directory_base_name: Option<String>,
    ) -> CzOutcome;
    fn duplicate(self: &CzArchiveReader) -> CzArchiveReader;
    #[swift_bridge(swift_name = "makeIterator")]
    fn make_iterator(self: &CzArchiveReader) -> CzArchiveIterator;
    #[swift_bridge(swift_name = "makeStreamingIterator")]
    fn make_streaming_iterator(self: &CzArchiveReader) -> CzStreamingIterator;
    #[swift_bridge(swift_name = "throwIfStreamFailed")]
    fn throw_if_stream_failed(self: &CzArchiveReader) -> CzOutcome;
    #[swift_bridge(swift_name = "extractContents")]
    fn extract_contents(self: &CzArchiveReader, to: &str) -> CzOutcome;
    // The outcome holds the entry and its data.
    #[swift_bridge(swift_name = "extractFile")]
    fn extract_file(self: &CzArchiveReader, path: &str) -> CzOutcome;

    // Each `next` outcome holds the entry and its data or reader, or
    // `Absent`.
    type CzArchiveIterator;
    fn next(self: &CzArchiveIterator) -> CzOutcome;
    type CzStreamingIterator;
    fn next(self: &CzStreamingIterator) -> CzOutcome;

    // `read(_:maxLength:)`: the outcome holds the bytes read, or fails for
    // Swift's `-1`.
    type CzArchiveEntryReader;
    fn read(self: &CzArchiveEntryReader, #[swift_bridge(label = "maxLength")] max_length: usize) -> CzOutcome;

    // ContainerizationOS. `CapabilityName` and `CapabilitySet` cross as their
    // `description`s. For unit tests: `CapabilityName.allCases`' descriptions
    // and `capValue`s, and each `CapabilitySet`'s description in the order of
    // Rust's `ALL`.
    #[swift_bridge(swift_name = "capabilityNameDescriptions")]
    fn cz_capability_name_descriptions() -> Vec<String>;
    #[swift_bridge(swift_name = "capabilityNameCapValues")]
    fn cz_capability_name_cap_values() -> Vec<u32>;
    #[swift_bridge(swift_name = "capabilitySetDescriptions")]
    fn cz_capability_set_descriptions() -> Vec<String>;
    #[swift_bridge(swift_name = "parseCapabilityName")]
    fn cz_capability_name_parse(#[swift_bridge(label = "rawValue")] raw_value: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "parseCapabilitySet")]
    fn cz_capability_set_parse(#[swift_bridge(label = "rawValue")] raw_value: &str) -> CzOutcome;

    // A `Terminal`. A `Terminal.Size?` crosses as a flag and its width and
    // height, and a `Terminal` argument as a `duplicate()`.
    type CzTerminal;
    #[swift_bridge(swift_name = "newTerminal")]
    fn cz_terminal_new(descriptor: i32, #[swift_bridge(label = "setInitState")] set_init_state: bool) -> CzOutcome;
    #[swift_bridge(swift_name = "currentTerminal")]
    fn cz_terminal_current() -> CzOutcome;
    #[swift_bridge(swift_name = "createTerminal")]
    fn cz_terminal_create(
      #[swift_bridge(label = "hasInitialSize")] has_initial_size: bool,
      width: u16,
      height: u16,
    ) -> CzOutcome;
    fn terminal(self: &CzOutcome) -> CzTerminal;
    #[swift_bridge(swift_name = "parentTerminal")]
    fn parent_terminal(self: &CzOutcome) -> CzTerminal;
    #[swift_bridge(swift_name = "childTerminal")]
    fn child_terminal(self: &CzOutcome) -> CzTerminal;
    #[swift_bridge(swift_name = "terminalSizeWidth")]
    fn terminal_size_width(self: &CzOutcome) -> u16;
    #[swift_bridge(swift_name = "terminalSizeHeight")]
    fn terminal_size_height(self: &CzOutcome) -> u16;
    fn duplicate(self: &CzTerminal) -> CzTerminal;
    fn handle(self: &CzTerminal) -> i32;
    fn write(self: &CzTerminal, data: Vec<u8>) -> CzOutcome;
    fn size(self: &CzTerminal) -> CzOutcome;
    #[swift_bridge(swift_name = "resizeFrom")]
    fn resize_from(self: &CzTerminal, pty: CzTerminal) -> CzOutcome;
    // `resize(size:)`, with the size's width and height.
    #[swift_bridge(swift_name = "resizeSize")]
    fn resize_size(self: &CzTerminal, width: u16, height: u16) -> CzOutcome;
    fn resize(self: &CzTerminal, width: u16, height: u16) -> CzOutcome;
    fn setraw(self: &CzTerminal) -> CzOutcome;
    #[swift_bridge(swift_name = "enableEcho")]
    fn enable_echo(self: &CzTerminal) -> CzOutcome;
    #[swift_bridge(swift_name = "disableEcho")]
    fn disable_echo(self: &CzTerminal) -> CzOutcome;
    fn close(self: &CzTerminal) -> CzOutcome;
    fn reset(self: &CzTerminal) -> CzOutcome;
    #[swift_bridge(swift_name = "tryReset")]
    fn try_reset(self: &CzTerminal);

    // `KeychainQuery`. `get`'s outcome holds a `KeychainQueryResult?`, and
    // `list`'s a `[RegistryInfo]`, their dates as seconds since 1970.
    #[swift_bridge(swift_name = "keychainQuerySave")]
    fn cz_keychain_query_save(
      #[swift_bridge(label = "securityDomain")] security_domain: &str,
      #[swift_bridge(label = "accessGroup")] access_group: Option<String>,
      hostname: &str,
      username: &str,
      password: &str,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "keychainQueryDelete")]
    fn cz_keychain_query_delete(
      #[swift_bridge(label = "securityDomain")] security_domain: &str,
      #[swift_bridge(label = "accessGroup")] access_group: Option<String>,
      hostname: &str,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "keychainQueryGet")]
    fn cz_keychain_query_get(
      #[swift_bridge(label = "securityDomain")] security_domain: &str,
      #[swift_bridge(label = "accessGroup")] access_group: Option<String>,
      hostname: &str,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "keychainQueryList")]
    fn cz_keychain_query_list(
      #[swift_bridge(label = "securityDomain")] security_domain: &str,
      #[swift_bridge(label = "accessGroup")] access_group: Option<String>,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "keychainQueryExists")]
    fn cz_keychain_query_exists(
      #[swift_bridge(label = "securityDomain")] security_domain: &str,
      #[swift_bridge(label = "accessGroup")] access_group: Option<String>,
      hostname: &str,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "keychainQueryResultUsername")]
    fn keychain_query_result_username(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "keychainQueryResultPassword")]
    fn keychain_query_result_password(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "keychainQueryResultModifiedDate")]
    fn keychain_query_result_modified_date(self: &CzOutcome) -> f64;
    #[swift_bridge(swift_name = "keychainQueryResultCreatedDate")]
    fn keychain_query_result_created_date(self: &CzOutcome) -> f64;

    // `Sysctl.byName(_:)`, whose outcome holds an `Int64`.
    #[swift_bridge(swift_name = "sysctlByName")]
    fn cz_sysctl_by_name(name: &str) -> CzOutcome;
    fn integer(self: &CzOutcome) -> i64;

    // `File.info(_:)`, and the `FileInfo` it returns. Swift's `Int`s cross as
    // `Int64`s.
    type CzFileInfo;
    #[swift_bridge(swift_name = "fileInfoAt")]
    fn cz_file_info(path: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "fileInfo")]
    fn file_info(self: &CzOutcome) -> CzFileInfo;
    fn mode(self: &CzFileInfo) -> u16;
    fn uid(self: &CzFileInfo) -> i64;
    fn gid(self: &CzFileInfo) -> i64;
    fn dev(self: &CzFileInfo) -> i64;
    fn ino(self: &CzFileInfo) -> i64;
    fn size(self: &CzFileInfo) -> i64;
    fn path(self: &CzFileInfo) -> String;
    #[swift_bridge(swift_name = "isDirectory")]
    fn is_directory(self: &CzFileInfo) -> bool;
    #[swift_bridge(swift_name = "isPipe")]
    fn is_pipe(self: &CzFileInfo) -> bool;
    #[swift_bridge(swift_name = "isSocket")]
    fn is_socket(self: &CzFileInfo) -> bool;
    #[swift_bridge(swift_name = "isLink")]
    fn is_link(self: &CzFileInfo) -> bool;
    #[swift_bridge(swift_name = "isRegularFile")]
    fn is_regular_file(self: &CzFileInfo) -> bool;
    #[swift_bridge(swift_name = "isBlock")]
    fn is_block(self: &CzFileInfo) -> bool;
    #[swift_bridge(swift_name = "isChar")]
    fn is_char(self: &CzFileInfo) -> bool;

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
    fn int32(self: &CzOutcome) -> i32;

    // Containerization's value types. Each crosses whole, and Swift does the
    // work: the outcomes hold what Swift returns.
    //
    // A `Kernel.CommandLine` crosses as its two lists, and each edit's outcome
    // holds the edited command line.
    #[swift_bridge(swift_name = "kernelCommandLineAddDebug")]
    fn cz_kernel_command_line_add_debug(
      #[swift_bridge(label = "kernelArgs")] kernel_args: Vec<String>,
      #[swift_bridge(label = "initArgs")] init_args: Vec<String>,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "kernelCommandLineAddPanic")]
    fn cz_kernel_command_line_add_panic(
      #[swift_bridge(label = "kernelArgs")] kernel_args: Vec<String>,
      #[swift_bridge(label = "initArgs")] init_args: Vec<String>,
      level: i64,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "kernelCommandLineSetAgentLogLevel")]
    fn cz_kernel_command_line_set_agent_log_level(
      #[swift_bridge(label = "kernelArgs")] kernel_args: Vec<String>,
      #[swift_bridge(label = "initArgs")] init_args: Vec<String>,
      level: LogLevel,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "commandLineKernelArgs")]
    fn command_line_kernel_args(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "commandLineInitArgs")]
    fn command_line_init_args(self: &CzOutcome) -> Vec<String>;
    // `clone(to:)`'s outcome holds a `Mount`, and `tagHash`'s a string.
    #[swift_bridge(swift_name = "cloneMount")]
    fn cz_mount_clone(mount: RustMount, to: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "mountTagHash")]
    fn cz_mount_tag_hash(mount: RustMount) -> CzOutcome;
    #[swift_bridge(swift_name = "validateDNS")]
    fn cz_dns_validate(dns: RustDns) -> CzOutcome;
    #[swift_bridge(swift_name = "dnsResolvConf")]
    fn cz_dns_resolv_conf(dns: RustDns) -> CzOutcome;
    #[swift_bridge(swift_name = "hostsFile")]
    fn cz_hosts_file(hosts: RustHosts) -> CzOutcome;
    #[swift_bridge(swift_name = "hostsEntryRendered")]
    fn cz_hosts_entry_rendered(entry: RustHostsEntry) -> CzOutcome;
    // The outcome holds a `Hosts.Entry`.
    #[swift_bridge(swift_name = "namedHostsEntry")]
    fn cz_hosts_entry_named(name: HostsEntryName, comment: Option<String>) -> CzOutcome;
    #[swift_bridge(swift_name = "hostsEntryIpAddress")]
    fn hosts_entry_ip_address(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "hostsEntryHostnames")]
    fn hosts_entry_hostnames(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "hostsEntryComment")]
    fn hosts_entry_comment(self: &CzOutcome) -> Option<String>;
    // The outcome holds an `ExitStatus`.
    #[swift_bridge(swift_name = "newExitStatus")]
    fn cz_exit_status_new(#[swift_bridge(label = "exitCode")] exit_code: i32) -> CzOutcome;
    // The outcome holds the kind's `description`.
    #[swift_bridge(swift_name = "parseLinuxRLimitKind")]
    fn cz_linux_rlimit_kind_parse(string: &str) -> CzOutcome;
    // Swift fills `seed` with the configuration it makes, and hands it to
    // `receive`.
    #[swift_bridge(swift_name = "linuxProcessConfigurationFromImageConfig")]
    fn cz_linux_process_configuration_from_image_config(
      config: RustImageConfig,
      seed: RustLinuxProcessConfiguration,
      receive: RustConfigureProcess,
    ) -> CzOutcome;
    // Swift reads `process`, sets its terminal, fills `process` back in, and
    // hands it to `receive`.
    #[swift_bridge(swift_name = "linuxProcessConfigurationSetTerminalIO")]
    fn cz_linux_process_configuration_set_terminal_io(
      process: RustLinuxProcessConfiguration,
      terminal: CzTerminal,
      receive: RustConfigureProcess,
    ) -> CzOutcome;
    // `Signal`. Parsing's outcome holds a raw value, the maps' a
    // `[String: Int32]`, `platformName`'s a name or `Absent`, and
    // `linuxSignal`'s a raw value or `Absent`.
    #[swift_bridge(swift_name = "parseSignal")]
    fn cz_signal_parse(name: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "parseSignalFrom")]
    fn cz_signal_parse_from(name: &str, names: Vec<String>, values: Vec<i32>) -> CzOutcome;
    #[swift_bridge(swift_name = "linuxSignals")]
    fn cz_signal_linux() -> CzOutcome;
    #[swift_bridge(swift_name = "platformSignals")]
    fn cz_signal_platform() -> CzOutcome;
    #[swift_bridge(swift_name = "signalPlatformName")]
    fn cz_signal_platform_name(signal: i32) -> CzOutcome;
    #[swift_bridge(swift_name = "signalLinuxSignal")]
    fn cz_signal_linux_signal(signal: i32) -> CzOutcome;

    // For unit tests: `LinuxRLimit.Kind`'s descriptions in Swift's order;
    // `LinuxCapabilities.allCapabilities` and `defaultOCICapabilities`, as
    // their `toOCI()`; the raw values of `Signal.Linux`'s and
    // `Signal.Darwin`'s signals, in Swift's order, with `rtmin()` last in
    // Linux's; and the raw values of `SystemPlatform.OS.allCases` and then
    // `Architecture.allCases`.
    #[swift_bridge(swift_name = "linuxRLimitKindDescriptions")]
    fn cz_linux_rlimit_kind_descriptions() -> Vec<String>;
    #[swift_bridge(swift_name = "linuxCapabilitiesPresets")]
    fn cz_linux_capabilities_presets() -> CzOutcome;
    #[swift_bridge(swift_name = "linuxSignalValues")]
    fn cz_signal_linux_values() -> Vec<i32>;
    #[swift_bridge(swift_name = "darwinSignalValues")]
    fn cz_signal_darwin_values() -> Vec<i32>;
    #[swift_bridge(swift_name = "systemPlatformRawValues")]
    fn cz_system_platform_raw_values() -> Vec<String>;

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
    fn duplicate(self: &CzLocalContentStore) -> CzLocalContentStore;
    // `ImageStore(path:contentStore:)` and `Image(description:contentStore:)`,
    // on the store they take.
    #[swift_bridge(swift_name = "imageStore")]
    fn image_store(self: &CzLocalContentStore, path: &str) -> CzOutcome;
    fn image(self: &CzLocalContentStore, description: RustImageDescription) -> CzImage;
    // `body` gets the ingest directory, and returns whether it succeeded.
    fn ingest(self: &CzLocalContentStore, body: Box<dyn FnOnce(String) -> bool>) -> CzOutcome;
    #[swift_bridge(swift_name = "newIngestSession")]
    fn new_ingest_session(self: &CzLocalContentStore) -> CzOutcome;
    #[swift_bridge(swift_name = "completeIngestSession")]
    fn complete_ingest_session(self: &CzLocalContentStore, id: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "cancelIngestSession")]
    fn cancel_ingest_session(self: &CzLocalContentStore, id: &str) -> CzOutcome;
    // What `newIngestSession` returned.
    #[swift_bridge(swift_name = "ingestSessionId")]
    fn ingest_session_id(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "ingestSessionDirectory")]
    fn ingest_session_directory(self: &CzOutcome) -> String;

    type CzContentWriter;
    #[swift_bridge(swift_name = "openContentWriter")]
    fn cz_content_writer_new(base: &str) -> CzOutcome;
    fn write(self: &CzContentWriter, data: Vec<u8>) -> CzOutcome;
    fn create(self: &CzContentWriter, from: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "copyContent")]
    fn cz_content_writer_copy(from: &str, destination: &str) -> CzOutcome;

    // `LocalContent(path:)`, held as a `Content`.
    #[swift_bridge(swift_name = "openLocalContent")]
    fn cz_local_content_open(path: &str) -> CzOutcome;
    // For a test that compares Rust's copy with it.
    #[swift_bridge(swift_name = "localContentMaxDecodedSize")]
    fn cz_local_content_max_decoded_size() -> isize;

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

    // An `Authentication?`.
    type CzAuthentication;
    #[swift_bridge(swift_name = "basicAuthentication")]
    fn cz_basic_authentication(username: &str, password: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "noAuthentication")]
    fn cz_no_authentication() -> CzOutcome;
    fn authentication(self: &CzOutcome) -> CzAuthentication;
    fn duplicate(self: &CzAuthentication) -> CzAuthentication;
    fn token(self: &CzAuthentication) -> CzOutcome;

    // `KeychainHelper`, as its security domain and access group. `list`'s
    // outcome holds a `[RegistryInfo]`, its dates as seconds since 1970.
    #[swift_bridge(swift_name = "keychainHelperLookup")]
    fn cz_keychain_helper_lookup(
      #[swift_bridge(label = "securityDomain")] security_domain: &str,
      #[swift_bridge(label = "accessGroup")] access_group: Option<String>,
      hostname: &str,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "keychainHelperList")]
    fn cz_keychain_helper_list(
      #[swift_bridge(label = "securityDomain")] security_domain: &str,
      #[swift_bridge(label = "accessGroup")] access_group: Option<String>,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "keychainHelperDelete")]
    fn cz_keychain_helper_delete(
      #[swift_bridge(label = "securityDomain")] security_domain: &str,
      #[swift_bridge(label = "accessGroup")] access_group: Option<String>,
      hostname: &str,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "keychainHelperSave")]
    fn cz_keychain_helper_save(
      #[swift_bridge(label = "securityDomain")] security_domain: &str,
      #[swift_bridge(label = "accessGroup")] access_group: Option<String>,
      hostname: &str,
      username: &str,
      password: &str,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "registryInfoHostname")]
    fn registry_info_hostname(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "registryInfoUsername")]
    fn registry_info_username(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "registryInfoModifiedDate")]
    fn registry_info_modified_date(self: &CzOutcome) -> f64;
    #[swift_bridge(swift_name = "registryInfoCreatedDate")]
    fn registry_info_created_date(self: &CzOutcome) -> f64;

    // `RegistryClient`'s inits. `retry_options` stands for `nil` when
    // `has_retry_options` is false.
    type CzRegistryClient;
    #[swift_bridge(swift_name = "newRegistryClient")]
    fn cz_registry_client_new(reference: &str, insecure: bool, auth: CzAuthentication) -> CzOutcome;
    #[swift_bridge(swift_name = "registryClientWithHost")]
    fn cz_registry_client_with_host(
      host: &str,
      scheme: Option<String>,
      port: Option<u16>,
      authentication: CzAuthentication,
      #[swift_bridge(label = "clientID")] client_id: Option<String>,
      #[swift_bridge(label = "hasRetryOptions")] has_retry_options: bool,
      #[swift_bridge(label = "maxRetries")] max_retries: isize,
      #[swift_bridge(label = "retryInterval")] retry_interval: u64,
      #[swift_bridge(label = "bufferSize")] buffer_size: usize,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "registryClient")]
    fn registry_client(self: &CzOutcome) -> CzRegistryClient;
    fn ping(self: &CzRegistryClient) -> CzOutcome;
    fn resolve(self: &CzRegistryClient, name: &str, tag: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "fetchData")]
    fn fetch_data(self: &CzRegistryClient, name: &str, descriptor: RustDescriptor) -> CzOutcome;
    // The outcome holds the size and the digest, as `ContentWriter`'s do.
    #[swift_bridge(swift_name = "fetchBlob")]
    fn fetch_blob(
      self: &CzRegistryClient,
      name: &str,
      descriptor: RustDescriptor,
      into: &str,
      progress: RustProgressHandler,
    ) -> CzOutcome;
    fn catalog(self: &CzRegistryClient, prefix: Option<String>) -> CzOutcome;
    fn referrers(
      self: &CzRegistryClient,
      name: &str,
      digest: &str,
      #[swift_bridge(label = "artifactType")] artifact_type: Option<String>,
    ) -> CzOutcome;

    type CzImageStore;
    #[swift_bridge(swift_name = "openImageStore")]
    fn cz_image_store_new(path: &str) -> CzOutcome;
    #[swift_bridge(swift_name = "defaultImageStore")]
    fn cz_image_store_default() -> CzOutcome;
    fn path(self: &CzImageStore) -> String;
    fn get(self: &CzImageStore, reference: &str, pull: bool) -> CzOutcome;
    fn list(self: &CzImageStore) -> CzOutcome;
    fn delete(
      self: &CzImageStore,
      reference: &str,
      #[swift_bridge(label = "performCleanup")] perform_cleanup: bool,
    ) -> CzOutcome;
    fn tag(self: &CzImageStore, existing: &str, new: &str) -> CzOutcome;
    // In these, `platform` stands for `nil` when `has_platform` is false.
    fn pull(
      self: &CzImageStore,
      reference: &str,
      #[swift_bridge(label = "hasPlatform")] has_platform: bool,
      platform: RustPlatform,
      insecure: bool,
      auth: CzAuthentication,
      progress: RustProgressHandler,
      #[swift_bridge(label = "maxConcurrentDownloads")] max_concurrent_downloads: usize,
    ) -> CzOutcome;
    fn push(
      self: &CzImageStore,
      reference: &str,
      #[swift_bridge(label = "hasPlatform")] has_platform: bool,
      platform: RustPlatform,
      insecure: bool,
      auth: CzAuthentication,
      progress: RustProgressHandler,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "pushAll")]
    fn push_all(
      self: &CzImageStore,
      references: Vec<String>,
      #[swift_bridge(label = "hasPlatform")] has_platform: bool,
      platform: RustPlatform,
      insecure: bool,
      auth: CzAuthentication,
      #[swift_bridge(label = "maxConcurrentUploads")] max_concurrent_uploads: usize,
      progress: RustProgressHandler,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "getInitImage")]
    fn get_init_image(
      self: &CzImageStore,
      reference: &str,
      auth: CzAuthentication,
      progress: RustProgressHandler,
    ) -> CzOutcome;
    fn save(
      self: &CzImageStore,
      references: Vec<String>,
      out: &str,
      #[swift_bridge(label = "hasPlatform")] has_platform: bool,
      platform: RustPlatform,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "cleanUpOrphanedBlobs")]
    fn clean_up_orphaned_blobs(self: &CzImageStore) -> CzOutcome;
    #[swift_bridge(swift_name = "calculateOrphanedBlobsSize")]
    fn calculate_orphaned_blobs_size(self: &CzImageStore) -> CzOutcome;
    fn create(self: &CzImageStore, description: RustImageDescription) -> CzOutcome;
    fn load(self: &CzImageStore, from: &str, progress: RustProgressHandler) -> CzOutcome;
    // `InitImage.create` and `KernelImage.create`, on the image store they
    // take, with the content store as a `duplicate()` and `labels` as its keys
    // and values.
    #[swift_bridge(swift_name = "createInitImage")]
    fn create_init_image(
      self: &CzImageStore,
      reference: &str,
      rootfs: &str,
      platform: RustPlatform,
      #[swift_bridge(label = "labelKeys")] label_keys: Vec<String>,
      #[swift_bridge(label = "labelValues")] label_values: Vec<String>,
      #[swift_bridge(label = "contentStore")] content_store: CzLocalContentStore,
    ) -> CzOutcome;
    #[swift_bridge(swift_name = "createKernelImage")]
    fn create_kernel_image(
      self: &CzImageStore,
      reference: &str,
      binaries: Vec<RustKernel>,
      #[swift_bridge(label = "labelKeys")] label_keys: Vec<String>,
      #[swift_bridge(label = "labelValues")] label_values: Vec<String>,
      #[swift_bridge(label = "contentStore")] content_store: CzLocalContentStore,
    ) -> CzOutcome;
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
    fn descriptor(self: &CzImage) -> CzOutcome;
    fn index(self: &CzImage) -> CzOutcome;
    fn manifest(self: &CzImage, platform: RustPlatform) -> CzOutcome;
    #[swift_bridge(swift_name = "descriptorFor")]
    fn descriptor_for(self: &CzImage, platform: RustPlatform) -> CzOutcome;
    fn config(self: &CzImage, platform: RustPlatform) -> CzOutcome;
    #[swift_bridge(swift_name = "referencedDigests")]
    fn referenced_digests(self: &CzImage) -> CzOutcome;
    #[swift_bridge(swift_name = "getContent")]
    fn get_content(self: &CzImage, digest: &str) -> CzOutcome;
    // `InitImage(image:)` and `KernelImage(image:)`.
    #[swift_bridge(swift_name = "initImage")]
    fn init_image(self: &CzImage) -> CzInitImage;
    #[swift_bridge(swift_name = "kernelImage")]
    fn kernel_image(self: &CzImage) -> CzKernelImage;

    type CzKernelImage;
    #[swift_bridge(swift_name = "kernelImage")]
    fn kernel_image(self: &CzOutcome) -> CzKernelImage;
    fn name(self: &CzKernelImage) -> String;
    fn kernel(self: &CzKernelImage, platform: RustSystemPlatform) -> CzOutcome;
    // For a test that compares Rust's copy with it.
    #[swift_bridge(swift_name = "kernelImageMediaType")]
    fn cz_kernel_image_media_type() -> String;
    // A `Kernel`, field by field: Rust builds its own.
    #[swift_bridge(swift_name = "kernelPath")]
    fn kernel_path(self: &CzOutcome) -> String;
    #[swift_bridge(swift_name = "kernelPlatformOs")]
    fn kernel_platform_os(self: &CzOutcome) -> PlatformOs;
    #[swift_bridge(swift_name = "kernelPlatformArchitecture")]
    fn kernel_platform_architecture(self: &CzOutcome) -> PlatformArchitecture;
    #[swift_bridge(swift_name = "kernelKernelArgs")]
    fn kernel_kernel_args(self: &CzOutcome) -> Vec<String>;
    #[swift_bridge(swift_name = "kernelInitArgs")]
    fn kernel_init_args(self: &CzOutcome) -> Vec<String>;

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
