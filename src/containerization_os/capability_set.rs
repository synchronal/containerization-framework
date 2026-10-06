use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::fmt;

/// `CapabilitySet`. Each case is one of Swift's static properties.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CapabilitySet {
  Bounding,
  Effective,
  Inheritable,
  Permitted,
  Ambient,
}

impl CapabilitySet {
  /// Every case, in the order the bridge lists Swift's descriptions.
  pub(crate) const ALL: &[Self] = &[
    Self::Bounding,
    Self::Effective,
    Self::Inheritable,
    Self::Permitted,
    Self::Ambient,
  ];

  /// `CapabilitySet(rawValue:)`, which ignores case.
  pub fn parse(raw_value: &str) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_capability_set_parse(raw_value),
      format!("parse the capability set {raw_value}"),
    )
    .map(|outcome| Self::from_description(&outcome.text()))
  }

  /// `description`, such as `bounding`.
  pub fn description(self) -> &'static str {
    match self {
      Self::Bounding => "bounding",
      Self::Effective => "effective",
      Self::Inheritable => "inheritable",
      Self::Permitted => "permitted",
      Self::Ambient => "ambient",
    }
  }

  /// The case Swift described. It panics if Swift has a case Rust lacks.
  fn from_description(description: &str) -> Self {
    Self::ALL
      .iter()
      .copied()
      .find(|case| case.description() == description)
      .unwrap_or_else(|| panic!("Swift's CapabilitySet has a case Rust lacks: {description:?}"))
  }
}

impl fmt::Display for CapabilitySet {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(self.description())
  }
}
