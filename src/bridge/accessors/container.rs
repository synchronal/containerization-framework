//! Getters and setters for Containerization's value types.

use super::entry_at;
use crate::bridge::ffi;
use crate::containerization;
use crate::containerization::container_statistics;
use crate::containerization::hosts;
use crate::containerization::kernel;
use crate::containerization::linux_container;
use crate::containerization::linux_rlimit;
use crate::containerization::mount;
use crate::containerization::system_platform;
use crate::containerization::unix_socket_configuration;
use crate::containerization::vmnet_network;
use crate::containerization_extras;
use crate::containerization_oci;
use crate::containerization_os;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn runtime_options_of(kind: ffi::RuntimeKind, options: Vec<String>) -> mount::RuntimeOptions {
  match kind {
    ffi::RuntimeKind::Virtioblk => mount::RuntimeOptions::Virtioblk(options),
    ffi::RuntimeKind::Virtiofs => mount::RuntimeOptions::Virtiofs(options),
    ffi::RuntimeKind::Shared => mount::RuntimeOptions::Shared,
    ffi::RuntimeKind::Generic => mount::RuntimeOptions::Any(options),
  }
}

impl ffi::CzOutcome {
  /// The `Mount` an outcome holds, read field by field.
  pub(crate) fn mount(&self) -> containerization::Mount {
    containerization::Mount {
      r#type: self.mount_type(),
      source: self.mount_source(),
      destination: self.mount_destination(),
      options: self.mount_options(),
      runtime_options: runtime_options_of(self.mount_runtime_kind(), self.mount_runtime_options()),
    }
  }

  /// The `Mount?` an outcome holds.
  pub(crate) fn optional_mount(&self) -> Option<containerization::Mount> {
    self.optional(Self::mount)
  }

  /// The `VmnetNetwork.Interface?` an outcome holds.
  pub(crate) fn optional_vmnet_interface(&self) -> Option<vmnet_network::Interface> {
    self.optional(|interface| vmnet_network::Interface {
      handle: interface.vmnet_interface(),
    })
  }

  /// The `ContainerStatistics` an outcome holds, each part read through the
  /// category that reports it.
  pub(crate) fn container_statistics(&self) -> containerization::ContainerStatistics {
    let part = |category: containerization::StatCategory| self.statistics_category(category.raw_value);

    containerization::ContainerStatistics {
      id: self.statistics_name(),
      process: part(containerization::StatCategory::PROCESS).optional(|process| {
        let [current, limit] = process.statistics_fields();
        container_statistics::ProcessStatistics { current, limit }
      }),
      memory: part(containerization::StatCategory::MEMORY).optional(|memory| {
        let [
          usage_bytes,
          limit_bytes,
          swap_usage_bytes,
          swap_limit_bytes,
          cache_bytes,
          kernel_stack_bytes,
          slab_bytes,
          page_faults,
          major_page_faults,
          inactive_file,
          anon,
          workingset_refault_anon,
          workingset_refault_file,
          pgsteal_kswapd,
          pgsteal_direct,
          pgsteal_khugepaged,
        ] = memory.statistics_fields();
        container_statistics::MemoryStatistics {
          usage_bytes,
          limit_bytes,
          swap_usage_bytes,
          swap_limit_bytes,
          cache_bytes,
          kernel_stack_bytes,
          slab_bytes,
          page_faults,
          major_page_faults,
          inactive_file,
          anon,
          workingset_refault_anon,
          workingset_refault_file,
          pgsteal_kswapd,
          pgsteal_direct,
          pgsteal_khugepaged,
        }
      }),
      cpu: part(containerization::StatCategory::CPU).optional(|cpu| {
        let [
          usage_usec,
          user_usec,
          system_usec,
          throttling_periods,
          throttled_periods,
          throttled_time_usec,
        ] = cpu.statistics_fields();
        container_statistics::CpuStatistics {
          usage_usec,
          user_usec,
          system_usec,
          throttling_periods,
          throttled_periods,
          throttled_time_usec,
        }
      }),
      block_io: part(containerization::StatCategory::BLOCK_IO).optional(|devices| {
        container_statistics::BlockIoStatistics {
          devices: devices.list(|device| {
            let [major, minor, read_bytes, write_bytes, read_operations, write_operations] = device.statistics_fields();
            container_statistics::BlockIoDevice {
              major,
              minor,
              read_bytes,
              write_bytes,
              read_operations,
              write_operations,
            }
          }),
        }
      }),
      networks: part(containerization::StatCategory::NETWORK).optional(|networks| {
        networks.list(|network| {
          let [
            received_packets,
            transmitted_packets,
            received_bytes,
            transmitted_bytes,
            received_errors,
            transmitted_errors,
          ] = network.statistics_fields();
          container_statistics::NetworkStatistics {
            interface: network.statistics_name(),
            received_packets,
            transmitted_packets,
            received_bytes,
            transmitted_bytes,
            received_errors,
            transmitted_errors,
          }
        })
      }),
      memory_events: part(containerization::StatCategory::MEMORY_EVENTS).optional(|events| {
        let [low, high, max, oom, oom_kill] = events.statistics_fields();
        container_statistics::MemoryEventStatistics {
          low,
          high,
          max,
          oom,
          oom_kill,
        }
      }),
      filesystem: part(containerization::StatCategory::FILESYSTEM).optional(|filesystems| {
        filesystems.list(|filesystem| {
          let [block_size, blocks, free_blocks, inodes, free_inodes] = filesystem.statistics_fields();
          container_statistics::FilesystemStatistics {
            mount_point: filesystem.statistics_name(),
            block_size,
            blocks,
            free_blocks,
            inodes,
            free_inodes,
          }
        })
      }),
    }
  }

