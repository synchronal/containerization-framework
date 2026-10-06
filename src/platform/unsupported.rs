//! Stand-ins for the bridge on every platform but macOS: the same types and
//! functions, so the wrappers are written once and a cross-platform workspace
//! still compiles.
//!
//! Nothing can make a Swift object here: every constructor's outcome is a
//! failure, and every handle is uninhabited, so its methods never run.

use super::ConfigureContainer;
use super::ConfigureProcess;
use super::Progress;
use crate::containerization;
use crate::containerization::container_manager;
use crate::containerization::hosts;
use crate::containerization::image;
use crate::containerization::kernel;
use crate::containerization::linux_container;
use crate::containerization_archive;
use crate::containerization_ext4::ext4;
use crate::containerization_extras;
use crate::containerization_extras::IPv6Address;
use crate::containerization_extras::IpAddress;
use crate::containerization_oci;
use crate::containerization_os;
use std::collections::BTreeMap;
use std::convert::Infallible;

const MACOS_ONLY: &str = "Containerization.framework is macOS only";

/// `ProgressEvent`, less its value.
pub(crate) enum ProgressKind {
  Items,
  TotalItems,
  Size,
  TotalSize,
}

/// swift-log's `Logger.Level`.
pub(crate) enum LogLevel {
  Trace,
  Debug,
  Info,
  Notice,
  Warning,
  Error,
  Critical,
}

impl From<kernel::LogLevel> for LogLevel {
  fn from(level: kernel::LogLevel) -> Self {
    match level {
      kernel::LogLevel::Trace => Self::Trace,
      kernel::LogLevel::Debug => Self::Debug,
      kernel::LogLevel::Info => Self::Info,
      kernel::LogLevel::Notice => Self::Notice,
      kernel::LogLevel::Warning => Self::Warning,
      kernel::LogLevel::Error => Self::Error,
      kernel::LogLevel::Critical => Self::Critical,
    }
  }
}

/// `Hosts.Entry`'s static constructors.
pub(crate) enum HostsEntryName {
  LocalHostIpv4,
  LocalHostIpv6,
  Ipv6LocalNet,
  Ipv6MulticastPrefix,
  Ipv6AllNodes,
  Ipv6AllRouters,
}

/// `FilesystemOperation`.
pub(crate) enum FilesystemOperationKind {
  Freeze,
  Thaw,
  Trim,
}

impl From<containerization::FilesystemOperation> for FilesystemOperationKind {
  fn from(operation: containerization::FilesystemOperation) -> Self {
    match operation {
      containerization::FilesystemOperation::Freeze => Self::Freeze,
      containerization::FilesystemOperation::Thaw => Self::Thaw,
      containerization::FilesystemOperation::Trim => Self::Trim,
    }
  }
}

/// Always a failure, so nothing is taken from it.
pub(crate) struct CzOutcome;

impl CzOutcome {
  pub(crate) fn error(&self) -> Option<String> {
    Some(MACOS_ONLY.to_string())
  }
}

macro_rules! taken {
  ($($name:ident -> $type:ty),* $(,)?) => {
    impl CzOutcome {
      $(pub(crate) fn $name(&self) -> $type {
        unreachable!("a failed outcome holds nothing")
      })*
    }
  };
}

