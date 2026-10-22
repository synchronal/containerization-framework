use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// An `Authentication`: a `BasicAuthentication`, or what
/// [`super::KeychainHelper::lookup`] returned. Swift's is a protocol, and its
/// conforming types keep their credentials internal, so Rust holds Swift's
/// value and can't read them.
pub struct Authentication {
  pub(crate) handle: ffi::CzAuthentication,
}

opaque_debug!(
  /// Without the credentials, which Rust can't read.
  Authentication
);

// Swift's `Authentication` is `Sendable`.
unsafe impl Send for Authentication {}
unsafe impl Sync for Authentication {}

impl Authentication {
  /// `BasicAuthentication(username:password:)`.
  pub fn basic(username: &str, password: &str) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_basic_authentication(username, password),
      "make a basic authentication",
    )
    .map(|outcome| Self {
      handle: outcome.authentication(),
    })
  }

  /// `Authentication.token()`.
  pub fn token(&self) -> Result<String, Error> {
    platform::outcome(self.handle.token(), "make an authentication token").map(|outcome| outcome.text())
  }

  /// An `Authentication?`, as it crosses to Swift.
  pub(crate) fn crossing(auth: Option<&Self>) -> Result<ffi::CzAuthentication, Error> {
    match auth {
      Some(auth) => Ok(auth.handle.duplicate()),
      None => {
        platform::outcome(ffi::cz_no_authentication(), "pass no authentication").map(|outcome| outcome.authentication())
      }
    }
  }
}

/// Swift's conforming type, `BasicAuthentication`, is a struct, so a copy
/// holds the same credentials.
impl Clone for Authentication {
  fn clone(&self) -> Self {
    Self {
      handle: self.handle.duplicate(),
    }
  }
}
