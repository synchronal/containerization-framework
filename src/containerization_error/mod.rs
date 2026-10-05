//! Containerization's `ContainerizationError` module.
//!
//! A thrown `ContainerizationError` reaches Rust as an [`Error`](crate::Error)
//! carrying its [`Code`].

use std::fmt;

/// `ContainerizationError.Code`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Code {
  Unknown,
  InvalidArgument,
  InternalError,
  Exists,
  NotFound,
  Cancelled,
  InvalidState,
  Empty,
  Timeout,
  Unsupported,
  Interrupted,
}

impl Code {
  const ALL: [Self; 11] = [
    Self::Unknown,
    Self::InvalidArgument,
    Self::InternalError,
    Self::Exists,
    Self::NotFound,
    Self::Cancelled,
    Self::InvalidState,
    Self::Empty,
    Self::Timeout,
    Self::Unsupported,
    Self::Interrupted,
  ];

  /// The code whose `description` Swift gave.
  pub(crate) fn from_description(description: &str) -> Option<Self> {
    Self::ALL
      .into_iter()
      .find(|code| code.description() == description)
  }

  fn description(self) -> &'static str {
    match self {
      Self::Unknown => "unknown",
      Self::InvalidArgument => "invalidArgument",
      Self::InternalError => "internalError",
      Self::Exists => "exists",
      Self::NotFound => "notFound",
      Self::Cancelled => "cancelled",
      Self::InvalidState => "invalidState",
      Self::Empty => "empty",
      Self::Timeout => "timeout",
      Self::Unsupported => "unsupported",
      Self::Interrupted => "interrupted",
    }
  }
}

/// `description`, as Swift writes it.
impl fmt::Display for Code {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(self.description())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reads_back_every_description_it_writes() {
    for code in Code::ALL {
      assert_eq!(Code::from_description(&code.to_string()), Some(code));
    }
  }

  #[test]
  fn reads_no_code_from_an_unknown_description() {
    assert_eq!(Code::from_description("NotFound"), None);
  }
}