  /// The `UInt64` fields of the statistics held, which number `N`.
  fn statistics_fields<const N: usize>(&self) -> [u64; N] {
    let fields = self.statistics_numbers();
    let count = fields.len();

    fields
      .try_into()
      .unwrap_or_else(|_| panic!("Swift's statistics have {count} numbers, not {N}"))
  }
}

impl From<containerization::FilesystemOperation> for ffi::FilesystemOperationKind {
  fn from(operation: containerization::FilesystemOperation) -> Self {
    match operation {
      containerization::FilesystemOperation::Freeze => Self::Freeze,
      containerization::FilesystemOperation::Thaw => Self::Thaw,
      containerization::FilesystemOperation::Trim => Self::Trim,
    }
  }
}

impl From<vmnet_network::Mode> for ffi::VmnetMode {
  fn from(mode: vmnet_network::Mode) -> Self {
    match mode {
      vmnet_network::Mode::Shared => Self::Shared,
      vmnet_network::Mode::Host => Self::Host,
      vmnet_network::Mode::Bridged => Self::Bridged,
    }
  }
}

impl From<kernel::LogLevel> for ffi::LogLevel {
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

impl ffi::CzOutcome {
  /// The `Hosts.Entry` an outcome holds, read field by field.
  pub(crate) fn hosts_entry(&self) -> hosts::Entry {
    hosts::Entry {
      ip_address: self.hosts_entry_ip_address(),
      hostnames: self.hosts_entry_hostnames(),
      comment: self.hosts_entry_comment(),
    }
  }

  /// A held `[String: Int32]`.
  pub(crate) fn int32_map(&self) -> BTreeMap<String, i32> {
    self.map_of(Self::int32)
  }
}

impl crate::containerization::container_manager::CreateOptions {
  pub(crate) fn rootfs_size_in_bytes(&self) -> u64 {
    self.rootfs_size_in_bytes
  }

  pub(crate) fn writable_layer_size_in_bytes(&self) -> Option<u64> {
    self.writable_layer_size_in_bytes
  }

  pub(crate) fn read_only(&self) -> bool {
    self.read_only
  }

  pub(crate) fn networking(&self) -> bool {
    self.networking
  }

  pub(crate) fn vm_cpus(&self) -> u32 {
    self.vm.cpus
  }

  pub(crate) fn vm_memory_in_bytes(&self) -> u64 {
    self.vm.memory_in_bytes
  }
}

impl containerization::Mount {
  pub(crate) fn mount_type(&self) -> &str {
    &self.r#type
  }

  pub(crate) fn source(&self) -> &str {
    &self.source
  }

  pub(crate) fn destination(&self) -> &str {
    &self.destination
  }

  pub(crate) fn options_len(&self) -> usize {
    self.options.len()
  }

  pub(crate) fn options_at(&self, index: usize) -> &str {
    &self.options[index]
  }

  pub(crate) fn runtime_kind(&self) -> ffi::RuntimeKind {
    match self.runtime_options {
      mount::RuntimeOptions::Virtioblk(_) => ffi::RuntimeKind::Virtioblk,
      mount::RuntimeOptions::Virtiofs(_) => ffi::RuntimeKind::Virtiofs,
      mount::RuntimeOptions::Shared => ffi::RuntimeKind::Shared,
      mount::RuntimeOptions::Any(_) => ffi::RuntimeKind::Generic,
    }
  }

  pub(crate) fn runtime_options_len(&self) -> usize {
    self.runtime_option_values().len()
  }

  pub(crate) fn runtime_options_at(&self, index: usize) -> &str {
    &self.runtime_option_values()[index]
  }

  /// What the runtime options carry, whatever their kind.
  fn runtime_option_values(&self) -> &[String] {
    match &self.runtime_options {
      mount::RuntimeOptions::Virtioblk(options)
      | mount::RuntimeOptions::Virtiofs(options)
      | mount::RuntimeOptions::Any(options) => options,
      mount::RuntimeOptions::Shared => &[],
    }
  }
}

impl containerization::UnixSocketConfiguration {
  pub(crate) fn source(&self) -> String {
    super::path(&self.source)
  }

  pub(crate) fn destination(&self) -> String {
    super::path(&self.destination)
  }

  pub(crate) fn permissions(&self) -> Option<u32> {
    self.permissions
  }

  pub(crate) fn direction(&self) -> ffi::SocketDirection {
    match self.direction {
      unix_socket_configuration::Direction::Into => ffi::SocketDirection::Into,
      unix_socket_configuration::Direction::OutOf => ffi::SocketDirection::OutOf,
    }
  }
}

impl containerization::NatInterface {
  pub(crate) fn ipv4_address_value(&self) -> u32 {
    self.ipv4_address.address.value
  }

  pub(crate) fn ipv4_prefix(&self) -> u8 {
    self.ipv4_address.prefix.length
  }

  pub(crate) fn ipv4_gateway(&self) -> Option<u32> {
    self.ipv4_gateway.map(|gateway| gateway.value)
  }

  pub(crate) fn has_ipv6_address(&self) -> bool {
    self.ipv6_address.is_some()
  }

  /// Only when [`Self::has_ipv6_address`].
  pub(crate) fn ipv6_address(&self) -> &containerization_extras::IPv6Address {
    &self.ipv6_cidr().address
  }

