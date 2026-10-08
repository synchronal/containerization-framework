//! Getters and setters for a container's configuration: its mounts, sockets,
//! boot log and seccomp profile.

use super::network::dns;
use super::network::hosts;
use super::network::interface_kind;
use super::network::nat;
use super::network::nat_interface;
use super::network::push_hosts_entry;
use super::network::set_ipv6_address;
use super::network::set_ipv6_gateway;
use super::network::vmnet;
use super::network::vmnet_interface;
use super::vm::boot_log_file;
use crate::bridge::accessors;
use crate::bridge::accessors::entry_at;
use crate::bridge::ffi;
use crate::containerization::container;
use crate::containerization::container::linux_container;
use crate::containerization::container::mount;
use crate::containerization::container::unix_socket_configuration;
use crate::containerization::network;
use crate::containerization::process;
use crate::containerization::vm;
use crate::containerization_oci::runtime;
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
  pub(crate) fn mount(&self) -> container::Mount {
    container::Mount {
      r#type: self.mount_type(),
      source: self.mount_source(),
      destination: self.mount_destination(),
      options: self.mount_options(),
      runtime_options: runtime_options_of(self.mount_runtime_kind(), self.mount_runtime_options()),
    }
  }

  /// The `Mount?` an outcome holds.
  pub(crate) fn optional_mount(&self) -> Option<container::Mount> {
    self.optional(Self::mount)
  }
}

impl container::Mount {
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

impl container::UnixSocketConfiguration {
  pub(crate) fn source(&self) -> String {
    accessors::path(&self.source)
  }

  pub(crate) fn destination(&self) -> String {
    accessors::path(&self.destination)
  }

  pub(crate) fn permissions(&self) -> Option<u32> {
    self.permissions
  }

  pub(crate) fn direction(&self) -> ffi::SocketDirection {
    self.direction.into()
  }
}

impl From<unix_socket_configuration::Direction> for ffi::SocketDirection {
  fn from(direction: unix_socket_configuration::Direction) -> Self {
    match direction {
      unix_socket_configuration::Direction::Into => Self::Into,
      unix_socket_configuration::Direction::OutOf => Self::OutOf,
    }
  }
}

impl From<ffi::SocketDirection> for unix_socket_configuration::Direction {
  fn from(direction: ffi::SocketDirection) -> Self {
    match direction {
      ffi::SocketDirection::Into => Self::Into,
      ffi::SocketDirection::OutOf => Self::OutOf,
    }
  }
}

impl linux_container::Configuration {
  pub(crate) fn process_mut(&mut self) -> &mut process::LinuxProcessConfiguration {
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
      .push(nat(ipv4_address, ipv4_prefix, ipv4_gateway, mac_address, mtu));
  }

  pub(crate) fn push_vmnet_interface(&mut self, interface: ffi::CzVmnetInterface) {
    self.interfaces.push(vmnet(interface));
  }

  /// Only after [`Self::push_interface`], for the interface it pushed.
  pub(crate) fn set_interface_ipv6_address(&mut self, high: u64, low: u64, zone: Option<String>, prefix: u8) {
    set_ipv6_address(&mut self.interfaces, high, low, zone, prefix);
  }

  /// Only after [`Self::push_interface`], for the interface it pushed.
  pub(crate) fn set_interface_ipv6_gateway(&mut self, high: u64, low: u64, zone: Option<String>) {
    set_ipv6_gateway(&mut self.interfaces, high, low, zone);
  }

