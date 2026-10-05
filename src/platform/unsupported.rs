//! Stand-ins for the bridge on every platform but macOS: the same types and
//! functions, so the wrappers are written once and a cross-platform workspace
//! still compiles.
//!
//! Nothing can make a Swift object here: every constructor's outcome is a
//! failure, and every handle is uninhabited, so its methods never run.

use super::Configure;
use super::Progress;
use crate::containerization;
use crate::containerization::container_manager;
use crate::containerization::image;
use crate::containerization::linux_container;
use crate::containerization_extras;
use crate::containerization_extras::IPv6Address;
use crate::containerization_extras::IpAddress;
use crate::containerization_oci;
use std::convert::Infallible;

const MACOS_ONLY: &str = "Containerization.framework is macOS only";

/// `ProgressEvent`, less its value.
pub(crate) enum ProgressKind {
  Items,
  TotalItems,
  Size,
  TotalSize,
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
  mount -> containerization::Mount,
  exit_code -> i32,
  exited_at -> f64,
  number -> u64,
  text -> String,
  strings -> Vec<String>,
  has_bytes -> bool,
  bytes -> Vec<u8>,
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
);

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

impl CzExt4Reader {
  pub(crate) fn export(&self, _archive: &str) -> CzOutcome {
    match self.0 {}
  }
}

impl CzLocalContentStore {
  pub(crate) fn get(&self, _digest: &str) -> CzOutcome {
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

  pub(crate) fn pull(&self, _reference: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn get_init_image(&self, _reference: &str) -> CzOutcome {
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
    _configuration: Configure,
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
    _configuration: Configure,
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