  /// Only when [`Self::has_ipv6_address`].
  pub(crate) fn ipv6_prefix(&self) -> u8 {
    self.ipv6_cidr().prefix.length
  }

  fn ipv6_cidr(&self) -> &containerization_extras::CIDRv6 {
    self
      .ipv6_address
      .as_ref()
      .expect("Swift asks for an IPv6 address only after has_ipv6_address")
  }

  pub(crate) fn has_ipv6_gateway(&self) -> bool {
    self.ipv6_gateway.is_some()
  }

  /// Only when [`Self::has_ipv6_gateway`].
  pub(crate) fn ipv6_gateway(&self) -> &containerization_extras::IPv6Address {
    self
      .ipv6_gateway
      .as_ref()
      .expect("Swift asks for an IPv6 gateway only after has_ipv6_gateway")
  }

  pub(crate) fn mac_address(&self) -> Option<u64> {
    self.mac_address.map(|address| address.value)
  }

  pub(crate) fn mtu(&self) -> u32 {
    self.mtu
  }
}

impl containerization::Dns {
  pub(crate) fn nameservers_len(&self) -> usize {
    self.nameservers.len()
  }

  pub(crate) fn nameservers_at(&self, index: usize) -> &str {
    &self.nameservers[index]
  }

  pub(crate) fn domain(&self) -> Option<&str> {
    self.domain.as_deref()
  }

  pub(crate) fn search_domains_len(&self) -> usize {
    self.search_domains.len()
  }

  pub(crate) fn search_domains_at(&self, index: usize) -> &str {
    &self.search_domains[index]
  }

  pub(crate) fn options_len(&self) -> usize {
    self.options.len()
  }

  pub(crate) fn options_at(&self, index: usize) -> &str {
    &self.options[index]
  }
}

impl hosts::Entry {
  pub(crate) fn ip_address(&self) -> &str {
    &self.ip_address
  }

  pub(crate) fn hostnames_len(&self) -> usize {
    self.hostnames.len()
  }

  pub(crate) fn hostnames_at(&self, index: usize) -> &str {
    &self.hostnames[index]
  }

  pub(crate) fn comment(&self) -> Option<&str> {
    self.comment.as_deref()
  }
}

impl containerization::Hosts {
  pub(crate) fn entries_len(&self) -> usize {
    self.entries.len()
  }

  pub(crate) fn entries_at(&self, index: usize) -> &hosts::Entry {
    &self.entries[index]
  }

  pub(crate) fn comment(&self) -> Option<&str> {
    self.comment.as_deref()
  }
}

impl containerization::BootLog {
  pub(crate) fn kind(&self) -> ffi::BootLogKind {
    match self {
      Self::File { .. } => ffi::BootLogKind::File,
      Self::FileHandle(_) => ffi::BootLogKind::FileHandle,
    }
  }

  /// Empty unless [`Self::kind`] is `File`.
  pub(crate) fn path(&self) -> String {
    match self {
      Self::File { path, .. } => super::path(path),
      Self::FileHandle(_) => String::new(),
    }
  }

  pub(crate) fn append(&self) -> bool {
    matches!(self, Self::File { append: true, .. })
  }

  /// `-1` unless [`Self::kind`] is `FileHandle`.
  pub(crate) fn file_handle(&self) -> i32 {
    match self {
      Self::FileHandle(descriptor) => *descriptor,
      Self::File { .. } => -1,
    }
  }
}

impl containerization_oci::User {
  pub(crate) fn uid(&self) -> u32 {
    self.uid
  }

  pub(crate) fn gid(&self) -> u32 {
    self.gid
  }

  pub(crate) fn umask(&self) -> Option<u32> {
    self.umask
  }

  pub(crate) fn additional_gids_len(&self) -> usize {
    self.additional_gids.len()
  }

  pub(crate) fn additional_gids_at(&self, index: usize) -> u32 {
    self.additional_gids[index]
  }

  pub(crate) fn username(&self) -> &str {
    &self.username
  }
}

impl containerization::LinuxCapabilities {
  pub(crate) fn set_len(&self, set: ffi::CapabilitySet) -> usize {
    self.set(set).len()
  }

  pub(crate) fn set_at(&self, set: ffi::CapabilitySet, index: usize) -> &str {
    self.set(set)[index].description()
  }

