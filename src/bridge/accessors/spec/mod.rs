//! The OCI runtime spec's types, both ways: Rust reads Swift's field by field
//! from an outcome, and Swift reads Rust's through these getters.
//!
//! A small type that only one other holds, like `Box` in `Process` or
//! `LinuxSeccompArg` in `LinuxSyscall`, is read through its holder's getters.
//! An enum crosses as its `rawValue`.

mod hooks;
mod linux;
mod mount;
mod process;
mod resources;
mod seccomp;

use crate::bridge::accessors::present;
use crate::bridge::ffi;
use crate::containerization_oci::runtime;

impl ffi::CzOutcome {
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

impl runtime::Spec {
  getters! {
    str version;
    present hooks: runtime::Hooks => has_hooks;
    present process: runtime::Process => has_process;
    str hostname;
    str domainname;
    list mounts: runtime::Mount => mounts_len, mounts_at;
    optional_map annotations => has_annotations, annotations_len, annotation_key_at, annotation_value_at;
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

  getters! {
    present linux: runtime::Linux => has_linux;
  }
}

#[cfg(test)]
mod tests {
  use crate::bridge::accessors::raw_values;
  use crate::bridge::ffi;
  use crate::containerization_oci::runtime;

  /// Checks each enum's raw values, in Rust's order, against Swift's.
  #[test]
  fn copies_swifts_raw_values() {
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

  /// Checks each `Default` against Swift's initializer with no arguments.
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
