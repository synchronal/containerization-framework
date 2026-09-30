//! Crate-private getters Swift reads the model through.
//!
//! Shapes swift-bridge can't return become pairs: optional nested value →
//! `has_x` + `x`; map → keys + lookup; enum → mode + payload.

use super::ffi::{RuntimeKind, SeccompMode, SocketDirection};
use crate::model;
use std::path::Path;

fn path(path: &Path) -> String {
  path.display().to_string()
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

  pub(crate) fn options(&self) -> Vec<String> {
    self.options.clone()
  }

  pub(crate) fn runtime_kind(&self) -> RuntimeKind {
    match self.runtime_options {
      model::RuntimeOptions::Virtioblk(_) => RuntimeKind::Virtioblk,
      model::RuntimeOptions::Virtiofs(_) => RuntimeKind::Virtiofs,
      model::RuntimeOptions::Any(_) => RuntimeKind::Generic,
    }
  }

  pub(crate) fn runtime_options(&self) -> Vec<String> {
    match &self.runtime_options {
      model::RuntimeOptions::Virtioblk(options)
      | model::RuntimeOptions::Virtiofs(options)
      | model::RuntimeOptions::Any(options) => options.clone(),
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

  pub(crate) fn ipv4_gateway(&self) -> Option<String> {
    self.ipv4_gateway.clone()
  }

  pub(crate) fn ipv6_address(&self) -> Option<String> {
    self.ipv6_address.clone()
  }

  pub(crate) fn ipv6_gateway(&self) -> Option<String> {
    self.ipv6_gateway.clone()
  }

  pub(crate) fn mac_address(&self) -> Option<String> {
    self.mac_address.clone()
  }

  pub(crate) fn mtu(&self) -> u32 {
    self.mtu
  }
}

impl model::Dns {
  pub(crate) fn nameservers(&self) -> Vec<String> {
    self.nameservers.clone()
  }

  pub(crate) fn domain(&self) -> Option<String> {
    self.domain.clone()
  }

  pub(crate) fn search_domains(&self) -> Vec<String> {
    self.search_domains.clone()
  }

  pub(crate) fn options(&self) -> Vec<String> {
    self.options.clone()
  }
}

impl model::HostsEntry {
  pub(crate) fn ip_address(&self) -> &str {
    &self.ip_address
  }

  pub(crate) fn hostnames(&self) -> Vec<String> {
    self.hostnames.clone()
  }

  pub(crate) fn comment(&self) -> Option<String> {
    self.comment.clone()
  }
}

impl model::Hosts {
  pub(crate) fn entries(&self) -> Vec<model::HostsEntry> {
    self.entries.clone()
  }

  pub(crate) fn comment(&self) -> Option<String> {
    self.comment.clone()
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

  pub(crate) fn additional_gids(&self) -> Vec<u32> {
    self.additional_gids.clone()
  }

  pub(crate) fn username(&self) -> &str {
    &self.username
  }
}

impl model::LinuxProcessConfiguration {
  pub(crate) fn has_arguments(&self) -> bool {
    self.arguments.is_some()
  }

  /// Empty when [`Self::has_arguments`] is false.
  pub(crate) fn arguments(&self) -> Vec<String> {
    self.arguments.clone().unwrap_or_default()
  }

  pub(crate) fn environment_variables(&self) -> Vec<String> {
    self.environment_variables.clone()
  }

  pub(crate) fn working_directory(&self) -> Option<String> {
    self.working_directory.clone()
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

  pub(crate) fn hostname(&self) -> Option<String> {
    self.hostname.clone()
  }

  pub(crate) fn sysctl_keys(&self) -> Vec<String> {
    self.sysctl.keys().cloned().collect()
  }

  pub(crate) fn sysctl(&self, key: &str) -> Option<String> {
    self.sysctl.get(key).cloned()
  }

  pub(crate) fn interfaces(&self) -> Vec<model::NatInterface> {
    self.interfaces.clone()
  }

  pub(crate) fn sockets(&self) -> Vec<model::UnixSocketConfiguration> {
    self.sockets.clone()
  }

  pub(crate) fn mounts(&self) -> Vec<model::Mount> {
    self.mounts.clone()
  }

  pub(crate) fn masked_paths(&self) -> Vec<String> {
    self.masked_paths.clone()
  }

  pub(crate) fn readonly_paths(&self) -> Vec<String> {
    self.readonly_paths.clone()
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

  pub(crate) fn oci_runtime_path(&self) -> Option<String> {
    self.oci_runtime_path.clone()
  }

  pub(crate) fn seccomp_mode(&self) -> SeccompMode {
    match self.seccomp_profile {
      model::SeccompProfile::Unconfined => SeccompMode::Unconfined,
      model::SeccompProfile::Default => SeccompMode::Default,
      model::SeccompProfile::Profile(_) => SeccompMode::Profile,
    }
  }

  /// The custom profile's JSON, when [`Self::seccomp_mode`] is `Profile`.
  pub(crate) fn seccomp_profile(&self) -> Option<String> {
    match &self.seccomp_profile {
      model::SeccompProfile::Profile(profile) => Some(profile.clone()),
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

  pub(crate) fn user(&self) -> Option<String> {
    self.user.clone()
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

  pub(crate) fn mounts(&self) -> Vec<model::Mount> {
    self.mounts.clone()
  }

  pub(crate) fn steps(&self) -> Vec<model::BuildStep> {
    self.steps.clone()
  }

  pub(crate) fn environment(&self) -> Vec<String> {
    self.environment.clone()
  }

  pub(crate) fn label_keys(&self) -> Vec<String> {
    self.labels.keys().cloned().collect()
  }

  pub(crate) fn label(&self, key: &str) -> Option<String> {
    self.labels.get(key).cloned()
  }

  pub(crate) fn user(&self) -> Option<String> {
    self.user.clone()
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

  pub(crate) fn shell(&self) -> Vec<String> {
    self.shell.0.clone()
  }

  pub(crate) fn keepalive(&self) -> Vec<String> {
    self.keepalive.clone()
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
    assert!(configuration.process().arguments().is_empty());
    assert!(!configuration.process().has_user());
    assert_eq!(configuration.seccomp_profile(), None);
  }

  #[test]
  fn reads_a_map_as_its_keys_and_a_lookup() {
    let configuration = model::LinuxContainerConfiguration {
      sysctl: [("net.core.somaxconn".to_string(), "4096".to_string())].into(),
      ..Default::default()
    };

    assert_eq!(configuration.sysctl_keys(), ["net.core.somaxconn"]);
    assert_eq!(configuration.sysctl("net.core.somaxconn").as_deref(), Some("4096"));
    assert_eq!(configuration.sysctl("vm.swappiness"), None);
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
    assert_eq!(plan.label_keys(), ["com.example.built-by"]);
    assert_eq!(plan.label("com.example.built-by").as_deref(), Some("example"));
    assert_eq!(plan.interface().ipv4_gateway().as_deref(), Some("192.168.64.1"));
  }

  #[test]
  fn reads_an_enum_as_its_mode_and_what_it_carries() {
    let configuration = model::LinuxContainerConfiguration {
      seccomp_profile: model::SeccompProfile::Profile("{}".into()),
      ..Default::default()
    };

    assert!(matches!(configuration.seccomp_mode(), super::SeccompMode::Profile));
    assert_eq!(configuration.seccomp_profile().as_deref(), Some("{}"));

    let mount = model::Mount::block("ext4", "/images/data.ext4", "/data", &[]);
    assert!(matches!(mount.runtime_kind(), super::RuntimeKind::Virtioblk));
    assert_eq!(mount.mount_type(), "ext4");
  }
}
