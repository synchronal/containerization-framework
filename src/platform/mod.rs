//! The bridge on macOS, its stand-in elsewhere, and how either one's failure
//! reads. [`crate::Session`] and [`crate::Builder`] call through here, so they
//! are written once for every platform.

#[cfg(not(target_os = "macos"))]
pub(crate) mod unsupported;

#[cfg(target_os = "macos")]
pub(crate) use crate::bridge::ffi;

#[cfg(not(target_os = "macos"))]
pub(crate) use self::unsupported as ffi;

/// The bridge's failure code; can't collide with a guest exit code (0...255).
const FAILED: i32 = -1;

/// Turns the bridge's `-1` into a failure of `action`, with the message Swift
/// left behind.
#[cfg(target_os = "macos")]
pub(crate) fn checked(code: i32, action: impl Into<String>) -> Result<i32, crate::Error> {
  if code == FAILED {
    return Err(crate::Error::failed(action, ffi::czbridge_last_error()));
  }

  Ok(code)
}

/// Elsewhere the bridge only ever fails, and nothing was attempted.
#[cfg(not(target_os = "macos"))]
pub(crate) fn checked(code: i32, action: impl Into<String>) -> Result<i32, crate::Error> {
  if code == FAILED {
    return Err(crate::Error::unavailable(
      action,
      "Containerization.framework is macOS only",
    ));
  }

  Ok(code)
}
