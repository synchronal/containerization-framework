//! The OCI runtime spec's types, both ways: Rust reads Swift's field by field
//! from an outcome, and Swift reads Rust's through these getters.
//!
//! A small type that only one other holds, like `Box` in `Process` or
//! `LinuxSeccompArg` in `LinuxSyscall`, is read through its holder's getters.
//! An enum crosses as its `rawValue`.

mod hooks;
mod image;
mod linux;
mod mount;
mod process;
mod resources;
mod seccomp;

use super::entry_at;
use crate::bridge::ffi;
use crate::containerization_oci::runtime;
use std::collections::BTreeMap;

impl ffi::CzOutcome {
  /// What an outcome holds as text, or `None` for `Absent`.
  pub(crate) fn optional_text(&self) -> Option<String> {
    self.optional(Self::text)
  }

  /// A held `[String: T]`, each value read by `read`.
  pub(super) fn map_of<T>(&self, read: impl Fn(&Self) -> T) -> BTreeMap<String, T> {
    self
      .entry_keys()
      .into_iter()
      .zip(self.entry_values().list(read))
      .collect()
  }

  /// The `Spec` an outcome holds, read field by field.
  pub(crate) fn spec(&self) -> runtime::Spec {
    runtime::Spec {
      version: self.spec_version(),
      hooks: self.spec_hooks().optional(Self::hooks),
      process: self.spec_process().optional(Self::process),
      hostname: self.spec_hostname(),
      domainname: self.spec_domainname(),
      mounts: self.spec_mounts().list(Self::oci_mount),
      annotations: self.spec_annotations().optional(Self::map),
      root: self.spec_root().optional(Self::root),
      linux: self.spec_linux().optional(Self::linux),
    }
  }

  fn root(&self) -> runtime::Root {
    runtime::Root {
      path: self.root_path(),
      readonly: self.root_readonly(),
    }
  }

  pub(crate) fn runtime_spec_version(&self) -> runtime::RuntimeSpecVersion {
    runtime::RuntimeSpecVersion {
      major: self.runtime_spec_version_major(),
      minor: self.runtime_spec_version_minor(),
      patch: self.runtime_spec_version_patch(),
      dev: self.runtime_spec_version_dev(),
    }
  }
}

/// What an optional field holds. Swift asks for it only after `has_`.
fn present<T>(value: &Option<T>) -> &T {
  value
    .as_ref()
    .expect("Swift asks for an optional field only after its `has_` getter")
}

/// The `index`th entry of a sorted map of values other than strings.
fn nth<T>(map: &BTreeMap<String, T>, index: usize) -> (&str, &T) {
  map
    .iter()
    .nth(index)
    .map(|(key, value)| (key.as_str(), value))
    .expect("Swift asks for an entry only below the length")
}

impl runtime::Spec {
  pub(crate) fn version(&self) -> &str {
    &self.version
  }

  pub(crate) fn has_hooks(&self) -> bool {
    self.hooks.is_some()
  }

  pub(crate) fn hooks(&self) -> &runtime::Hooks {
    present(&self.hooks)
  }

  pub(crate) fn has_process(&self) -> bool {
    self.process.is_some()
  }

  pub(crate) fn process(&self) -> &runtime::Process {
    present(&self.process)
  }

  pub(crate) fn hostname(&self) -> &str {
    &self.hostname
  }

  pub(crate) fn domainname(&self) -> &str {
    &self.domainname
  }

  pub(crate) fn mounts_len(&self) -> usize {
    self.mounts.len()
  }

  pub(crate) fn mounts_at(&self, index: usize) -> &runtime::Mount {
    &self.mounts[index]
  }

  pub(crate) fn has_annotations(&self) -> bool {
    self.annotations.is_some()
  }

  pub(crate) fn annotations_len(&self) -> usize {
    self.annotations.as_ref().map_or(0, BTreeMap::len)
  }

  pub(crate) fn annotation_key_at(&self, index: usize) -> &str {
    entry_at(present(&self.annotations), index).0
  }

  pub(crate) fn annotation_value_at(&self, index: usize) -> &str {
    entry_at(present(&self.annotations), index).1
  }

  pub(crate) fn has_root(&self) -> bool {
    self.root.is_some()
  }

  pub(crate) fn root_path(&self) -> &str {
    &present(&self.root).path
  }

  pub(crate) fn root_readonly(&self) -> bool {
    present(&self.root).readonly
  }

  pub(crate) fn has_linux(&self) -> bool {
    self.linux.is_some()
  }

  pub(crate) fn linux(&self) -> &runtime::Linux {
    present(&self.linux)
  }
}

#[cfg(test)]
mod tests {
  use crate::bridge::ffi;
  use crate::containerization_oci::runtime;

  /// Each enum's raw values, in Rust's order, against Swift's.
  #[test]
  fn copies_swifts_raw_values() {
    fn raw_values<T: Copy>(all: &[T], raw_value: fn(T) -> &'static str) -> Vec<&'static str> {
      all.iter().map(|case| raw_value(*case)).collect()
    }

    let copies = [
      (
        "LinuxNamespaceType",
        raw_values(runtime::LinuxNamespaceType::ALL, runtime::LinuxNamespaceType::raw_value),
      ),
      (
        "LinuxPersonalityDomain",
        raw_values(
          runtime::LinuxPersonalityDomain::ALL,
          runtime::LinuxPersonalityDomain::raw_value,
        ),
      ),
      (
        "LinuxSeccompFlag",
        raw_values(runtime::LinuxSeccompFlag::ALL, runtime::LinuxSeccompFlag::raw_value),
      ),
      ("Arch", raw_values(runtime::Arch::ALL, runtime::Arch::raw_value)),
      (
        "LinuxSeccompAction",
        raw_values(runtime::LinuxSeccompAction::ALL, runtime::LinuxSeccompAction::raw_value),
      ),
      (
        "LinuxSeccompOperator",
        raw_values(
          runtime::LinuxSeccompOperator::ALL,
          runtime::LinuxSeccompOperator::raw_value,
        ),
      ),
      (
        "ContainerState",
        raw_values(runtime::ContainerState::ALL, runtime::ContainerState::raw_value),
      ),
    ];

    for (name, copy) in copies {
      assert_eq!(ffi::cz_raw_values(name), copy, "{name}");
    }
  }

  #[test]
  fn copies_swifts_seccomp_fd_name() {
    assert_eq!(ffi::cz_seccomp_fd_name(), runtime::SECCOMP_FD_NAME);
  }

  /// Each `Default` against Swift's initializer with no arguments.
  #[test]
  fn defaults_as_swift_does() {
    assert_eq!(ffi::cz_default("Spec").spec(), runtime::Spec::default());
    assert_eq!(ffi::cz_default("Process").process(), runtime::Process::default());
    assert_eq!(
      ffi::cz_default("LinuxCapabilities").oci_linux_capabilities(),
      runtime::LinuxCapabilities::default()
    );
    assert_eq!(ffi::cz_default("Linux").linux(), runtime::Linux::default());
    assert_eq!(
      ffi::cz_default("LinuxResources").resources(),
      runtime::LinuxResources::default()
    );
    assert_eq!(ffi::cz_default("LinuxMemory").memory(), runtime::LinuxMemory::default());
    assert_eq!(ffi::cz_default("LinuxCPU").cpu(), runtime::LinuxCPU::default());
  }
}
