//! `Sysctl`.

use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `Sysctl.byName(_:)`: the value of the integer sysctl `name`, such as
/// `hw.memsize`.
pub fn by_name(name: &str) -> Result<i64, Error> {
  platform::outcome(ffi::cz_sysctl_by_name(name), format!("read the sysctl {name}")).map(|outcome| outcome.integer())
}
