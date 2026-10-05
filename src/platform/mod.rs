//! The bridge on macOS, its stand-in elsewhere, and how either one's failure
//! reads. Every wrapper calls through here, so each is written once for every
//! platform.

// The bridge's functions take Swift's arguments, however many there are.
#[cfg(not(target_os = "macos"))]
#[allow(clippy::too_many_arguments)]
pub(crate) mod unsupported;

#[cfg(target_os = "macos")]
pub(crate) use crate::bridge::ffi;

#[cfg(not(target_os = "macos"))]
pub(crate) use self::unsupported as ffi;

use crate::containerization;
use crate::containerization::linux_container;
#[cfg(target_os = "macos")]
use crate::containerization_error;
use crate::containerization_extras::ProgressHandler;
use crate::error::Error;
use std::sync::Mutex;
use std::time::Duration;
use std::time::SystemTime;

/// A `ProgressHandler?`, as it crosses to Swift.
pub(crate) struct Progress(pub(crate) Option<ProgressHandler>);

type ConfigureContainer = Box<dyn FnOnce(&mut linux_container::Configuration) + Send>;

/// A `(inout LinuxContainer.Configuration) -> Void` closure, as it crosses to
/// Swift. Swift calls it once, with the configuration it seeded.
pub(crate) struct Configure(pub(crate) Mutex<Option<ConfigureContainer>>);

impl Configure {
  pub(crate) fn new(configure: impl FnOnce(&mut linux_container::Configuration) + Send + 'static) -> Self {
    Self(Mutex::new(Some(Box::new(configure))))
  }
}

/// What a throwing Swift call returned, or what it threw as a failure of
/// `action`.
pub(crate) fn outcome(outcome: ffi::CzOutcome, action: impl Into<String>) -> Result<ffi::CzOutcome, Error> {
  match outcome.error() {
    Some(message) => Err(error(&outcome, action, message)),
    None => Ok(outcome),
  }
}

#[cfg(target_os = "macos")]
fn error(outcome: &ffi::CzOutcome, action: impl Into<String>, message: String) -> Error {
  let code = outcome
    .error_code()
    .and_then(|code| containerization_error::Code::from_description(&code));
  Error::failed_with_code(action, message, code)
}

/// Elsewhere the bridge only ever fails, and nothing was attempted.
#[cfg(not(target_os = "macos"))]
fn error(_outcome: &ffi::CzOutcome, action: impl Into<String>, message: String) -> Error {
  Error::unavailable(action, message)
}

/// `ExitStatus`, its `exitedAt` crossing as seconds since 1970.
pub(crate) fn exit_status(outcome: &ffi::CzOutcome) -> containerization::ExitStatus {
  containerization::ExitStatus {
    exit_code: outcome.exit_code(),
    exited_at: SystemTime::UNIX_EPOCH + Duration::from_secs_f64(outcome.exited_at().max(0.0)),
  }
}
