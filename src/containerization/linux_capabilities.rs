use crate::containerization_oci;
use crate::containerization_os::CapabilityName;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `LinuxCapabilities`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LinuxCapabilities {
  pub bounding: Vec<CapabilityName>,
  pub effective: Vec<CapabilityName>,
  pub inheritable: Vec<CapabilityName>,
  pub permitted: Vec<CapabilityName>,
  pub ambient: Vec<CapabilityName>,
}

impl LinuxCapabilities {
  /// `LinuxCapabilities.allCapabilities`, which holds every capability in
  /// every set.
  pub fn all_capabilities() -> Self {
    let all = CapabilityName::ALL_CASES.to_vec();

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
    Self::with(vec![
      CapabilityName::Chown,
      CapabilityName::DacOverride,
      CapabilityName::Fsetid,
      CapabilityName::Fowner,
      CapabilityName::Mknod,
      CapabilityName::NetRaw,
      CapabilityName::Setgid,
      CapabilityName::Setuid,
      CapabilityName::Setfcap,
      CapabilityName::Setpcap,
      CapabilityName::NetBindService,
      CapabilityName::SysChroot,
      CapabilityName::Kill,
      CapabilityName::AuditWrite,
    ])
  }

  /// `LinuxCapabilities(capabilities:)`, which puts the capabilities in the
  /// bounding, effective and permitted sets, and leaves the others empty.
  pub fn with(capabilities: Vec<CapabilityName>) -> Self {
    Self {
      bounding: capabilities.clone(),
      effective: capabilities.clone(),
      inheritable: Vec::new(),
      permitted: capabilities,
      ambient: Vec::new(),
    }
  }

  /// `LinuxCapabilities.toOCI()`, which leaves out empty sets.
  pub fn to_oci(&self) -> Result<containerization_oci::LinuxCapabilities, Error> {
    platform::outcome(
      ffi::cz_linux_capabilities_to_oci(self.clone()),
      "convert capabilities to OCI's",
    )
    .map(|outcome| outcome.oci_linux_capabilities())
  }
}
