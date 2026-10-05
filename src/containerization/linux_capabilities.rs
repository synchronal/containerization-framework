use super::strings;

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
}
