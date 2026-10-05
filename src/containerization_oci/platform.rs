use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `Platform`. Swift reads one through `Platform(arch:os:osVersion:osFeatures:variant:)`,
/// and so normalizes its `architecture` as that does.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Platform {
  pub architecture: String,
  pub os: String,
  pub os_version: Option<String>,
  pub os_features: Option<Vec<String>>,
  pub variant: Option<String>,
}

impl Platform {
  /// `Platform.current`.
  pub fn current() -> Result<Self, Error> {
    platform::outcome(ffi::cz_platform_current(), "read the current platform").map(|outcome| outcome.platform())
  }
}
