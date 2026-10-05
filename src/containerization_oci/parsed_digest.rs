use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::path::Path;
use std::path::PathBuf;

/// `ParsedDigest`. Only Swift makes one, so its `encoded` is valid.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ParsedDigest {
  pub(crate) encoded: String,
}

impl ParsedDigest {
  /// `ParsedDigest.algorithm`.
  pub const ALGORITHM: &str = "sha256";

  /// `ParsedDigest(parsing:)`, which needs the `sha256:` prefix.
  pub fn parse(digest: &str) -> Result<Self, Error> {
    Self::parsed(
      ffi::cz_parsed_digest_parse(digest),
      format!("parse {digest:?} as a digest"),
    )
  }

  /// `ParsedDigest(parsingPathComponent:)`, which also accepts the digits
  /// without it.
  pub fn parse_path_component(component: &str) -> Result<Self, Error> {
    Self::parsed(
      ffi::cz_parsed_digest_parse_path_component(component),
      format!("parse {component:?} as a digest"),
    )
  }

  fn parsed(outcome: ffi::CzOutcome, action: String) -> Result<Self, Error> {
    platform::outcome(outcome, action).map(|outcome| Self {
      encoded: outcome.parsed_digest_encoded(),
    })
  }

  /// `ParsedDigest.isValid(_:)`.
  pub fn is_valid(digest: &str) -> Result<bool, Error> {
    platform::outcome(
      ffi::cz_parsed_digest_is_valid(digest),
      format!("check whether {digest:?} is a digest"),
    )
    .map(|outcome| outcome.boolean())
  }

  /// `ParsedDigest.encoded`.
  pub fn encoded(&self) -> &str {
    &self.encoded
  }

  /// `ParsedDigest.description`.
  pub fn description(&self) -> Result<String, Error> {
    platform::outcome(ffi::cz_parsed_digest_description(&self.encoded), "describe a digest")
      .map(|outcome| outcome.text())
  }

  /// `ParsedDigest.path(in:)`.
  pub fn path(&self, in_root: &Path) -> Result<PathBuf, Error> {
    platform::outcome(
      ffi::cz_parsed_digest_path(&self.encoded, &in_root.display().to_string()),
      format!("find {} in {}", self.encoded, in_root.display()),
    )
    .map(|outcome| PathBuf::from(outcome.text()))
  }
}
