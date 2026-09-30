//! Crate-private getters Swift reads the model through.
//!
//! Each lends what the model holds, so Swift's copy is the only one. Shapes
//! swift-bridge can't return become sets: optional nested value → `has_x` +
//! `x`; list → `x_len` + `x_at`; map → length + key and value at an index;
//! enum → mode + payload. Swift asks for an index only below the length.

use super::ffi::{RuntimeKind, SeccompMode, SocketDirection};
use crate::model;
use std::collections::BTreeMap;
use std::path::Path;

fn path(path: &Path) -> String {
  path.display().to_string()
}

/// The `index`th entry of a sorted map.
fn entry_at(map: &BTreeMap<String, String>, index: usize) -> (&str, &str) {
  map
    .iter()
    .nth(index)
    .map(|(key, value)| (key.as_str(), value.as_str()))
    .expect("Swift asks for an entry only below the length")
}

impl model::Mount {
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

  pub(crate) fn runtime_kind(&self) -> RuntimeKind {
    match self.runtime_options {
      model::RuntimeOptions::Virtioblk(_) => RuntimeKind::Virtioblk,
      model::RuntimeOptions::Virtiofs(_) => RuntimeKind::Virtiofs,
      model::RuntimeOptions::Any(_) => RuntimeKind::Generic,
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
      model::RuntimeOptions::Virtioblk(options)
      | model::RuntimeOptions::Virtiofs(options)
      | model::RuntimeOptions::Any(options) => options,
    }
  }
}

impl model::UnixSocketConfiguration {
  pub(crate) fn source(&self) -> String {
    path(&self.source)
  }

  pub(crate) fn destination(&self) -> String {
    path(&self.destination)
  }

  pub(crate) fn permissions(&self) -> Option<u32> {
    self.permissions
  }

  pub(crate) fn direction(&self) -> SocketDirection {
    match self.direction {
      model::Direction::Into => SocketDirection::Into,
      model::Direction::OutOf => SocketDirection::OutOf,
    }
  }
}

impl model::NatInterface {
  pub(crate) fn ipv4_address(&self) -> &str {
    &self.ipv4_address
  }

  pub(crate) fn ipv4_gateway(&self) -> Option<&str> {
    self.ipv4_gateway.as_deref()
  }

  pub(crate) fn ipv6_address(&self) -> Option<&str> {
    self.ipv6_address.as_deref()
  }

  pub(crate) fn ipv6_gateway(&self) -> Option<&str> {
    self.ipv6_gateway.as_deref()
  }

  pub(crate) fn mac_address(&self) -> Option<&str> {
    self.mac_address.as_deref()
  }

  pub(crate) fn mtu(&self) -> u32 {
    self.mtu
  }
}

impl model::Dns {
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

impl model::HostsEntry {
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

impl model::Hosts {
  pub(crate) fn entries_len(&self) -> usize {
    self.entries.len()
  }

  pub(crate) fn entries_at(&self, index: usize) -> &model::HostsEntry {
    &self.entries[index]
  }

  pub(crate) fn comment(&self) -> Option<&str> {
    self.comment.as_deref()
  }
}

impl model::BootLog {
  pub(crate) fn path(&self) -> String {
    path(&self.path)
  }

  pub(crate) fn append(&self) -> bool {
    self.append
  }
}

impl model::User {
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

impl model::LinuxProcessConfiguration {
  pub(crate) fn has_arguments(&self) -> bool {
    self.arguments.is_some()
  }

  /// Zero when [`Self::has_arguments`] is false.
  pub(crate) fn arguments_len(&self) -> usize {
    self.arguments.as_deref().unwrap_or_default().len()
  }

  pub(crate) fn arguments_at(&self, index: usize) -> &str {
    &self.arguments.as_deref().unwrap_or_default()[index]
  }

  pub(crate) fn environment_variables_len(&self) -> usize {
    self.environment_variables.len()
  }

  pub(crate) fn environment_variables_at(&self, index: usize) -> &str {
    &self.environment_variables[index]
  }

  pub(crate) fn working_directory(&self) -> Option<&str> {
    self.working_directory.as_deref()
  }

  pub(crate) fn has_user(&self) -> bool {
    self.user.is_some()
  }

