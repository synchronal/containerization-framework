use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `Platform`. Swift reads one through `Platform(arch:os:osVersion:osFeatures:variant:)`,
/// and so normalizes its `architecture` as that does.
///
/// `==` is Swift's, which treats an `arm64` platform with no variant as
/// `arm64/v8` and ignores `os_version` and `os_features`. Off macOS, where
/// Swift can't be asked, comparing two platforms panics.
#[derive(Clone, Debug)]
pub struct Platform {
  pub architecture: String,
  pub os: String,
  pub os_version: Option<String>,
  pub os_features: Option<Vec<String>>,
  pub variant: Option<String>,
}

impl PartialEq for Platform {
  fn eq(&self, other: &Self) -> bool {
    platform::outcome(
      ffi::cz_platform_equals(self.clone(), other.clone()),
      "compare two platforms",
    )
    .map(|outcome| outcome.boolean())
    .unwrap_or_else(|error| panic!("{error}"))
  }
}

/// Swift's `==` is an equivalence.
impl Eq for Platform {}

impl Platform {
  /// `Platform.current`.
  pub fn current() -> Result<Self, Error> {
    platform::outcome(ffi::cz_platform_current(), "read the current platform").map(|outcome| outcome.platform())
  }

  /// `Platform(from platform: String)`, such as `linux/arm64`.
  pub fn parse(platform: &str) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_platform_parse(platform),
      format!("parse {platform:?} as a platform"),
    )
    .map(|outcome| outcome.platform())
  }

  /// `Platform.description`, such as `linux/arm/v7`.
  pub fn description(&self) -> Result<String, Error> {
    platform::outcome(ffi::cz_platform_description(self.clone()), "describe a platform").map(|outcome| outcome.text())
  }

  /// `~=`: whether an image for this platform runs on `other`.
  pub fn matches(&self, other: &Platform) -> Result<bool, Error> {
    platform::outcome(
      ffi::cz_platform_matches(self.clone(), other.clone()),
      "check whether one platform runs on another",
    )
    .map(|outcome| outcome.boolean())
  }
}

/// A `Platform?`, as it crosses to Swift: whether there is one, and it, or an
/// empty one that Swift doesn't read. (swift-bridge can't pass an `Option` of
/// a Rust type.)
pub(crate) fn crossing(platform: Option<&Platform>) -> (bool, Platform) {
  match platform {
    Some(platform) => (true, platform.clone()),
    None => (
      false,
      Platform {
        architecture: String::new(),
        os: String::new(),
        os_version: None,
        os_features: None,
        variant: None,
      },
    ),
  }
}
