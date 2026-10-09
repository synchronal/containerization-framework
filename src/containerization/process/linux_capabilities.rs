use crate::containerization_oci;
use crate::containerization_os;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `LinuxCapabilities`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LinuxCapabilities {
  pub bounding: Vec<containerization_os::linux::CapabilityName>,
  pub effective: Vec<containerization_os::linux::CapabilityName>,
  pub inheritable: Vec<containerization_os::linux::CapabilityName>,
  pub permitted: Vec<containerization_os::linux::CapabilityName>,
  pub ambient: Vec<containerization_os::linux::CapabilityName>,
}

impl LinuxCapabilities {
  /// `LinuxCapabilities.allCapabilities`, which holds every capability in
  /// every set.
  pub fn all_capabilities() -> Self {
    let all = containerization_os::linux::CapabilityName::ALL_CASES.to_vec();

    Self {
      bounding: all.clone(),
      effective: all.clone(),
      inheritable: all.clone(),
      permitted: all.clone(),
      ambient: all,
    }
  }

  /// `LinuxCapabilities.defaultOCICapabilities`.
  pub fn default_oci_capabilities() -> Self {
    Self::with_capabilities(vec![
      containerization_os::linux::CapabilityName::Chown,
      containerization_os::linux::CapabilityName::DacOverride,
      containerization_os::linux::CapabilityName::Fsetid,
      containerization_os::linux::CapabilityName::Fowner,
      containerization_os::linux::CapabilityName::Mknod,
      containerization_os::linux::CapabilityName::NetRaw,
      containerization_os::linux::CapabilityName::Setgid,
      containerization_os::linux::CapabilityName::Setuid,
      containerization_os::linux::CapabilityName::Setfcap,
      containerization_os::linux::CapabilityName::Setpcap,
      containerization_os::linux::CapabilityName::NetBindService,
      containerization_os::linux::CapabilityName::SysChroot,
      containerization_os::linux::CapabilityName::Kill,
      containerization_os::linux::CapabilityName::AuditWrite,
    ])
  }

  /// `LinuxCapabilities(capabilities:)`, which puts the capabilities in the
  /// bounding, effective and permitted sets, and leaves the others empty.
  /// Rust has no overloading, so the suffix names the argument label that
  /// tells it apart from `LinuxCapabilities()`.
  pub fn with_capabilities(capabilities: Vec<containerization_os::linux::CapabilityName>) -> Self {
    Self {
      bounding: capabilities.clone(),
      effective: capabilities.clone(),
      inheritable: Vec::new(),
      permitted: capabilities,
      ambient: Vec::new(),
    }
  }

  /// `LinuxCapabilities.toOCI()`, which leaves out empty sets.
  pub fn to_oci(&self) -> Result<containerization_oci::runtime::LinuxCapabilities, Error> {
    platform::outcome(
      ffi::cz_linux_capabilities_to_oci(self.clone()),
      "convert capabilities to OCI's",
    )
    .map(|outcome| outcome.oci_linux_capabilities())
  }
}
