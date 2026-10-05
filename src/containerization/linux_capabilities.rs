use super::strings;
use crate::containerization_oci;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `LinuxCapabilities`. Each set holds `CapabilityName`s by name, e.g.
/// `CAP_CHOWN`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LinuxCapabilities {
  pub bounding: Vec<String>,
  pub effective: Vec<String>,
  pub inheritable: Vec<String>,
  pub permitted: Vec<String>,
  pub ambient: Vec<String>,
}

impl LinuxCapabilities {
  /// `LinuxCapabilities.defaultOCICapabilities`.
  pub fn default_oci_capabilities() -> Self {
    let defaults = strings(&[
      "CAP_CHOWN",
      "CAP_DAC_OVERRIDE",
      "CAP_FSETID",
      "CAP_FOWNER",
      "CAP_MKNOD",
      "CAP_NET_RAW",
      "CAP_SETGID",
      "CAP_SETUID",
      "CAP_SETFCAP",
      "CAP_SETPCAP",
      "CAP_NET_BIND_SERVICE",
      "CAP_SYS_CHROOT",
      "CAP_KILL",
      "CAP_AUDIT_WRITE",
    ]);

    Self {
      bounding: defaults.clone(),
      effective: defaults.clone(),
      inheritable: Vec::new(),
      permitted: defaults,
      ambient: Vec::new(),
    }
  }

  /// `LinuxCapabilities.toOCI()`, which leaves out empty sets. It fails on a
  /// name Swift doesn't know.
  pub fn to_oci(&self) -> Result<containerization_oci::LinuxCapabilities, Error> {
    platform::outcome(
      ffi::cz_linux_capabilities_to_oci(self.clone()),
      "convert capabilities to OCI's",
    )
    .map(|outcome| outcome.oci_linux_capabilities())
  }
}