  pub(crate) fn push_socket(
    &mut self,
    source: String,
    destination: String,
    permissions: Option<u32>,
    direction: ffi::SocketDirection,
  ) {
    self.sockets.push(container::UnixSocketConfiguration {
      source: PathBuf::from(source),
      destination: PathBuf::from(destination),
      permissions,
      direction: direction.into(),
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
    self.mounts.push(container::Mount {
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
    self.dns = Some(dns(nameservers, domain, search_domains, options));
  }

  pub(crate) fn set_hosts(&mut self, comment: Option<String>) {
    self.hosts = Some(hosts(comment));
  }

  /// Only after [`Self::set_hosts`].
  pub(crate) fn push_hosts_entry(&mut self, ip_address: String, hostnames: Vec<String>, comment: Option<String>) {
    push_hosts_entry(&mut self.hosts, ip_address, hostnames, comment);
  }

  pub(crate) fn set_boot_log_file(&mut self, path: String, append: bool) {
    self.boot_log = Some(boot_log_file(path, append));
  }

  pub(crate) fn set_boot_log_file_handle(&mut self, descriptor: i32) {
    self.boot_log = Some(vm::BootLog::FileHandle(descriptor));
  }

  /// `profile` holds the `LinuxSeccomp` of a `Profile`, or `Absent`.
  pub(crate) fn set_seccomp_profile(&mut self, mode: ffi::SeccompMode, profile: ffi::CzOutcome) {
    self.seccomp_profile = seccomp_profile(mode, profile);
  }

  pub(crate) fn process(&self) -> &process::LinuxProcessConfiguration {
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
    interface_kind(&self.interfaces[index])
  }

  /// Only when [`Self::interface_kind_at`] is `Nat`.
  pub(crate) fn nat_interface_at(&self, index: usize) -> &network::NatInterface {
    nat_interface(&self.interfaces[index])
  }

  /// Only when [`Self::interface_kind_at`] is `Vmnet`.
  pub(crate) fn vmnet_interface_at(&self, index: usize) -> ffi::CzVmnetInterface {
    vmnet_interface(&self.interfaces[index])
  }

  pub(crate) fn sockets_len(&self) -> usize {
    self.sockets.len()
  }

  pub(crate) fn sockets_at(&self, index: usize) -> &container::UnixSocketConfiguration {
    &self.sockets[index]
  }

  pub(crate) fn mounts_len(&self) -> usize {
    self.mounts.len()
  }

  pub(crate) fn mounts_at(&self, index: usize) -> &container::Mount {
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
  pub(crate) fn dns(&self) -> &network::Dns {
    accessors::present(&self.dns)
  }

  pub(crate) fn has_hosts(&self) -> bool {
    self.hosts.is_some()
  }

  /// Only when [`Self::has_hosts`].
  pub(crate) fn hosts(&self) -> &network::Hosts {
    accessors::present(&self.hosts)
  }

  pub(crate) fn virtualization(&self) -> bool {
    self.virtualization
  }

  pub(crate) fn has_boot_log(&self) -> bool {
    self.boot_log.is_some()
  }

  /// Only when [`Self::has_boot_log`].
  pub(crate) fn boot_log(&self) -> &vm::BootLog {
    accessors::present(&self.boot_log)
  }

  pub(crate) fn oci_runtime_path(&self) -> Option<&str> {
    self.oci_runtime_path.as_deref()
  }

  pub(crate) fn seccomp_mode(&self) -> ffi::SeccompMode {
    seccomp_mode(&self.seccomp_profile)
  }

  /// The custom profile. Swift asks only when [`Self::seccomp_mode`] is
  /// `Profile`.
  pub(crate) fn seccomp_profile(&self) -> &runtime::LinuxSeccomp {
    custom_seccomp(&self.seccomp_profile)
  }

  pub(crate) fn use_init(&self) -> bool {
    self.use_init
  }
}

/// `profile` holds the `LinuxSeccomp` of a `Profile`, or `Absent`.
pub(in super::super) fn seccomp_profile(
  mode: ffi::SeccompMode,
  profile: ffi::CzOutcome,
) -> linux_container::SeccompProfile {
  match mode {
    ffi::SeccompMode::Unconfined => linux_container::SeccompProfile::Unconfined,
    ffi::SeccompMode::Default => linux_container::SeccompProfile::Default,
    ffi::SeccompMode::Profile => linux_container::SeccompProfile::Profile(profile.seccomp()),
  }
}

pub(in super::super) fn seccomp_mode(profile: &linux_container::SeccompProfile) -> ffi::SeccompMode {
  match profile {
    linux_container::SeccompProfile::Unconfined => ffi::SeccompMode::Unconfined,
    linux_container::SeccompProfile::Default => ffi::SeccompMode::Default,
    linux_container::SeccompProfile::Profile(_) => ffi::SeccompMode::Profile,
  }
}

/// Only when the mode is `Profile`.
pub(in super::super) fn custom_seccomp(profile: &linux_container::SeccompProfile) -> &runtime::LinuxSeccomp {
  match profile {
    linux_container::SeccompProfile::Profile(profile) => profile,
    _ => panic!("Swift asks for the profile only when the mode is `Profile`"),
  }
}

#[cfg(test)]
mod tests {
  use crate::bridge::ffi;
  use crate::containerization::container;
  use crate::containerization::container::linux_container;
  use crate::containerization::container::mount;
  use crate::containerization::process;
  use crate::containerization_oci::runtime;

  #[test]
  fn copies_swifts_container_defaults() {
    assert_eq!(
      ffi::cz_linux_container_default_mounts().list(|mounts| mounts.list(ffi::CzOutcome::mount)),
      [
        container::LinuxContainer::default_mounts(),
        container::LinuxContainer::default_oci_mounts(),
      ]
    );
    assert_eq!(
      ffi::cz_linux_container_default_copy_chunk_size(),
      container::LinuxContainer::DEFAULT_COPY_CHUNK_SIZE
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
    let process = process::LinuxProcessConfiguration::new(&["/bin/echo", "hello"]);

    assert_eq!(process.arguments_len(), 2);
    assert_eq!(process.arguments_at(1), "hello");

    let mount = container::Mount {
      runtime_options: mount::RuntimeOptions::Virtiofs(vec!["cache=auto".into()]),
      ..container::Mount::share("/Users/user/workspace", "/workspace", &["ro"], &[])
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
    let profile = runtime::LinuxSeccomp::new(
      runtime::LinuxSeccompAction::ActErrno,
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

    let mount = container::Mount::block("ext4", "/images/data.ext4", "/data", &[], &[]);
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
