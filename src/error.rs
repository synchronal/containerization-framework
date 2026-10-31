//! The error that every fallible call in this crate returns.

use crate::containerization_error;
use crate::containerization_os::file_descriptor_ops;
use crate::containerization_os::volume_subpath;
use std::fmt;

/// An error thrown by Swift or by a Rust closure, a value that could not cross
/// between Rust and Swift, or a call made off macOS.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
  /// The call failed, or a value it passed or returned could not cross
  /// between Rust and Swift.
  Failed {
    /// What was attempted, such as `pull docker.io/library/alpine:3`.
    action: String,
    /// Why it failed, usually in Containerization's words. For a failure
    /// inside the VM, this is the only account there is.
    message: String,
    /// The code of the thrown `ContainerizationError`, if Swift threw one.
    code: Option<containerization_error::Code>,
  },
  /// The call threw a `FileDescriptorOps.Error`.
  FileDescriptorOps {
    /// What was attempted, such as `open the file config.json`.
    action: String,
    /// What Swift threw.
    error: file_descriptor_ops::Error,
  },
  /// The call threw a `VolumeSubpath.Error`.
  VolumeSubpath {
    /// What was attempted, such as `take the subpath from the mount options`.
    action: String,
    /// What Swift threw.
    error: volume_subpath::Error,
  },
  /// The call was not attempted, because Containerization runs only on
  /// macOS.
  Unavailable {
    /// What would have been attempted.
    action: String,
    /// Why it could not be attempted.
    message: String,
  },
}

impl Error {
  /// A failure of `action` with no `ContainerizationError` code. A
  /// configuration closure returns one of these to throw, as a Swift closure
  /// would.
  pub fn failed(action: impl Into<String>, message: impl fmt::Display) -> Self {
    Self::Failed {
      action: action.into(),
      message: message.to_string(),
      code: None,
    }
  }

  /// A failure of `action`, with the code of the `ContainerizationError` that
  /// Swift threw, if it threw one.
  #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
  pub(crate) fn failed_with_code(
    action: impl Into<String>,
    message: impl fmt::Display,
    code: Option<containerization_error::Code>,
  ) -> Self {
    Self::Failed {
      action: action.into(),
      message: message.to_string(),
      code,
    }
  }

  /// A call that could not be attempted off macOS.
  #[cfg_attr(target_os = "macos", allow(dead_code))]
  pub(crate) fn unavailable(action: impl Into<String>, message: impl fmt::Display) -> Self {
    Self::Unavailable {
      action: action.into(),
      message: message.to_string(),
    }
  }

  /// What was attempted, for a caller rewording the failure.
  pub fn action(&self) -> &str {
    match self {
      Self::Failed { action, .. }
      | Self::FileDescriptorOps { action, .. }
      | Self::VolumeSubpath { action, .. }
      | Self::Unavailable { action, .. } => action,
    }
  }

  /// `ContainerizationError.isCode(_:)`: whether this is a thrown
  /// `ContainerizationError` with `code`.
  pub fn is_code(&self, code: containerization_error::Code) -> bool {
    matches!(self, Self::Failed { code: Some(thrown), .. } if *thrown == code)
  }
}

impl fmt::Display for Error {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Failed { action, message, .. } => write!(formatter, "could not {action}: {message}"),
      Self::FileDescriptorOps { action, error } => write!(formatter, "could not {action}: {error}"),
      Self::VolumeSubpath { action, error } => write!(formatter, "could not {action}: {error}"),
      Self::Unavailable { action, message } => write!(formatter, "cannot {action}: {message}"),
    }
  }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn says_what_was_attempted_and_what_it_said() {
    assert_eq!(
      Error::failed("boot session-one", "no kernel at /nowhere").to_string(),
      "could not boot session-one: no kernel at /nowhere"
    );
    assert_eq!(
      Error::unavailable("read the image store", "it has never been provisioned").to_string(),
      "cannot read the image store: it has never been provisioned"
    );
  }

  #[test]
  fn names_the_action_without_its_message() {
    assert_eq!(Error::failed("boot session-one", "nope").action(), "boot session-one");
  }

  #[test]
  fn has_only_the_code_it_was_thrown_with() {
    let not_found = containerization_error::Code::NotFound;
    let error = Error::failed_with_code("get alpine", "not found", Some(not_found));
    assert!(error.is_code(not_found));
    assert!(!error.is_code(containerization_error::Code::Exists));
    assert!(!Error::failed("get alpine", "not found").is_code(not_found));
    assert!(!Error::unavailable("get alpine", "not on macOS").is_code(not_found));
  }

  #[test]
  fn says_what_file_descriptor_ops_threw() {
    let error = Error::FileDescriptorOps {
      action: "open the file config.json".to_string(),
      error: file_descriptor_ops::Error::Conflict(file_descriptor_ops::EntryType::Directory),
    };

    assert_eq!(error.action(), "open the file config.json");
    assert_eq!(
      error.to_string(),
      "could not open the file config.json: a directory entry is in the way of a file descriptor operation"
    );
  }
}
