//! `SystemPlatform`, and its nested `SystemPlatform.OS` and
//! `SystemPlatform.Architecture`.

use crate::containerization_oci;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

macro_rules! raw_values {
  ($(#[$doc:meta])* $name:ident { $($case:ident = $raw_value:literal),* $(,)? }) => {
    $(#[$doc])*
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub enum $name {
      $($case),*
    }

    impl $name {
      /// `allCases`.
      pub const ALL_CASES: &[Self] = &[$(Self::$case),*];

      /// `init(rawValue:)`, which is `None` for a raw value Swift lacks.
      pub fn from_raw_value(raw_value: &str) -> Option<Self> {
        Self::ALL_CASES.iter().copied().find(|case| case.as_str() == raw_value)
      }

      /// `rawValue`.
      pub fn as_str(self) -> &'static str {
        match self {
          $(Self::$case => $raw_value),*
        }
      }
    }
  };
}

raw_values!(
  /// `SystemPlatform.OS`.
  Os { Linux = "linux", Darwin = "darwin" }
);

raw_values!(
  /// `SystemPlatform.Architecture`.
  Architecture { Arm64 = "arm64", Amd64 = "amd64" }
);

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reads_back_every_raw_value_it_writes() {
    for os in Os::ALL_CASES {
      assert_eq!(Os::from_raw_value(os.as_str()), Some(*os));
    }
    for architecture in Architecture::ALL_CASES {
      assert_eq!(Architecture::from_raw_value(architecture.as_str()), Some(*architecture));
    }
    assert_eq!(Os::from_raw_value("Linux"), None, "raw values are case sensitive");
  }
}
