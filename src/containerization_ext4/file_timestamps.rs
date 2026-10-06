use crate::platform;
use std::time::SystemTime;

/// `FileTimestamps`. [`Default`] is Swift's `init()`, which sets every date
/// to now.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileTimestamps {
  pub access: SystemTime,
  pub modification: SystemTime,
  pub creation: SystemTime,
  pub now: SystemTime,
}

impl FileTimestamps {
  /// `FileTimestamps(access:modification:creation:)`. A missing date is now.
  pub fn new(access: Option<SystemTime>, modification: Option<SystemTime>, creation: Option<SystemTime>) -> Self {
    let now = SystemTime::now();

    Self {
      access: access.unwrap_or(now),
      modification: modification.unwrap_or(now),
      creation: creation.unwrap_or(now),
      now,
    }
  }

  /// `accessLo`.
  pub fn access_lo(&self) -> u32 {
    lo(self.access)
  }

  /// `accessHi`.
  pub fn access_hi(&self) -> u32 {
    hi(self.access)
  }

  /// `modificationLo`.
  pub fn modification_lo(&self) -> u32 {
    lo(self.modification)
  }

  /// `modificationHi`.
  pub fn modification_hi(&self) -> u32 {
    hi(self.modification)
  }

  /// `creationLo`.
  pub fn creation_lo(&self) -> u32 {
    lo(self.creation)
  }

  /// `creationHi`.
  pub fn creation_hi(&self) -> u32 {
    hi(self.creation)
  }

  /// `nowLo`.
  pub fn now_lo(&self) -> u32 {
    lo(self.now)
  }

  /// `nowHi`.
  pub fn now_hi(&self) -> u32 {
    hi(self.now)
  }
}

impl Default for FileTimestamps {
  fn default() -> Self {
    Self::new(None, None, None)
  }
}

fn lo(time: SystemTime) -> u32 {
  fs(time) as u32
}

fn hi(time: SystemTime) -> u32 {
  (fs(time) >> 32) as u32
}

/// Swift's internal `Date.fs()`, which the getters above read: the low 32 bits
/// of the seconds since 1970, two bits counting how often they wrapped, and 30
/// bits of nanoseconds. The date is read as a Swift `Date` holds it, as
/// seconds in a `Double`.
pub(crate) fn fs(time: SystemTime) -> u64 {
  // `Date.distantPast`, the first instant of year 1 in Foundation's calendar.
  const DISTANT_PAST: f64 = -62_135_769_600.0;

  let seconds = platform::seconds(time);
  if seconds == DISTANT_PAST {
    return 0;
  }
  if seconds < -(0x8000_0000_i64 as f64) {
    return 0x8000_0000;
  }
  if seconds > 0x3_7fff_ffff_i64 as f64 {
    return 0x3_7fff_ffff;
  }

  let whole = seconds.floor() as i64;
  let base = whole as i32;
  let epoch = (whole - base as i64) as u64;
  let nanoseconds = (((seconds - seconds.floor()) * 1_000_000_000.0) as u32).min(999_999_999);

  base as u32 as u64 | epoch | ((nanoseconds as u64) << 34)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::time::Duration;

  #[test]
  fn takes_missing_dates_as_now() {
    let access = SystemTime::UNIX_EPOCH + Duration::from_secs(7);
    let timestamps = FileTimestamps::new(Some(access), None, None);

    assert_eq!(timestamps.access, access);
    assert_eq!(timestamps.modification, timestamps.now);
    assert_eq!(timestamps.creation, timestamps.now);
  }
}
