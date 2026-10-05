//! `SystemPlatform`, and its nested `SystemPlatform.OS` and
//! `SystemPlatform.Architecture`.

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
}