  fn set(&self, set: ffi::CapabilitySet) -> &[containerization_os::CapabilityName] {
    match set {
      ffi::CapabilitySet::Bounding => &self.bounding,
      ffi::CapabilitySet::Effective => &self.effective,
      ffi::CapabilitySet::Inheritable => &self.inheritable,
      ffi::CapabilitySet::Permitted => &self.permitted,
      ffi::CapabilitySet::Ambient => &self.ambient,
    }
  }
}

fn rlimit_kind(kind: linux_rlimit::Kind) -> ffi::RlimitKind {
  match kind {
    linux_rlimit::Kind::AddressSpace => ffi::RlimitKind::AddressSpace,
    linux_rlimit::Kind::CoreFileSize => ffi::RlimitKind::CoreFileSize,
    linux_rlimit::Kind::CpuTime => ffi::RlimitKind::CpuTime,
    linux_rlimit::Kind::DataSize => ffi::RlimitKind::DataSize,
    linux_rlimit::Kind::FileSize => ffi::RlimitKind::FileSize,
    linux_rlimit::Kind::Locks => ffi::RlimitKind::Locks,
    linux_rlimit::Kind::LockedMemory => ffi::RlimitKind::LockedMemory,
    linux_rlimit::Kind::MessageQueue => ffi::RlimitKind::MessageQueue,
    linux_rlimit::Kind::Nice => ffi::RlimitKind::Nice,
    linux_rlimit::Kind::OpenFiles => ffi::RlimitKind::OpenFiles,
    linux_rlimit::Kind::NumberOfProcesses => ffi::RlimitKind::NumberOfProcesses,
    linux_rlimit::Kind::ResidentSetSize => ffi::RlimitKind::ResidentSetSize,
    linux_rlimit::Kind::RealtimePriority => ffi::RlimitKind::RealtimePriority,
    linux_rlimit::Kind::RealtimeTimeout => ffi::RlimitKind::RealtimeTimeout,
    linux_rlimit::Kind::SignalsPending => ffi::RlimitKind::SignalsPending,
    linux_rlimit::Kind::StackSize => ffi::RlimitKind::StackSize,
  }
}

fn linux_rlimit_kind(kind: ffi::RlimitKind) -> linux_rlimit::Kind {
  match kind {
    ffi::RlimitKind::AddressSpace => linux_rlimit::Kind::AddressSpace,
    ffi::RlimitKind::CoreFileSize => linux_rlimit::Kind::CoreFileSize,
    ffi::RlimitKind::CpuTime => linux_rlimit::Kind::CpuTime,
    ffi::RlimitKind::DataSize => linux_rlimit::Kind::DataSize,
    ffi::RlimitKind::FileSize => linux_rlimit::Kind::FileSize,
    ffi::RlimitKind::Locks => linux_rlimit::Kind::Locks,
    ffi::RlimitKind::LockedMemory => linux_rlimit::Kind::LockedMemory,
    ffi::RlimitKind::MessageQueue => linux_rlimit::Kind::MessageQueue,
    ffi::RlimitKind::Nice => linux_rlimit::Kind::Nice,
    ffi::RlimitKind::OpenFiles => linux_rlimit::Kind::OpenFiles,
    ffi::RlimitKind::NumberOfProcesses => linux_rlimit::Kind::NumberOfProcesses,
    ffi::RlimitKind::ResidentSetSize => linux_rlimit::Kind::ResidentSetSize,
    ffi::RlimitKind::RealtimePriority => linux_rlimit::Kind::RealtimePriority,
    ffi::RlimitKind::RealtimeTimeout => linux_rlimit::Kind::RealtimeTimeout,
    ffi::RlimitKind::SignalsPending => linux_rlimit::Kind::SignalsPending,
    ffi::RlimitKind::StackSize => linux_rlimit::Kind::StackSize,
  }
}

impl containerization::LinuxProcessConfiguration {
  pub(crate) fn set_arguments(&mut self, arguments: Vec<String>) {
    self.arguments = arguments;
  }

  pub(crate) fn set_environment_variables(&mut self, environment_variables: Vec<String>) {
    self.environment_variables = environment_variables;
  }

  pub(crate) fn set_working_directory(&mut self, working_directory: String) {
    self.working_directory = working_directory;
  }

  pub(crate) fn set_user(
    &mut self,
    uid: u32,
    gid: u32,
    umask: Option<u32>,
    additional_gids: Vec<u32>,
    username: String,
  ) {
    self.user = containerization_oci::User {
      uid,
      gid,
      umask,
      additional_gids,
      username,
    };
  }

  pub(crate) fn set_no_new_privileges(&mut self, no_new_privileges: bool) {
    self.no_new_privileges = no_new_privileges;
  }

  pub(crate) fn set_capabilities(
    &mut self,
    bounding: Vec<String>,
    effective: Vec<String>,
    inheritable: Vec<String>,
    permitted: Vec<String>,
    ambient: Vec<String>,
  ) {
    let names = |set: Vec<String>| {
      set
        .iter()
        .map(|description| containerization_os::CapabilityName::from_description(description))
        .collect()
    };

    self.capabilities = containerization::LinuxCapabilities {
      bounding: names(bounding),
      effective: names(effective),
      inheritable: names(inheritable),
      permitted: names(permitted),
      ambient: names(ambient),
    };
  }

  pub(crate) fn set_terminal(&mut self, terminal: bool) {
    self.terminal = terminal;
  }

  pub(crate) fn clear_rlimits(&mut self) {
    self.rlimits.clear();
  }

  pub(crate) fn push_rlimit(&mut self, kind: ffi::RlimitKind, hard: u64, soft: u64) {
    self.rlimits.push(containerization::LinuxRLimit {
      kind: linux_rlimit_kind(kind),
      hard,
      soft,
    });
  }

  pub(crate) fn arguments_len(&self) -> usize {
    self.arguments.len()
  }

  pub(crate) fn arguments_at(&self, index: usize) -> &str {
    &self.arguments[index]
  }

  pub(crate) fn environment_variables_len(&self) -> usize {
    self.environment_variables.len()
  }

  pub(crate) fn environment_variables_at(&self, index: usize) -> &str {
    &self.environment_variables[index]
  }

  pub(crate) fn working_directory(&self) -> &str {
    &self.working_directory
  }

  pub(crate) fn user(&self) -> &containerization_oci::User {
    &self.user
  }

  pub(crate) fn rlimits_len(&self) -> usize {
    self.rlimits.len()
  }

  pub(crate) fn rlimit_kind_at(&self, index: usize) -> ffi::RlimitKind {
    rlimit_kind(self.rlimits[index].kind)
  }

  pub(crate) fn rlimit_hard_at(&self, index: usize) -> u64 {
    self.rlimits[index].hard
  }

  pub(crate) fn rlimit_soft_at(&self, index: usize) -> u64 {
    self.rlimits[index].soft
  }

