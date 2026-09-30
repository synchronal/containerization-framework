//! Getters for an image to build.

use super::entry_at;
use crate::model;

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
    self.workdir.as_deref().map(super::path)
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
}