taken!(
  local_content_store -> CzLocalContentStore,
  content -> CzContent,
  image_store -> CzImageStore,
  image -> CzImage,
  images -> CzImages,
  init_image -> CzInitImage,
  container_manager -> CzContainerManager,
  linux_container -> CzLinuxContainer,
  linux_process -> CzLinuxProcess,
  content_writer -> CzContentWriter,
  written_size -> i64,
  written_digest -> String,
  ext4_reader -> CzExt4Reader,
  ext4_formatter -> CzExt4Formatter,
  scanned_size -> i64,
  scanned_items -> isize,
  inode_number -> u32,
  inode -> ext4::Inode,
  compressed_name_id -> u8,
  compressed_name_str -> String,
  len -> usize,
  read_stream -> CzReadStream,
  archive_writer -> CzArchiveWriter,
  archive_reader -> CzArchiveReader,
  write_entry -> CzWriteEntry,
  entry_data -> Vec<u8>,
  archive_entry_reader -> CzArchiveEntryReader,
  data_map_keys -> Vec<String>,
  terminal -> CzTerminal,
  parent_terminal -> CzTerminal,
  child_terminal -> CzTerminal,
  terminal_size_width -> u16,
  terminal_size_height -> u16,
  keychain_query_result -> containerization_os::KeychainQueryResult,
  integer -> i64,
  file_info -> CzFileInfo,
  reference -> CzReference,
  authentication -> CzAuthentication,
  registry_infos -> Vec<containerization_os::RegistryInfo>,
  kernel_image -> CzKernelImage,
  kernel -> containerization::Kernel,
  ingest_session_id -> String,
  ingest_session_directory -> String,
  registry_client -> CzRegistryClient,
  parsed_digest_encoded -> String,
  boolean -> bool,
  is_some -> bool,
  ipv4_address -> containerization_extras::IPv4Address,
  ipv6_address -> containerization_extras::IPv6Address,
  ip_address -> containerization_extras::IpAddress,
  prefix -> containerization_extras::Prefix,
  cidr_v4 -> containerization_extras::CIDRv4,
  cidr_v6 -> containerization_extras::CIDRv6,
  cidr -> containerization_extras::Cidr,
  mac_address -> containerization_extras::MACAddress,
  uint128 -> u128,
  platform -> containerization_oci::Platform,
  descriptor -> containerization_oci::Descriptor,
  index -> containerization_oci::Index,
  manifest -> containerization_oci::Manifest,
  oci_image -> containerization_oci::Image,
  spec -> containerization_oci::Spec,
  process -> containerization_oci::Process,
  oci_linux_capabilities -> containerization_oci::LinuxCapabilities,
  posix_rlimit -> containerization_oci::POSIXRlimit,
  seccomp -> containerization_oci::LinuxSeccomp,
  runtime_spec_version -> containerization_oci::RuntimeSpecVersion,
  optional_text -> Option<String>,
  mount -> containerization::Mount,
  optional_mount -> Option<containerization::Mount>,
  container_statistics -> containerization::ContainerStatistics,
  exit_code -> i32,
  exited_at -> f64,
  number -> u64,
  text -> String,
  strings -> Vec<String>,
  has_bytes -> bool,
  bytes -> Vec<u8>,
  int32 -> i32,
  int32_map -> BTreeMap<String, i32>,
  command_line_kernel_args -> Vec<String>,
  command_line_init_args -> Vec<String>,
  hosts_entry -> hosts::Entry,
);

macro_rules! handles {
  ($($name:ident),* $(,)?) => {
    $(pub(crate) struct $name(Infallible);)*
  };
}

handles!(
  CzLocalContentStore,
  CzContent,
  CzImageStore,
  CzImages,
  CzImage,
  CzInitImage,
  CzContainerManager,
  CzLinuxContainer,
  CzLinuxProcess,
  CzContentWriter,
  CzExt4Reader,
  CzExt4Formatter,
  CzReadStream,
  CzDataStream,
  CzReference,
  CzAuthentication,
  CzKernelImage,
  CzRegistryClient,
  CzWriteEntry,
  CzArchiveWriter,
  CzArchiveWriterTransaction,
  CzArchiveReader,
  CzArchiveIterator,
  CzStreamingIterator,
  CzArchiveEntryReader,
  CzTerminal,
  CzFileInfo,
);

impl CzOutcome {
  pub(crate) fn data_map_value(&self, _key: &str) -> Vec<u8> {
    unreachable!("a failed outcome holds nothing")
  }
}

/// Swift would own the descriptor, so it is closed here instead.
pub(crate) fn cz_archive_reader_with_file_handle(_format: &str, _filter: &str, file_handle: i32) -> CzOutcome {
  // SAFETY: `ArchiveReader::with_file_handle` passes the descriptor of an
  // `OwnedFd` it gave up.
  drop(unsafe { <std::os::fd::OwnedFd as std::os::fd::FromRawFd>::from_raw_fd(file_handle) });
  CzOutcome
}

pub(crate) fn cz_local_content_store_new(_path: &str) -> CzOutcome {
  CzOutcome
}

pub(crate) fn cz_image_store_new(_path: &str) -> CzOutcome {
  CzOutcome
}

pub(crate) fn cz_content_writer_new(_base: &str) -> CzOutcome {
  CzOutcome
}

pub(crate) fn cz_platform_current() -> CzOutcome {
  CzOutcome
}

pub(crate) fn cz_platform_parse(_platform: &str) -> CzOutcome {
  CzOutcome
}

pub(crate) fn cz_platform_description(_platform: containerization_oci::Platform) -> CzOutcome {
  CzOutcome
}

