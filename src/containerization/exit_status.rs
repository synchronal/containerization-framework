use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::time::SystemTime;

/// `ExitStatus`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExitStatus {
  pub exit_code: i32,
  pub exited_at: SystemTime,
}

impl ExitStatus {
  /// `ExitStatus(exitCode:)`, which Swift dates now.
  pub fn new(exit_code: i32) -> Result<Self, Error> {
    platform::outcome(ffi::cz_exit_status_new(exit_code), "make an exit status")
      .map(|outcome| platform::exit_status(&outcome))
  }
}