  pub(crate) fn no_new_privileges(&self) -> bool {
    self.no_new_privileges
  }

  pub(crate) fn capabilities(&self) -> &containerization::LinuxCapabilities {
    &self.capabilities
  }

  pub(crate) fn terminal(&self) -> bool {
    self.terminal
  }

  pub(crate) fn stdin(&self) -> Option<i32> {
    self.stdin
  }

  pub(crate) fn stdout(&self) -> Option<i32> {
    self.stdout
  }

  pub(crate) fn stderr(&self) -> Option<i32> {
    self.stderr
  }
}

impl linux_container::Configuration {
  pub(crate) fn process_mut(&mut self) -> &mut containerization::LinuxProcessConfiguration {
    &mut self.process
  }

  pub(crate) fn set_cpus(&mut self, cpus: u32) {
    self.cpus = cpus;
  }

  pub(crate) fn set_memory_in_bytes(&mut self, memory_in_bytes: u64) {
    self.memory_in_bytes = memory_in_bytes;
  }

  pub(crate) fn set_hostname(&mut self, hostname: Option<String>) {
    self.hostname = hostname;
  }

  pub(crate) fn set_masked_paths(&mut self, masked_paths: Vec<String>) {
    self.masked_paths = masked_paths;
  }

  pub(crate) fn set_readonly_paths(&mut self, readonly_paths: Vec<String>) {
    self.readonly_paths = readonly_paths;
  }

  pub(crate) fn set_virtualization(&mut self, virtualization: bool) {
    self.virtualization = virtualization;
  }

  pub(crate) fn set_oci_runtime_path(&mut self, oci_runtime_path: Option<String>) {
    self.oci_runtime_path = oci_runtime_path;
  }

  pub(crate) fn set_use_init(&mut self, use_init: bool) {
    self.use_init = use_init;
  }

  pub(crate) fn clear_sysctl(&mut self) {
    self.sysctl.clear();
  }

  pub(crate) fn clear_interfaces(&mut self) {
    self.interfaces.clear();
  }

  pub(crate) fn clear_sockets(&mut self) {
    self.sockets.clear();
  }

  pub(crate) fn clear_mounts(&mut self) {
    self.mounts.clear();
  }

  pub(crate) fn clear_dns(&mut self) {
    self.dns = None;
  }

  pub(crate) fn clear_hosts(&mut self) {
    self.hosts = None;
  }

  pub(crate) fn clear_boot_log(&mut self) {
    self.boot_log = None;
  }

  pub(crate) fn insert_sysctl(&mut self, key: String, value: String) {
    self.sysctl.insert(key, value);
  }

  pub(crate) fn push_interface(
    &mut self,
    ipv4_address: u32,
    ipv4_prefix: u8,
    ipv4_gateway: Option<u32>,
    mac_address: Option<u64>,
    mtu: u32,
  ) {
    self
      .interfaces
      .push(containerization::Interface::Nat(containerization::NatInterface {
        ipv4_address: containerization_extras::CIDRv4 {
          address: containerization_extras::IPv4Address::new(ipv4_address),
          prefix: containerization_extras::Prefix { length: ipv4_prefix },
        },
        ipv4_gateway: ipv4_gateway.map(containerization_extras::IPv4Address::new),
        ipv6_address: None,
        ipv6_gateway: None,
        mac_address: mac_address.map(|value| containerization_extras::MACAddress { value }),
        mtu,
      }));
  }

  pub(crate) fn push_vmnet_interface(&mut self, interface: ffi::CzVmnetInterface) {
    self
      .interfaces
      .push(containerization::Interface::Vmnet(vmnet_network::Interface {
        handle: interface,
      }));
  }

  /// Only after [`Self::push_interface`], for the interface it pushed.
  pub(crate) fn set_interface_ipv6_address(&mut self, high: u64, low: u64, zone: Option<String>, prefix: u8) {
    self.last_interface().ipv6_address = Some(containerization_extras::CIDRv6 {
      address: containerization_extras::IPv6Address::from_halves(high, low, zone),
      prefix: containerization_extras::Prefix { length: prefix },
    });
  }

  /// Only after [`Self::push_interface`], for the interface it pushed.
  pub(crate) fn set_interface_ipv6_gateway(&mut self, high: u64, low: u64, zone: Option<String>) {
    self.last_interface().ipv6_gateway = Some(containerization_extras::IPv6Address::from_halves(high, low, zone));
  }

  fn last_interface(&mut self) -> &mut containerization::NatInterface {
    match self.interfaces.last_mut() {
      Some(containerization::Interface::Nat(interface)) => interface,
      _ => unreachable!("Swift sets an interface's IPv6 addresses only after push_interface"),
    }
  }

  pub(crate) fn push_socket(
    &mut self,
    source: String,
    destination: String,
    permissions: Option<u32>,
    direction: ffi::SocketDirection,
  ) {
    self
      .sockets
      .push(containerization::UnixSocketConfiguration {
        source: PathBuf::from(source),
        destination: PathBuf::from(destination),
        permissions,
        direction: match direction {
          ffi::SocketDirection::Into => unix_socket_configuration::Direction::Into,
          ffi::SocketDirection::OutOf => unix_socket_configuration::Direction::OutOf,
        },
      });
  }

  pub(crate) fn push_mount(
    &mut self,
    mount_type: String,
    source: String,
    destination: String,
    options: Vec<String>,
    kind: ffi::RuntimeKind,
    runtime_options: Vec<String>,
  ) {
    self.mounts.push(containerization::Mount {
      r#type: mount_type,
      source,
      destination,
      options,
      runtime_options: runtime_options_of(kind, runtime_options),
    });
  }

