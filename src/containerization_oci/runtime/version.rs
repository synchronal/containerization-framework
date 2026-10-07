use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `RuntimeSpecVersion`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RuntimeSpecVersion {
  pub major: isize,
  pub minor: isize,
  pub patch: isize,
  pub dev: String,
}

impl RuntimeSpecVersion {
  /// `RuntimeSpecVersion(major:minor:patch:dev:)`.
  pub fn new(major: isize, minor: isize, patch: isize, dev: impl Into<String>) -> Self {
    Self {
      major,
      minor,
      patch,
      dev: dev.into(),
    }
  }

  /// `RuntimeSpecVersion.current`.
  pub fn current() -> Result<Self, Error> {
    platform::outcome(ffi::cz_runtime_spec_version_current(), "read the runtime spec version")
      .map(|outcome| outcome.runtime_spec_version())
  }
}