pub(crate) fn cz_platform_equals(
  _lhs: containerization_oci::Platform,
  _rhs: containerization_oci::Platform,
) -> CzOutcome {
  CzOutcome
}

pub(crate) fn cz_platform_matches(
  _lhs: containerization_oci::Platform,
  _rhs: containerization_oci::Platform,
) -> CzOutcome {
  CzOutcome
}

pub(crate) fn cz_progress_event_event(_kind: ProgressKind, _value: i64) -> CzOutcome {
  CzOutcome
}

pub(crate) fn cz_proxy_from_environment(
  _scheme: Option<String>,
  _host: &str,
  _has_env: bool,
  _env_keys: Vec<String>,
  _env_values: Vec<String>,
) -> CzOutcome {
  CzOutcome
}

pub(crate) fn cz_ext4_reader_new(_block_device: &str) -> CzOutcome {
  CzOutcome
}

/// Free functions that only ever fail here, by their argument types.
macro_rules! failing {
  ($($name:ident($($type:ty),*)),* $(,)?) => {
    $(pub(crate) fn $name($(_: $type),*) -> CzOutcome {
      CzOutcome
    })*
  };
}

failing!(
  cz_registry_client_new(&str, bool, CzAuthentication),
  cz_registry_client_with_host(
    &str,
    Option<String>,
    Option<u16>,
    CzAuthentication,
    Option<String>,
    bool,
    isize,
    u64,
    usize,
  ),
  cz_content_writer_copy(&str, &str),
  cz_ext4_unpack_archive(containerization::Ext4Unpacker, &str, &str, &str),
  cz_ext4_reader_read_inline_extended_attributes(Vec<u8>),
  cz_ext4_reader_read_block_extended_attributes(Vec<u8>),
  cz_ext4_inode_root(),
  cz_ext4_extended_attribute_compress_name(&str),
  cz_ext4_extended_attribute_decompress_name(isize, &str),
  cz_ext4_formatter_new(&str, ext4::formatter::FormatterOptions),
  cz_ext4_formatter_scan_archive_headers(&str, &str, &str),
  cz_read_stream_new(),
  cz_read_stream_with_url(&str, usize),
  cz_read_stream_with_data(Vec<u8>, usize),
  cz_xattr_format_description(&str),
  cz_write_entry_new(),
  cz_archive_writer_new(containerization_archive::ArchiveWriterConfiguration),
  cz_archive_writer_with_file(containerization_archive::ArchiveWriterConfiguration, &str),
  cz_archive_reader_new(&str),
  cz_archive_reader_with_format(&str, &str, &str),
  cz_archive_reader_with_bundle(&str, Vec<u8>, Option<String>),
  cz_local_content_open(&str),
  cz_image_store_default(),
  cz_basic_authentication(&str, &str),
  cz_no_authentication(),
  cz_keychain_helper_lookup(&str, Option<String>, &str),
  cz_keychain_helper_list(&str, Option<String>),
  cz_keychain_helper_delete(&str, Option<String>, &str),
  cz_keychain_helper_save(&str, Option<String>, &str, &str, &str),
  cz_reference_new(&str, Option<String>, Option<String>, Option<String>),
  cz_reference_parse(&str),
  cz_reference_with_name(&str),
  cz_reference_resolve_domain(&str),
  cz_parsed_digest_parse(&str),
  cz_parsed_digest_parse_path_component(&str),
  cz_parsed_digest_is_valid(&str),
  cz_parsed_digest_description(&str),
  cz_parsed_digest_path(&str, &str),
  cz_process_from_image_config(containerization_oci::ImageConfig),
  cz_process_description(containerization_oci::Process),
  cz_hook_description(containerization_oci::Hook),
  cz_linux_seccomp_decode(Vec<u8>),
  cz_linux_seccomp_default_profile(bool, containerization_oci::LinuxCapabilities, &str),
  cz_arch_current(),
  cz_arch_current_verified(),
  cz_runtime_spec_version_current(),
  cz_linux_rlimit_to_oci(containerization::LinuxRLimit),
  cz_linux_capabilities_to_oci(containerization::LinuxCapabilities),
  cz_kernel_command_line_add_debug(Vec<String>, Vec<String>),
  cz_kernel_command_line_add_panic(Vec<String>, Vec<String>, i64),
  cz_kernel_command_line_set_agent_log_level(Vec<String>, Vec<String>, LogLevel),
  cz_mount_clone(containerization::Mount, &str),
  cz_mount_tag_hash(containerization::Mount),
  cz_dns_validate(containerization::Dns),
  cz_dns_resolv_conf(containerization::Dns),
  cz_hosts_file(containerization::Hosts),
  cz_hosts_entry_rendered(hosts::Entry),
  cz_hosts_entry_named(HostsEntryName, Option<String>),
  cz_exit_status_new(i32),
  cz_linux_rlimit_kind_parse(&str),
  cz_linux_process_configuration_from_image_config(
    containerization_oci::ImageConfig,
    containerization::LinuxProcessConfiguration,
    ConfigureProcess,
  ),
  cz_linux_process_configuration_set_terminal_io(
    containerization::LinuxProcessConfiguration,
    CzTerminal,
    ConfigureProcess,
  ),
  cz_signal_parse(&str),
  cz_signal_parse_from(&str, Vec<String>, Vec<i32>),
  cz_signal_linux(),
  cz_signal_platform(),
  cz_signal_platform_name(i32),
  cz_signal_linux_signal(i32),
  cz_system_platform_oci_platform(containerization::SystemPlatform),
  cz_bundle_create(&str, containerization_oci::Spec),
  cz_bundle_create_from_data(&str, Vec<u8>),
  cz_bundle_load(&str),
  cz_bundle_config_path(&str),
  cz_bundle_rootfs_path(&str),
  cz_bundle_delete(&str),
  cz_bundle_load_config(&str),
  cz_ipv4_address_from_bytes(Vec<u8>),
  cz_ipv4_address_parse(&str),
  cz_ipv4_address_bytes(u32),
  cz_ipv4_address_description(u32),
  cz_ipv4_address_is_unspecified(u32),
  cz_ipv4_address_is_loopback(u32),
  cz_ipv4_address_is_multicast(u32),
  cz_ipv4_address_is_link_local(u32),
  cz_ipv4_address_is_broadcast(u32),
  cz_ipv4_address_less_than(u32, u32),
  cz_ipv6_address_parse(&str),
  cz_ipv6_address_from_bytes(Vec<u8>, Option<String>),
  cz_ipv6_address_unspecified(),
  cz_ipv6_address_loopback(),
  cz_ipv6_address_bytes(IPv6Address),
  cz_ipv6_address_description(IPv6Address),
  cz_ipv6_address_is_unspecified(IPv6Address),
  cz_ipv6_address_is_loopback(IPv6Address),
  cz_ipv6_address_is_multicast(IPv6Address),
  cz_ipv6_address_is_link_local(IPv6Address),
  cz_ipv6_address_is_unique_local(IPv6Address),
  cz_ipv6_address_is_global_unicast(IPv6Address),
  cz_ipv6_address_is_documentation(IPv6Address),
  cz_ipv6_address_less_than(IPv6Address, IPv6Address),
  cz_ip_address_parse(&str),
  cz_ip_address_description(IpAddress),
  cz_ip_address_is_v4(IpAddress),
  cz_ip_address_is_v6(IpAddress),
  cz_ip_address_ipv4(IpAddress),
  cz_ip_address_ipv6(IpAddress),
  cz_ip_address_is_loopback(IpAddress),
  cz_ip_address_is_multicast(IpAddress),
  cz_ip_address_is_unspecified(IpAddress),
  cz_prefix_new(u8),
  cz_prefix_ipv4(u8),
  cz_prefix_ipv6(u8),
  cz_prefix_description(u8),
  cz_prefix_suffix_mask32(u8),
  cz_prefix_prefix_mask32(u8),
  cz_prefix_suffix_mask128(u8),
  cz_prefix_prefix_mask128(u8),
  cz_cidr_v4_parse(&str),
  cz_cidr_v4_new(u32, u8),
  cz_cidr_v4_from_range(u32, u32),
  cz_cidr_v4_lower(u32, u8),
  cz_cidr_v4_upper(u32, u8),
  cz_cidr_v4_gateway(u32, u8),
  cz_cidr_v4_contains(u32, u8, u32),
  cz_cidr_v4_description(u32, u8),
  cz_cidr_v6_parse(&str),
  cz_cidr_v6_new(IPv6Address, u8),
  cz_cidr_v6_from_range(IPv6Address, IPv6Address),
  cz_cidr_v6_lower(IPv6Address, u8),
  cz_cidr_v6_upper(IPv6Address, u8),
  cz_cidr_v6_gateway(IPv6Address, u8),
  cz_cidr_v6_contains(IPv6Address, u8, IPv6Address),
  cz_cidr_v6_description(IPv6Address, u8),
  cz_cidr_parse(&str),
  cz_cidr_new(IpAddress, u8),
  cz_cidr_from_range(IpAddress, IpAddress),
  cz_cidr_address(IpAddress, u8),
  cz_cidr_prefix(IpAddress, u8),
  cz_cidr_lower(IpAddress, u8),
  cz_cidr_upper(IpAddress, u8),
  cz_cidr_contains(IpAddress, u8, IpAddress),
  cz_cidr_description(IpAddress, u8),
  cz_mac_address_new(u64),
  cz_mac_address_from_bytes(Vec<u8>),
  cz_mac_address_parse(&str),
  cz_mac_address_bytes(u64),
  cz_mac_address_description(u64),
  cz_mac_address_is_locally_administered(u64),
  cz_mac_address_is_multicast(u64),
  cz_mac_address_ipv6_address(u64, IPv6Address),
  cz_mac_address_less_than(u64, u64),
  cz_capability_name_parse(&str),
  cz_capability_set_parse(&str),
  cz_terminal_new(i32, bool),
  cz_terminal_current(),
  cz_terminal_create(bool, u16, u16),
  cz_keychain_query_save(&str, Option<String>, &str, &str, &str),
  cz_keychain_query_delete(&str, Option<String>, &str),
  cz_keychain_query_get(&str, Option<String>, &str),
  cz_keychain_query_list(&str, Option<String>),
  cz_keychain_query_exists(&str, Option<String>, &str),
  cz_sysctl_by_name(&str),
  cz_file_info(&str),
);