  pub(crate) fn set_dns(
    &mut self,
    nameservers: Vec<String>,
    domain: Option<String>,
    search_domains: Vec<String>,
    options: Vec<String>,
  ) {
    self.dns = Some(containerization::Dns {
      nameservers,
      domain,
      search_domains,
      options,
    });
  }

  pub(crate) fn set_hosts(&mut self, comment: Option<String>) {
    self.hosts = Some(containerization::Hosts {
      entries: Vec::new(),
      comment,
    });
  }

  /// Only after [`Self::set_hosts`].
  pub(crate) fn push_hosts_entry(&mut self, ip_address: String, hostnames: Vec<String>, comment: Option<String>) {
    self
      .hosts
      .as_mut()
      .expect("Swift adds a hosts entry only after set_hosts")
      .entries
      .push(hosts::Entry {
        ip_address,
        hostnames,
        comment,
      });
  }

  pub(crate) fn set_boot_log_file(&mut self, path: String, append: bool) {
    self.boot_log = Some(containerization::BootLog::File {
      path: PathBuf::from(path),
      append,
    });
  }

  pub(crate) fn set_boot_log_file_handle(&mut self, descriptor: i32) {
    self.boot_log = Some(containerization::BootLog::FileHandle(descriptor));
  }

  /// `profile` holds the `LinuxSeccomp` of a `Profile`, or `Absent`.
  pub(crate) fn set_seccomp_profile(&mut self, mode: ffi::SeccompMode, profile: ffi::CzOutcome) {
    self.seccomp_profile = match mode {
      ffi::SeccompMode::Unconfined => linux_container::SeccompProfile::Unconfined,
      ffi::SeccompMode::Default => linux_container::SeccompProfile::Default,
      ffi::SeccompMode::Profile => linux_container::SeccompProfile::Profile(profile.seccomp()),
    };
  }

  pub(crate) fn process(&self) -> &containerization::LinuxProcessConfiguration {
    &self.process
  }

  pub(crate) fn cpus(&self) -> u32 {
    self.cpus
  }

  pub(crate) fn memory_in_bytes(&self) -> u64 {
    self.memory_in_bytes
  }

  pub(crate) fn hostname(&self) -> Option<&str> {
    self.hostname.as_deref()
  }

  pub(crate) fn sysctl_len(&self) -> usize {
    self.sysctl.len()
  }

  pub(crate) fn sysctl_key_at(&self, index: usize) -> &str {
    entry_at(&self.sysctl, index).0
  }

  pub(crate) fn sysctl_value_at(&self, index: usize) -> &str {
    entry_at(&self.sysctl, index).1
  }

  pub(crate) fn interfaces_len(&self) -> usize {
    self.interfaces.len()
  }

  pub(crate) fn interface_kind_at(&self, index: usize) -> ffi::InterfaceKind {
    match self.interfaces[index] {
      containerization::Interface::Nat(_) => ffi::InterfaceKind::Nat,
      containerization::Interface::Vmnet(_) => ffi::InterfaceKind::Vmnet,
    }
  }

  /// Only when [`Self::interface_kind_at`] is `Nat`.
  pub(crate) fn nat_interface_at(&self, index: usize) -> &containerization::NatInterface {
    match &self.interfaces[index] {
      containerization::Interface::Nat(interface) => interface,
      containerization::Interface::Vmnet(_) => unreachable!("Swift asks for a NAT interface only of its kind"),
    }
  }

  /// Only when [`Self::interface_kind_at`] is `Vmnet`: a handle on it for
  /// Swift to keep.
  pub(crate) fn vmnet_interface_at(&self, index: usize) -> ffi::CzVmnetInterface {
    match &self.interfaces[index] {
      containerization::Interface::Vmnet(interface) => interface.handle.duplicate(),
      containerization::Interface::Nat(_) => unreachable!("Swift asks for a vmnet interface only of its kind"),
    }
  }

  pub(crate) fn sockets_len(&self) -> usize {
    self.sockets.len()
  }

  pub(crate) fn sockets_at(&self, index: usize) -> &containerization::UnixSocketConfiguration {
    &self.sockets[index]
  }

  pub(crate) fn mounts_len(&self) -> usize {
    self.mounts.len()
  }

  pub(crate) fn mounts_at(&self, index: usize) -> &containerization::Mount {
    &self.mounts[index]
  }

  pub(crate) fn masked_paths_len(&self) -> usize {
    self.masked_paths.len()
  }

  pub(crate) fn masked_paths_at(&self, index: usize) -> &str {
    &self.masked_paths[index]
  }

  pub(crate) fn readonly_paths_len(&self) -> usize {
    self.readonly_paths.len()
  }

  pub(crate) fn readonly_paths_at(&self, index: usize) -> &str {
    &self.readonly_paths[index]
  }

  pub(crate) fn has_dns(&self) -> bool {
    self.dns.is_some()
  }

  /// Only when [`Self::has_dns`].
  pub(crate) fn dns(&self) -> &containerization::Dns {
    self
      .dns
      .as_ref()
      .expect("Swift asks for DNS only after has_dns")
  }

  pub(crate) fn has_hosts(&self) -> bool {
    self.hosts.is_some()
  }

  /// Only when [`Self::has_hosts`].
  pub(crate) fn hosts(&self) -> &containerization::Hosts {
    self
      .hosts
      .as_ref()
      .expect("Swift asks for hosts only after has_hosts")
  }

