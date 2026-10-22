//! Getters and setters for a container's configuration: its mounts, sockets,
//! boot log and seccomp profile.

use crate::bridge::ffi;
use crate::containerization::container;
use crate::containerization::container::linux_container;
use crate::containerization::container::mount;
use crate::containerization::network;
use crate::containerization::process;
use crate::containerization::vm;
use crate::containerization_oci::runtime;

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
  getters! {
    str r#type => mount_type;
    str source;
    str destination;
    list options: str => options_len, options_at;
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

  pub(crate) fn clear_unified(&mut self) {
    self.unified.clear();
  }

  network_setters!();
  boot_log_setters!();
  seccomp_accessors!();
  interface_getters!();

  pub(crate) fn clear_sockets(&mut self) {
    self.sockets.clear();
  }

  pub(crate) fn clear_mounts(&mut self) {
    self.mounts.clear();
  }

  pub(crate) fn insert_sysctl(&mut self, key: String, value: String) {
    self.sysctl.insert(key, value);
  }

  pub(crate) fn insert_unified(&mut self, key: String, value: String) {
    self.unified.insert(key, value);
  }

  pub(crate) fn push_socket(&mut self, socket: ffi::CzUnixSocketConfiguration) {
    self
      .sockets
      .push(container::UnixSocketConfiguration { handle: socket });
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

  getters! {
    borrow process: process::LinuxProcessConfiguration;
    copy cpus: u32;
    copy memory_in_bytes: u64;
    optional_str hostname;
    map sysctl => sysctl_len, sysctl_key_at, sysctl_value_at;
    list mounts: container::Mount => mounts_len, mounts_at;
    list masked_paths: str => masked_paths_len, masked_paths_at;
    list readonly_paths: str => readonly_paths_len, readonly_paths_at;
    present dns: network::DNS => has_dns;
    present hosts: network::Hosts => has_hosts;
    copy virtualization: bool;
    present boot_log: vm::BootLog => has_boot_log;
    optional_str oci_runtime_path;
    copy use_init: bool;
    map unified => unified_len, unified_key_at, unified_value_at;
  }

  pub(crate) fn sockets_len(&self) -> usize {
    self.sockets.len()
  }

  pub(crate) fn sockets_at(&self, index: usize) -> ffi::CzUnixSocketConfiguration {
    self.sockets[index].handle.duplicate()
  }
}

/// `profile` holds the `LinuxSeccomp` of a `Profile`, or `Absent`.
pub(super) fn seccomp_profile(
  mode: ffi::SeccompMode,
  profile: ffi::CzOutcome,
) -> linux_container::configuration::SeccompProfile {
  match mode {
    ffi::SeccompMode::Unconfined => linux_container::configuration::SeccompProfile::Unconfined,
    ffi::SeccompMode::Default => linux_container::configuration::SeccompProfile::Default,
    ffi::SeccompMode::Profile => linux_container::configuration::SeccompProfile::Profile(profile.seccomp()),
  }
}

pub(super) fn seccomp_mode(profile: &linux_container::configuration::SeccompProfile) -> ffi::SeccompMode {
  match profile {
    linux_container::configuration::SeccompProfile::Unconfined => ffi::SeccompMode::Unconfined,
    linux_container::configuration::SeccompProfile::Default => ffi::SeccompMode::Default,
    linux_container::configuration::SeccompProfile::Profile(_) => ffi::SeccompMode::Profile,
  }
}

/// Swift asks for this only when the mode is `Profile`.
pub(super) fn custom_seccomp(profile: &linux_container::configuration::SeccompProfile) -> &runtime::LinuxSeccomp {
  match profile {
    linux_container::configuration::SeccompProfile::Profile(profile) => profile,
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
      seccomp_profile: linux_container::configuration::SeccompProfile::Profile(profile.clone()),
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
    let socket = container::UnixSocketConfiguration::new("/host.sock", "/guest.sock").expect("a socket configuration");
    configuration.clear_sockets();
    configuration.push_socket(socket.handle.duplicate());

    assert_eq!(configuration.sockets_len(), 1);
    assert_eq!(
      configuration.sockets_at(0).id(),
      socket.id(),
      "a socket keeps its id both ways"
    );
    assert_eq!(configuration.sockets, [socket]);
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