  /// Only when [`Self::has_user`].
  pub(crate) fn user(&self) -> &model::User {
    self
      .user
      .as_ref()
      .expect("Swift asks for a user only after has_user")
  }
}

impl model::LinuxContainerConfiguration {
  pub(crate) fn process(&self) -> &model::LinuxProcessConfiguration {
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

  pub(crate) fn interfaces_at(&self, index: usize) -> &model::NatInterface {
    &self.interfaces[index]
  }

  pub(crate) fn sockets_len(&self) -> usize {
    self.sockets.len()
  }

  pub(crate) fn sockets_at(&self, index: usize) -> &model::UnixSocketConfiguration {
    &self.sockets[index]
  }

  pub(crate) fn mounts_len(&self) -> usize {
    self.mounts.len()
  }

  pub(crate) fn mounts_at(&self, index: usize) -> &model::Mount {
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
  pub(crate) fn dns(&self) -> &model::Dns {
    self
      .dns
      .as_ref()
      .expect("Swift asks for DNS only after has_dns")
  }

  pub(crate) fn has_hosts(&self) -> bool {
    self.hosts.is_some()
  }

  /// Only when [`Self::has_hosts`].
  pub(crate) fn hosts(&self) -> &model::Hosts {
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
  pub(crate) fn boot_log(&self) -> &model::BootLog {
    self
      .boot_log
      .as_ref()
      .expect("Swift asks for a boot log only after has_boot_log")
  }

  pub(crate) fn oci_runtime_path(&self) -> Option<&str> {
    self.oci_runtime_path.as_deref()
  }

  pub(crate) fn seccomp_mode(&self) -> SeccompMode {
    match self.seccomp_profile {
      model::SeccompProfile::Unconfined => SeccompMode::Unconfined,
      model::SeccompProfile::Default => SeccompMode::Default,
      model::SeccompProfile::Profile(_) => SeccompMode::Profile,
    }
  }

  /// The custom profile's JSON, when [`Self::seccomp_mode`] is `Profile`.
  pub(crate) fn seccomp_profile(&self) -> Option<&str> {
    match &self.seccomp_profile {
      model::SeccompProfile::Profile(profile) => Some(profile),
      _ => None,
    }
  }

  pub(crate) fn use_init(&self) -> bool {
    self.use_init
  }
}

impl model::BootSpec {
  pub(crate) fn id(&self) -> &str {
    &self.id
  }

  pub(crate) fn reference(&self) -> &str {
    &self.reference
  }

  pub(crate) fn rootfs_size_in_bytes(&self) -> u64 {
    self.rootfs_size_in_bytes
  }

  pub(crate) fn vm_cpus(&self) -> u32 {
    self.vm.cpus
  }

  pub(crate) fn vm_memory_in_bytes(&self) -> u64 {
    self.vm.memory_in_bytes
  }

  pub(crate) fn configuration(&self) -> &model::LinuxContainerConfiguration {
    &self.configuration
  }
}

impl model::BuildStep {
  pub(crate) fn name(&self) -> &str {
    &self.name
  }

  pub(crate) fn script(&self) -> &str {
    &self.script
  }

  pub(crate) fn user(&self) -> Option<&str> {
    self.user.as_deref()
  }

  /// Unsalted; the builder mixes in the base's digest and the rootfs ceiling.
  pub(crate) fn cache_key(&self) -> &str {
    &self.cache_key
  }
}

impl model::BuildPlan {
  pub(crate) fn name(&self) -> &str {
    &self.name
  }

  pub(crate) fn base(&self) -> &str {
    &self.base
  }

  pub(crate) fn tag(&self) -> &str {
    &self.tag
  }

  pub(crate) fn cpus(&self) -> u32 {
    self.cpus
  }

  pub(crate) fn memory_in_bytes(&self) -> u64 {
    self.memory_in_bytes
  }

  pub(crate) fn vm_cpus(&self) -> u32 {
    self.vm.cpus
  }

  pub(crate) fn vm_memory_in_bytes(&self) -> u64 {
    self.vm.memory_in_bytes
  }

  pub(crate) fn mounts_len(&self) -> usize {
    self.mounts.len()
  }

  pub(crate) fn mounts_at(&self, index: usize) -> &model::Mount {
    &self.mounts[index]
  }

  pub(crate) fn steps_len(&self) -> usize {
    self.steps.len()
  }

  pub(crate) fn steps_at(&self, index: usize) -> &model::BuildStep {
    &self.steps[index]
  }

  pub(crate) fn environment_len(&self) -> usize {
    self.environment.len()
  }

  pub(crate) fn environment_at(&self, index: usize) -> &str {
    &self.environment[index]
  }

  pub(crate) fn labels_len(&self) -> usize {
    self.labels.len()
  }

  pub(crate) fn label_key_at(&self, index: usize) -> &str {
    entry_at(&self.labels, index).0
  }

  pub(crate) fn label_value_at(&self, index: usize) -> &str {
    entry_at(&self.labels, index).1
  }

  pub(crate) fn user(&self) -> Option<&str> {
    self.user.as_deref()
  }

  pub(crate) fn working_directory(&self) -> Option<String> {
    self.workdir.as_deref().map(path)
  }

  pub(crate) fn interface(&self) -> &model::NatInterface {
    &self.interface
  }

  pub(crate) fn base_key(&self) -> &str {
    &self.base_key
  }

  pub(crate) fn rootfs_size_in_bytes(&self) -> u64 {
    self.rootfs_size_in_bytes
  }

  pub(crate) fn cache_restore(&self) -> bool {
    self.cache.restore
  }

  pub(crate) fn cache_keep(&self) -> u64 {
    self.cache.keep as u64
  }

  pub(crate) fn cache_keep_for_seconds(&self) -> u64 {
    self.cache.keep_for.as_secs()
  }

  pub(crate) fn shell_len(&self) -> usize {
    self.shell.0.len()
  }

  pub(crate) fn shell_at(&self, index: usize) -> &str {
    &self.shell.0[index]
  }

  pub(crate) fn keepalive_len(&self) -> usize {
    self.keepalive.len()
  }

  pub(crate) fn keepalive_at(&self, index: usize) -> &str {
    &self.keepalive[index]
  }

  pub(crate) fn reclaim(&self) -> bool {
    self.reclaim
  }
}

#[cfg(test)]
mod tests {
  use crate::model;

  #[test]
  fn reads_an_absent_option_as_absent() {
    let configuration = model::LinuxContainerConfiguration::default();

    assert!(!configuration.has_dns());
    assert!(!configuration.has_hosts());
    assert!(!configuration.has_boot_log());
    assert!(!configuration.process().has_arguments());
    assert_eq!(configuration.process().arguments_len(), 0);
    assert!(!configuration.process().has_user());
    assert_eq!(configuration.seccomp_profile(), None);
  }

  #[test]
  fn reads_a_list_as_its_length_and_each_element() {
    let process = model::LinuxProcessConfiguration::new(&["/bin/echo", "hello"]);

    assert_eq!(process.arguments_len(), 2);
    assert_eq!(process.arguments_at(1), "hello");

    let mount = model::Mount {
      runtime_options: model::RuntimeOptions::Virtiofs(vec!["cache=auto".into()]),
      ..model::Mount::share("/Users/user/workspace", "/workspace", &["ro"])
    };

    assert_eq!(mount.options_len(), 1);
    assert_eq!(mount.options_at(0), "ro");
    assert_eq!(mount.runtime_options_len(), 1);
    assert_eq!(mount.runtime_options_at(0), "cache=auto");
  }

  #[test]
  fn reads_a_map_as_its_length_and_each_entry() {
    let configuration = model::LinuxContainerConfiguration {
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
  fn reads_a_plan_in_the_units_swift_takes() {
    let mut plan = model::BuildPlan::new(
      "builder",
      "docker.io/library/debian:stable-slim",
      "example/base:latest",
      model::NatInterface::new("192.168.64.7/24", "192.168.64.1"),
      "basekey",
    );
    plan.workdir = Some("/workspace".into());
    plan.labels = [("com.example.built-by".to_string(), "example".to_string())].into();

    assert_eq!(plan.cache_keep_for_seconds(), 14 * 24 * 60 * 60);
    assert_eq!(plan.working_directory().as_deref(), Some("/workspace"));
    assert_eq!(plan.labels_len(), 1);
    assert_eq!(plan.label_key_at(0), "com.example.built-by");
    assert_eq!(plan.label_value_at(0), "example");
    assert_eq!(plan.interface().ipv4_gateway(), Some("192.168.64.1"));
  }

  #[test]
  fn reads_an_enum_as_its_mode_and_what_it_carries() {
    let configuration = model::LinuxContainerConfiguration {
      seccomp_profile: model::SeccompProfile::Profile("{}".into()),
      ..Default::default()
    };

    assert!(matches!(configuration.seccomp_mode(), super::SeccompMode::Profile));
    assert_eq!(configuration.seccomp_profile(), Some("{}"));

    let mount = model::Mount::block("ext4", "/images/data.ext4", "/data", &[]);
    assert!(matches!(mount.runtime_kind(), super::RuntimeKind::Virtioblk));
    assert_eq!(mount.mount_type(), "ext4");
  }
}
