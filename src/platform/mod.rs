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

use crate::containerization::container;
use crate::containerization::process;
#[cfg(target_os = "macos")]
use crate::containerization_error;
use crate::containerization_extras::ProgressHandler;
use crate::error::Error;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::time::Duration;
use std::time::SystemTime;

/// A `ProgressHandler?`, as it crosses to Swift.
pub(crate) struct Progress(pub(crate) Option<ProgressHandler>);

type Configuring<T> = Box<dyn FnOnce(&mut T) + Send>;

/// An `(inout T) -> Void` closure, as it crosses to Swift. Swift calls it
/// once, with the value it filled.
pub(crate) struct Configure<T>(Mutex<Option<Configuring<T>>>);

impl<T> Configure<T> {
  pub(crate) fn new(configure: impl FnOnce(&mut T) + Send + 'static) -> Self {
    Self(Mutex::new(Some(Box::new(configure))))
  }

  /// Runs the closure on the value Swift filled, the first time only.
  pub(crate) fn call(&self, value: &mut T) {
    let configure = self.0.lock().unwrap_or_else(PoisonError::into_inner).take();

    if let Some(configure) = configure {
      configure(value);
    }
  }
}

/// The manager's `(inout LinuxContainer.Configuration) -> Void`.
pub(crate) type ConfigureContainer = Configure<container::linux_container::Configuration>;

/// An `(inout LinuxProcessConfiguration) -> Void`.
pub(crate) type ConfigureProcess = Configure<process::LinuxProcessConfiguration>;

/// An `(inout LinuxPod.Configuration) -> Void`.
pub(crate) type ConfigurePod = Configure<container::linux_pod::Configuration>;

/// An `(inout LinuxPod.ContainerConfiguration) -> Void`.
pub(crate) type ConfigurePodContainer = Configure<container::linux_pod::ContainerConfiguration>;

/// What `call` returned, and the value its Swift call filled and handed to
/// the closure `call` passes it, if it did.
pub(crate) fn filled<T: Default + Send + 'static, R>(call: impl FnOnce(Configure<T>) -> R) -> (R, Option<T>) {
  let filled = Arc::new(Mutex::new(None));
  let slot = Arc::clone(&filled);
  let receive = Configure::new(move |value| {
    *slot.lock().unwrap_or_else(PoisonError::into_inner) = Some(std::mem::take(value));
  });

  let returned = call(receive);
  let value = filled.lock().unwrap_or_else(PoisonError::into_inner).take();

  (returned, value)
}

/// The process configuration a throwing Swift call filled and handed to the
/// closure `call` passes it.
pub(crate) fn filled_process(
  call: impl FnOnce(ConfigureProcess) -> ffi::CzOutcome,
  action: impl Into<String>,
) -> Result<process::LinuxProcessConfiguration, Error> {
  let (returned, process) = filled(call);

  outcome(returned, action)?;
  Ok(process.expect("Swift hands back the configuration it fills"))
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
pub(crate) fn exit_status(outcome: &ffi::CzOutcome) -> process::ExitStatus {
  process::ExitStatus {
    exit_code: outcome.exit_code(),
    exited_at: SystemTime::UNIX_EPOCH + Duration::from_secs_f64(outcome.exited_at().max(0.0)),
  }
}

/// A Swift `Date`, crossing as seconds since 1970, which are negative before
/// it.
pub(crate) fn system_time(seconds: f64) -> SystemTime {
  if seconds < 0.0 {
    SystemTime::UNIX_EPOCH - Duration::from_secs_f64(-seconds)
  } else {
    SystemTime::UNIX_EPOCH + Duration::from_secs_f64(seconds)
  }
}

/// A struct Swift sends as its bytes, laid out as Swift's is.
///
/// # Safety
///
/// It is `repr(C)`, and any bytes of its size make a valid one.
pub(crate) unsafe trait Plain: Copy {}

/// A value from the bytes Swift sent for it. It panics if Swift's size isn't
/// Rust's.
pub(crate) fn from_bytes<T: Plain>(bytes: &[u8]) -> T {
  assert_eq!(
    bytes.len(),
    std::mem::size_of::<T>(),
    "Swift's {} is a different size from Rust's",
    std::any::type_name::<T>()
  );

  // SAFETY: `Plain` makes any bytes of its size valid, and the read doesn't
  // assume the bytes are aligned.
  unsafe { bytes.as_ptr().cast::<T>().read_unaligned() }
}

/// A `SystemTime` as a Swift `Date`'s seconds since 1970.
pub(crate) fn seconds(time: SystemTime) -> f64 {
  match time.duration_since(SystemTime::UNIX_EPOCH) {
    Ok(after) => after.as_secs_f64(),
    Err(before) => -before.duration().as_secs_f64(),
  }
}

/// A `[String: Data]` of extended attributes as it crosses to Swift: the
/// names, each value's length, and the values joined into one.
pub(crate) fn xattrs(xattrs: &BTreeMap<String, Vec<u8>>) -> (Vec<String>, Vec<u64>, Vec<u8>) {
  (
    xattrs.keys().cloned().collect(),
    xattrs.values().map(|value| value.len() as u64).collect(),
    xattrs.values().flatten().copied().collect(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn crosses_dates_on_either_side_of_1970() {
    for seconds in [-86_400.5, 0.0, 1_700_000_000.25] {
      assert_eq!(super::seconds(system_time(seconds)), seconds);
    }
  }
}