  pub(crate) fn virtualization(&self) -> bool {
    self.virtualization
  }

  pub(crate) fn has_boot_log(&self) -> bool {
    self.boot_log.is_some()
  }

  /// Only when [`Self::has_boot_log`].
  pub(crate) fn boot_log(&self) -> &containerization::BootLog {
    self
      .boot_log
      .as_ref()
      .expect("Swift asks for a boot log only after has_boot_log")
  }

  pub(crate) fn oci_runtime_path(&self) -> Option<&str> {
    self.oci_runtime_path.as_deref()
  }

  pub(crate) fn seccomp_mode(&self) -> ffi::SeccompMode {
    match self.seccomp_profile {
      linux_container::SeccompProfile::Unconfined => ffi::SeccompMode::Unconfined,
      linux_container::SeccompProfile::Default => ffi::SeccompMode::Default,
      linux_container::SeccompProfile::Profile(_) => ffi::SeccompMode::Profile,
    }
  }

  /// The custom profile. Swift asks only when [`Self::seccomp_mode`] is
  /// `Profile`.
  pub(crate) fn seccomp_profile(&self) -> &containerization_oci::LinuxSeccomp {
    match &self.seccomp_profile {
      linux_container::SeccompProfile::Profile(profile) => profile,
      _ => panic!("Swift asks for the profile only when the mode is `Profile`"),
    }
  }

  pub(crate) fn use_init(&self) -> bool {
    self.use_init
  }
}

impl containerization::LinuxRLimit {
  pub(crate) fn kind(&self) -> ffi::RlimitKind {
    rlimit_kind(self.kind)
  }

  pub(crate) fn hard(&self) -> u64 {
    self.hard
  }

  pub(crate) fn soft(&self) -> u64 {
    self.soft
  }
}

impl containerization::SystemPlatform {
  pub(crate) fn os(&self) -> ffi::PlatformOs {
    match self.os {
      system_platform::Os::Linux => ffi::PlatformOs::Linux,
      system_platform::Os::Darwin => ffi::PlatformOs::Darwin,
    }
  }

  pub(crate) fn architecture(&self) -> ffi::PlatformArchitecture {
    match self.architecture {
      system_platform::Architecture::Arm64 => ffi::PlatformArchitecture::Arm64,
      system_platform::Architecture::Amd64 => ffi::PlatformArchitecture::Amd64,
    }
  }
}

impl containerization::Kernel {
  pub(crate) fn path(&self) -> String {
    super::path(&self.path)
  }

  pub(crate) fn platform(&self) -> &containerization::SystemPlatform {
    &self.platform
  }

  pub(crate) fn kernel_args_len(&self) -> usize {
    self.command_line.kernel_args.len()
  }

  pub(crate) fn kernel_args_at(&self, index: usize) -> &str {
    &self.command_line.kernel_args[index]
  }

  pub(crate) fn init_args_len(&self) -> usize {
    self.command_line.init_args.len()
  }

  pub(crate) fn init_args_at(&self, index: usize) -> &str {
    &self.command_line.init_args[index]
  }
}

#[cfg(test)]
mod tests {
  use crate::bridge::ffi;
  use crate::containerization;
  use crate::containerization::linux_container;
  use crate::containerization::linux_rlimit;
  use crate::containerization::mount;
  use crate::containerization::signal;
  use crate::containerization::system_platform;
  use crate::containerization_oci;
  use crate::containerization_os;

  #[test]
  fn copies_swifts_rlimit_kinds() {
    assert_eq!(
      ffi::cz_linux_rlimit_kind_descriptions(),
      linux_rlimit::Kind::ALL
        .iter()
        .map(|kind| kind.description())
        .collect::<Vec<_>>()
    );
  }

  #[test]
  fn copies_swifts_capability_presets() {
    let oci = |capabilities: containerization::LinuxCapabilities| {
      let set = |set: Vec<containerization_os::CapabilityName>| {
        (!set.is_empty()).then(|| {
          set
            .iter()
            .map(|name| name.description().to_string())
            .collect()
        })
      };

      containerization_oci::LinuxCapabilities {
        bounding: set(capabilities.bounding),
        effective: set(capabilities.effective),
        inheritable: set(capabilities.inheritable),
        permitted: set(capabilities.permitted),
        ambient: set(capabilities.ambient),
      }
    };

    assert_eq!(
      ffi::cz_linux_capabilities_presets().list(ffi::CzOutcome::oci_linux_capabilities),
      [
        oci(containerization::LinuxCapabilities::all_capabilities()),
        oci(containerization::LinuxCapabilities::default_oci_capabilities()),
      ]
    );
  }

  #[test]
  fn copies_swifts_signals() {
    let raw_values = |signals: &[containerization::Signal]| {
      signals
        .iter()
        .map(|signal| signal.raw_value)
        .collect::<Vec<_>>()
    };

    let mut linux = raw_values(signal::linux::ALL);
    linux.push(signal::linux::rtmin(0).raw_value);

    assert_eq!(ffi::cz_signal_linux_values(), linux);
    assert_eq!(ffi::cz_signal_darwin_values(), raw_values(signal::darwin::ALL));
  }

  #[test]
  fn copies_swifts_system_platform_raw_values() {
    let os = system_platform::Os::ALL_CASES.iter().map(|os| os.as_str());
    let architectures = system_platform::Architecture::ALL_CASES
      .iter()
      .map(|architecture| architecture.as_str());

    assert_eq!(
      ffi::cz_system_platform_raw_values(),
      os.chain(architectures).collect::<Vec<_>>()
    );
  }

