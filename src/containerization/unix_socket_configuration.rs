//! `UnixSocketConfiguration`, and its nested `UnixSocketConfiguration.Direction`.

use std::path::PathBuf;

/// `UnixSocketConfiguration.Direction`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Direction {
  #[default]
  Into,
  OutOf,
}

/// `UnixSocketConfiguration`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnixSocketConfiguration {
  pub source: PathBuf,
  pub destination: PathBuf,
  pub permissions: Option<u32>,
  pub direction: Direction,
}

impl UnixSocketConfiguration {
  /// `UnixSocketConfiguration(source:destination:)`, its other arguments at
  /// their defaults.
  pub fn new(source: impl Into<PathBuf>, destination: impl Into<PathBuf>) -> Self {
    Self {
      source: source.into(),
      destination: destination.into(),
      permissions: None,
      direction: Direction::Into,
    }
  }
}
