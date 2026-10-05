//! `SystemPlatform`, and its nested `SystemPlatform.OS` and
//! `SystemPlatform.Architecture`.

use crate::containerization_oci;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `SystemPlatform.OS`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Os {
  Linux,
  Darwin,
}

/// `SystemPlatform.Architecture`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Architecture {
  Arm64,
  Amd64,
}

/// `SystemPlatform`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemPlatform {
  pub os: Os,
  pub architecture: Architecture,
}

impl SystemPlatform {
  /// `SystemPlatform.linuxArm`.
  pub const LINUX_ARM: Self = Self {
    os: Os::Linux,
    architecture: Architecture::Arm64,
  };

  /// `SystemPlatform.linuxAmd`.
  pub const LINUX_AMD: Self = Self {
    os: Os::Linux,
    architecture: Architecture::Amd64,
  };

  /// `SystemPlatform.ociPlatform()`.
  pub fn oci_platform(&self) -> Result<containerization_oci::Platform, Error> {
    platform::outcome(
      ffi::cz_system_platform_oci_platform(*self),
      "convert a system platform to OCI's",
    )
    .map(|outcome| outcome.platform())
  }
}