  #[test]
  fn copies_swifts_container_defaults() {
    assert_eq!(
      ffi::cz_linux_container_default_mounts().list(|mounts| mounts.list(ffi::CzOutcome::mount)),
      [
        containerization::LinuxContainer::default_mounts(),
        containerization::LinuxContainer::default_oci_mounts(),
      ]
    );
    assert_eq!(
      ffi::cz_linux_container_default_copy_chunk_size(),
      containerization::LinuxContainer::DEFAULT_COPY_CHUNK_SIZE
    );
  }

  #[test]
  fn reads_an_absent_option_as_absent() {
    let configuration = linux_container::Configuration::default();

    assert!(!configuration.has_dns());
    assert!(!configuration.has_hosts());
    assert!(!configuration.has_boot_log());
    assert_eq!(configuration.process().stdin(), None);
    assert!(matches!(configuration.seccomp_mode(), ffi::SeccompMode::Unconfined));
  }

  #[test]
  fn reads_a_list_as_its_length_and_each_element() {
    let process = containerization::LinuxProcessConfiguration::new(&["/bin/echo", "hello"]);

    assert_eq!(process.arguments_len(), 2);
    assert_eq!(process.arguments_at(1), "hello");

    let mount = containerization::Mount {
      runtime_options: mount::RuntimeOptions::Virtiofs(vec!["cache=auto".into()]),
      ..containerization::Mount::share("/Users/user/workspace", "/workspace", &["ro"], &[])
    };

    assert_eq!(mount.options_len(), 1);
    assert_eq!(mount.options_at(0), "ro");
    assert_eq!(mount.runtime_options_len(), 1);
    assert_eq!(mount.runtime_options_at(0), "cache=auto");
  }

  #[test]
  fn reads_a_map_as_its_length_and_each_entry() {
    let configuration = linux_container::Configuration {
      sysctl: [
        ("vm.swappiness".to_string(), "10".to_string()),
        ("net.core.somaxconn".to_string(), "4096".to_string()),
      ]
      .into(),
      ..Default::default()
    };

    assert_eq!(configuration.sysctl_len(), 2);
    assert_eq!(configuration.sysctl_key_at(0), "net.core.somaxconn");
    assert_eq!(configuration.sysctl_value_at(0), "4096");
    assert_eq!(configuration.sysctl_key_at(1), "vm.swappiness");
    assert_eq!(configuration.sysctl_value_at(1), "10");
  }

  #[test]
  fn reads_an_enum_as_its_kind_and_what_it_carries() {
    let profile = containerization_oci::LinuxSeccomp::new(
      containerization_oci::LinuxSeccompAction::ActErrno,
      None,
      Vec::new(),
      Vec::new(),
      "",
      "",
      Vec::new(),
    );
    let configuration = linux_container::Configuration {
      seccomp_profile: linux_container::SeccompProfile::Profile(profile.clone()),
      ..Default::default()
    };

    assert!(matches!(configuration.seccomp_mode(), ffi::SeccompMode::Profile));
    assert_eq!(configuration.seccomp_profile(), &profile);

    let mount = containerization::Mount::block("ext4", "/images/data.ext4", "/data", &[], &[]);
    assert!(matches!(mount.runtime_kind(), ffi::RuntimeKind::Virtioblk));
    assert_eq!(mount.mount_type(), "ext4");
  }

  #[test]
  fn fills_what_it_reads() {
    let mut configuration = linux_container::Configuration::default();
    configuration.clear_mounts();
    configuration.push_mount(
      "virtiofs".into(),
      "/host".into(),
      "/guest".into(),
      vec!["ro".into()],
      ffi::RuntimeKind::Shared,
      Vec::new(),
    );
    configuration.set_hosts(None);
    configuration.push_hosts_entry("127.0.0.1".into(), vec!["localhost".into()], None);
    configuration.push_interface(0xc0a8_4002, 24, Some(0xc0a8_4001), Some(0x0242_ac11_0002), 1500);
    configuration.set_interface_ipv6_address(0xfd00_00cf_0000_0000, 2, Some("eth0".into()), 64);
    configuration.set_interface_ipv6_gateway(0xfd00_00cf_0000_0000, 1, None);
    configuration
      .process_mut()
      .set_arguments(vec!["/bin/true".into()]);

    assert_eq!(configuration.mounts_len(), 1, "Swift's mounts replace the defaults");
    assert!(matches!(
      configuration.mounts_at(0).runtime_kind(),
      ffi::RuntimeKind::Shared
    ));
    assert!(matches!(configuration.interface_kind_at(0), ffi::InterfaceKind::Nat));
    let interface = configuration.nat_interface_at(0);
    assert_eq!(interface.ipv4_address_value(), 0xc0a8_4002);
    assert_eq!(interface.ipv4_prefix(), 24);
    assert_eq!(interface.ipv4_gateway(), Some(0xc0a8_4001));
    assert_eq!(interface.mac_address(), Some(0x0242_ac11_0002));
    assert_eq!(interface.ipv6_address().value_high(), 0xfd00_00cf_0000_0000);
    assert_eq!(interface.ipv6_address().value_low(), 2);
    assert_eq!(interface.ipv6_address().zone(), Some("eth0"));
    assert_eq!(interface.ipv6_prefix(), 64);
    assert_eq!(interface.ipv6_gateway().value_low(), 1);
    assert_eq!(configuration.hosts().entries_len(), 1);
    assert_eq!(configuration.process().arguments_at(0), "/bin/true");
  }
}