pub(crate) fn cz_ext4_unpack(
  _unpacker: containerization::Ext4Unpacker,
  image: CzImage,
  _platform: containerization_oci::Platform,
  _at: &str,
  _progress: Progress,
) -> CzOutcome {
  match image.0 {}
}

impl CzReference {
  pub(crate) fn domain(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn resolved_domain(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn path(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn tag(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn digest(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn name(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn description(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn with_tag(&self, _tag: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn with_digest(&self, _digest: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn normalize(&self) {
    match self.0 {}
  }
}

impl CzRegistryClient {
  pub(crate) fn ping(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn resolve(&self, _name: &str, _tag: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn fetch_data(&self, _name: &str, _descriptor: containerization_oci::Descriptor) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn fetch_blob(
    &self,
    _name: &str,
    _descriptor: containerization_oci::Descriptor,
    _into: &str,
    _progress: Progress,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn catalog(&self, _prefix: Option<String>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn referrers(&self, _name: &str, _digest: &str, _artifact_type: Option<String>) -> CzOutcome {
    match self.0 {}
  }
}

impl CzAuthentication {
  pub(crate) fn duplicate(&self) -> CzAuthentication {
    match self.0 {}
  }

  pub(crate) fn token(&self) -> CzOutcome {
    match self.0 {}
  }
}

impl CzExt4Reader {
  pub(crate) fn super_block(&self) -> Vec<u8> {
    match self.0 {}
  }

  pub(crate) fn exists(&self, _path: &str, _follow_symlinks: bool) -> bool {
    match self.0 {}
  }

  pub(crate) fn stat(&self, _path: &str, _follow_symlinks: bool) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn list_directory(&self, _path: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn read_file(&self, _at: &str, _offset: u64, _count: Option<usize>, _follow_symlinks: bool) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn export(&self, _archive: &str) -> CzOutcome {
    match self.0 {}
  }
}

impl CzExt4Formatter {
  pub(crate) fn link(&self, _link: &str, _target: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn unlink(&self, _path: &str, _directory_whiteout: bool) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn create(
    &self,
    _path: &str,
    _link: Option<String>,
    _mode: u16,
    _access: f64,
    _modification: f64,
    _creation: f64,
    _now: f64,
    _has_buf: bool,
    _buf: Vec<u8>,
    _uid: Option<u32>,
    _gid: Option<u32>,
    _has_xattrs: bool,
    _xattr_names: Vec<String>,
    _xattr_lengths: Vec<u64>,
    _xattr_values: Vec<u8>,
    _recursion: bool,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn close(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn unpack(&self, _source: &str, _format: &str, _compression: &str, _progress: Progress) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn unpack_reader(&self, _reader: CzArchiveReader, _progress: Progress) -> CzOutcome {
    match self.0 {}
  }
}

impl CzReadStream {
  pub(crate) fn reset(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn data_stream(&self) -> CzDataStream {
    match self.0 {}
  }
}

impl CzDataStream {
  pub(crate) fn next(&self) -> CzOutcome {
    match self.0 {}
  }
}

impl CzWriteEntry {
  pub(crate) fn duplicate(&self) -> CzWriteEntry {
    match self.0 {}
  }

  pub(crate) fn has_size(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn size(&self) -> i64 {
    match self.0 {}
  }

  pub(crate) fn set_size(&self, _is_set: bool, _size: i64) {
    match self.0 {}
  }

  pub(crate) fn permissions(&self) -> u16 {
    match self.0 {}
  }

  pub(crate) fn set_permissions(&self, _permissions: u16) {
    match self.0 {}
  }

  pub(crate) fn has_owner(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn owner(&self) -> u32 {
    match self.0 {}
  }

  pub(crate) fn set_owner(&self, _is_set: bool, _owner: u32) {
    match self.0 {}
  }

  pub(crate) fn has_group(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn group(&self) -> u32 {
    match self.0 {}
  }

  pub(crate) fn set_group(&self, _is_set: bool, _group: u32) {
    match self.0 {}
  }

  pub(crate) fn hardlink(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn set_hardlink(&self, _hardlink: Option<String>) {
    match self.0 {}
  }

  pub(crate) fn hardlink_utf8(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn set_hardlink_utf8(&self, _hardlink: Option<String>) {
    match self.0 {}
  }

  pub(crate) fn strmode(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn file_type(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn set_file_type(&self, _file_type: &str) {
    match self.0 {}
  }

  pub(crate) fn has_content_access_date(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn content_access_date(&self) -> f64 {
    match self.0 {}
  }

  pub(crate) fn set_content_access_date(&self, _is_set: bool, _seconds: f64) {
    match self.0 {}
  }

  pub(crate) fn has_creation_date(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn creation_date(&self) -> f64 {
    match self.0 {}
  }

  pub(crate) fn set_creation_date(&self, _is_set: bool, _seconds: f64) {
    match self.0 {}
  }

  pub(crate) fn has_modification_date(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn modification_date(&self) -> f64 {
    match self.0 {}
  }

  pub(crate) fn set_modification_date(&self, _is_set: bool, _seconds: f64) {
    match self.0 {}
  }

  pub(crate) fn path(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn set_path(&self, _path: Option<String>) {
    match self.0 {}
  }

  pub(crate) fn path_utf8(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn set_path_utf8(&self, _path: Option<String>) {
    match self.0 {}
  }

  pub(crate) fn symlink_target(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn set_symlink_target(&self, _target: Option<String>) {
    match self.0 {}
  }

  pub(crate) fn xattrs(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn set_xattrs(&self, _names: Vec<String>, _lengths: Vec<u64>, _values: Vec<u8>) {
    match self.0 {}
  }
}

impl CzArchiveWriter {
  pub(crate) fn new_entry(&self) -> CzWriteEntry {
    match self.0 {}
  }

  pub(crate) fn open(&self, _file: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn open_with_file_descriptor(&self, _file_descriptor: i32) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn finish_encoding(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn make_transaction_writer(&self) -> CzArchiveWriterTransaction {
    match self.0 {}
  }

  pub(crate) fn write_entry(&self, _entry: CzWriteEntry, _has_data: bool, _data: Vec<u8>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn archive_directory(&self, _dir: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn archive(&self, _paths: Vec<String>, _base: &str) -> CzOutcome {
    match self.0 {}
  }
}

impl CzArchiveWriterTransaction {
  pub(crate) fn write_header(&self, _entry: CzWriteEntry) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn write_chunk(&self, _data: Vec<u8>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn finish(&self) -> CzOutcome {
    match self.0 {}
  }
}

impl CzArchiveReader {
  pub(crate) fn duplicate(&self) -> CzArchiveReader {
    match self.0 {}
  }

  pub(crate) fn make_iterator(&self) -> CzArchiveIterator {
    match self.0 {}
  }

  pub(crate) fn make_streaming_iterator(&self) -> CzStreamingIterator {
    match self.0 {}
  }

  pub(crate) fn throw_if_stream_failed(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn extract_contents(&self, _to: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn extract_file(&self, _path: &str) -> CzOutcome {
    match self.0 {}
  }
}

impl CzArchiveIterator {
  pub(crate) fn next(&self) -> CzOutcome {
    match self.0 {}
  }
}

impl CzStreamingIterator {
  pub(crate) fn next(&self) -> CzOutcome {
    match self.0 {}
  }
}

impl CzArchiveEntryReader {
  pub(crate) fn read(&self, _max_length: usize) -> CzOutcome {
    match self.0 {}
  }
}

impl CzTerminal {
  pub(crate) fn duplicate(&self) -> CzTerminal {
    match self.0 {}
  }

  pub(crate) fn handle(&self) -> i32 {
    match self.0 {}
  }

  pub(crate) fn write(&self, _data: Vec<u8>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn size(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn resize_from(&self, _pty: CzTerminal) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn resize_size(&self, _width: u16, _height: u16) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn resize(&self, _width: u16, _height: u16) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn setraw(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn enable_echo(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn disable_echo(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn close(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn reset(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn try_reset(&self) {
    match self.0 {}
  }
}

impl CzFileInfo {
  pub(crate) fn mode(&self) -> u16 {
    match self.0 {}
  }

  pub(crate) fn uid(&self) -> i64 {
    match self.0 {}
  }

  pub(crate) fn gid(&self) -> i64 {
    match self.0 {}
  }

  pub(crate) fn dev(&self) -> i64 {
    match self.0 {}
  }

  pub(crate) fn ino(&self) -> i64 {
    match self.0 {}
  }

  pub(crate) fn size(&self) -> i64 {
    match self.0 {}
  }

  pub(crate) fn path(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn is_directory(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn is_pipe(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn is_socket(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn is_link(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn is_regular_file(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn is_block(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn is_char(&self) -> bool {
    match self.0 {}
  }
}

impl CzLocalContentStore {
  pub(crate) fn get(&self, _digest: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn duplicate(&self) -> CzLocalContentStore {
    match self.0 {}
  }

  pub(crate) fn new_ingest_session(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn complete_ingest_session(&self, _id: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn cancel_ingest_session(&self, _id: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn image(&self, _description: image::Description) -> CzImage {
    match self.0 {}
  }

  pub(crate) fn delete_digests(&self, _digests: Vec<String>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn delete_keeping(&self, _keeping: Vec<String>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn total_allocated_size(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn image_store(&self, _path: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn ingest(&self, _body: Box<dyn FnOnce(String) -> bool>) -> CzOutcome {
    match self.0 {}
  }
}

impl CzContentWriter {
  pub(crate) fn write(&self, _data: Vec<u8>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn create(&self, _from: &str) -> CzOutcome {
    match self.0 {}
  }
}

impl CzContent {
  pub(crate) fn is_some(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn path(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn digest(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn size(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn data(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn data_range(&self, _offset: u64, _length: usize) -> CzOutcome {
    match self.0 {}
  }
}

impl CzImageStore {
  pub(crate) fn path(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn create_init_image(
    &self,
    _reference: &str,
    _rootfs: &str,
    _platform: containerization_oci::Platform,
    _label_keys: Vec<String>,
    _label_values: Vec<String>,
    _content_store: CzLocalContentStore,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn create_kernel_image(
    &self,
    _reference: &str,
    _binaries: Vec<containerization::Kernel>,
    _label_keys: Vec<String>,
    _label_values: Vec<String>,
    _content_store: CzLocalContentStore,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn get(&self, _reference: &str, _pull: bool) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn list(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn delete(&self, _reference: &str, _perform_cleanup: bool) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn tag(&self, _existing: &str, _new: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn pull(
    &self,
    _reference: &str,
    _has_platform: bool,
    _platform: containerization_oci::Platform,
    _insecure: bool,
    _auth: CzAuthentication,
    _progress: Progress,
    _max_concurrent_downloads: usize,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn push(
    &self,
    _reference: &str,
    _has_platform: bool,
    _platform: containerization_oci::Platform,
    _insecure: bool,
    _auth: CzAuthentication,
    _progress: Progress,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn push_all(
    &self,
    _references: Vec<String>,
    _has_platform: bool,
    _platform: containerization_oci::Platform,
    _insecure: bool,
    _auth: CzAuthentication,
    _max_concurrent_uploads: usize,
    _progress: Progress,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn get_init_image(&self, _reference: &str, _auth: CzAuthentication, _progress: Progress) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn save(
    &self,
    _references: Vec<String>,
    _out: &str,
    _has_platform: bool,
    _platform: containerization_oci::Platform,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn clean_up_orphaned_blobs(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn calculate_orphaned_blobs_size(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn create(&self, _description: image::Description) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn load(&self, _from: &str, _progress: Progress) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn container_manager(
    &self,
    _kernel: containerization::Kernel,
    _initfs: containerization::Mount,
    _rosetta: bool,
    _nested_virtualization: bool,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn container_manager_with_initfs_reference(
    &self,
    _kernel: containerization::Kernel,
    _initfs_reference: &str,
    _rosetta: bool,
    _nested_virtualization: bool,
  ) -> CzOutcome {
    match self.0 {}
  }
}

impl CzImages {
  pub(crate) fn len(&self) -> usize {
    match self.0 {}
  }

  pub(crate) fn at(&self, _index: usize) -> CzImage {
    match self.0 {}
  }
}

impl CzImage {
  pub(crate) fn duplicate(&self) -> CzImage {
    match self.0 {}
  }

  pub(crate) fn init_image(&self) -> CzInitImage {
    match self.0 {}
  }

  pub(crate) fn kernel_image(&self) -> CzKernelImage {
    match self.0 {}
  }

  pub(crate) fn reference(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn digest(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn media_type(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn descriptor(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn index(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn manifest(&self, _platform: containerization_oci::Platform) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn descriptor_for(&self, _platform: containerization_oci::Platform) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn config(&self, _platform: containerization_oci::Platform) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn referenced_digests(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn get_content(&self, _digest: &str) -> CzOutcome {
    match self.0 {}
  }
}

impl CzKernelImage {
  pub(crate) fn name(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn kernel(&self, _platform: containerization::SystemPlatform) -> CzOutcome {
    match self.0 {}
  }
}

impl CzInitImage {
  pub(crate) fn name(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn init_block(&self, _at: &str, _platform: containerization::SystemPlatform) -> CzOutcome {
    match self.0 {}
  }
}

impl CzContainerManager {
  pub(crate) fn create(
    &self,
    _id: &str,
    _image: CzImage,
    _options: container_manager::CreateOptions,
    _seed: linux_container::Configuration,
    _configuration: ConfigureContainer,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn create_with_rootfs(
    &self,
    _id: &str,
    _image: CzImage,
    _rootfs: containerization::Mount,
    _options: container_manager::RootfsCreateOptions,
    _seed: linux_container::Configuration,
    _configuration: ConfigureContainer,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn delete(&self, _id: &str) -> CzOutcome {
    match self.0 {}
  }
}

impl CzLinuxContainer {
  pub(crate) fn id(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn rootfs(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn writable_layer(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn config(&self, _seed: linux_container::Configuration, _receive: ConfigureContainer) {
    match self.0 {}
  }

  pub(crate) fn vm_cpus(&self) -> u32 {
    match self.0 {}
  }

  pub(crate) fn vm_memory_in_bytes(&self) -> u64 {
    match self.0 {}
  }

  pub(crate) fn exec_with(
    &self,
    _id: &str,
    _seed: containerization::LinuxProcessConfiguration,
    _configuration: ConfigureProcess,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn dial_vsock(&self, _port: u32) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn statistics(&self, _categories: i64) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn filesystem_operation(&self, _operation: FilesystemOperationKind, _path: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn copy_in(
    &self,
    _source: &str,
    _destination: &str,
    _mode: u32,
    _create_parents: bool,
    _chunk_size: usize,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn copy_out(
    &self,
    _source: &str,
    _destination: &str,
    _create_parents: bool,
    _chunk_size: usize,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn create(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn start(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn stop(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn kill(&self, _signal: i32) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn wait(&self, _timeout_in_seconds: Option<i64>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn resize(&self, _width: u16, _height: u16) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn exec(&self, _id: &str, _configuration: containerization::LinuxProcessConfiguration) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn close_stdin(&self) -> CzOutcome {
    match self.0 {}
  }
}

impl CzLinuxProcess {
  pub(crate) fn id(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn owning_container(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn pid(&self) -> i32 {
    match self.0 {}
  }

  pub(crate) fn start(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn kill(&self, _signal: i32) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn resize(&self, _width: u16, _height: u16) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn close_stdin(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn wait(&self, _timeout_in_seconds: Option<i64>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn delete(&self) -> CzOutcome {
    match self.0 {}
  }
}
