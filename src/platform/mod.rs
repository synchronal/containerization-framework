//! The bridge on macOS, its stand-in elsewhere, and how either one's failure
//! reads. Every wrapper calls through here, so each is written once for every
//! platform.

#[cfg(not(target_os = "macos"))]
pub(crate) mod unsupported;

#[cfg(target_os = "macos")]
pub(crate) use crate::bridge::ffi;

#[cfg(not(target_os = "macos"))]
pub(crate) use self::unsupported as ffi;

use crate::containerization;
use crate::error::Error;
use std::time::Duration;
use std::time::SystemTime;

/// What a throwing Swift call returned, or what it threw as a failure of
/// `action`.
pub(crate) fn outcome(outcome: ffi::CzOutcome, action: impl Into<String>) -> Result<ffi::CzOutcome, Error> {
  match outcome.error() {
    Some(message) => Err(error(action, message)),
    None => Ok(outcome),
  }
}

#[cfg(target_os = "macos")]
fn error(action: impl Into<String>, message: String) -> Error {
  Error::failed(action, message)
}

/// Elsewhere the bridge only ever fails, and nothing was attempted.
#[cfg(not(target_os = "macos"))]
fn error(action: impl Into<String>, message: String) -> Error {
  Error::unavailable(action, message)
}

/// `ExitStatus`, its `exitedAt` crossing as seconds since 1970.
pub(crate) fn exit_status(outcome: &ffi::CzOutcome) -> containerization::ExitStatus {
  containerization::ExitStatus {
    exit_code: outcome.exit_code(),
    exited_at: SystemTime::UNIX_EPOCH + Duration::from_secs_f64(outcome.exited_at().max(0.0)),
  }
}
