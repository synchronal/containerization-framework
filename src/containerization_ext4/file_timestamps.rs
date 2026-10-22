use crate::platform;
use std::time::SystemTime;

/// `FileTimestamps`. [`Default`] uses Swift's `init()`, which sets every date
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

  /// `FileTimestamps.accessLo`, the low 32 bits of `access`'s seconds since
  /// 1970, as an ext4 inode stores them.
  pub fn access_lo(&self) -> u32 {
    lo(self.access)
  }

  /// `FileTimestamps.accessHi`, `access`'s ext4 extra field: two bits
  /// counting how often the seconds wrapped, and 30 bits of nanoseconds.
  pub fn access_hi(&self) -> u32 {
    hi(self.access)
  }

  /// `FileTimestamps.modificationLo`, the low 32 bits of `modification`'s
  /// seconds since 1970, as an ext4 inode stores them.
  pub fn modification_lo(&self) -> u32 {
    lo(self.modification)
  }

  /// `FileTimestamps.modificationHi`, `modification`'s ext4 extra field: two
  /// bits counting how often the seconds wrapped, and 30 bits of
  /// nanoseconds.
  pub fn modification_hi(&self) -> u32 {
    hi(self.modification)
  }

  /// `FileTimestamps.creationLo`, the low 32 bits of `creation`'s seconds
  /// since 1970, as an ext4 inode stores them.
  pub fn creation_lo(&self) -> u32 {
    lo(self.creation)
  }

  /// `FileTimestamps.creationHi`, `creation`'s ext4 extra field: two bits
  /// counting how often the seconds wrapped, and 30 bits of nanoseconds.
  pub fn creation_hi(&self) -> u32 {
    hi(self.creation)
  }

  /// `FileTimestamps.nowLo`, the low 32 bits of `now`'s seconds since 1970,
  /// as an ext4 inode stores them.
  pub fn now_lo(&self) -> u32 {
    lo(self.now)
  }

  /// `FileTimestamps.nowHi`, `now`'s ext4 extra field: two bits counting how
  /// often the seconds wrapped, and 30 bits of nanoseconds.
  pub fn now_hi(&self) -> u32 {
    hi(self.now)
  }
}

impl Default for FileTimestamps {
  fn default() -> Self {
    Self::new(None, None, None)
  }
}

impl FileTimestamps {
  /// Swift's internal `Date.fs()`, which the getters above read: the low 32
  /// bits of the seconds since 1970, two bits counting how often they
  /// wrapped, and 30 bits of nanoseconds.
  ///
  /// The date is read as a Swift `Date` holds it, as seconds since 2001 in a
  /// `Double`. Swift's `timeIntervalSince1970` adds the years before 2001
  /// back, which can change a fraction's low bits, so this does the same.
  pub(crate) fn fs(time: SystemTime) -> u64 {
    const SECONDS_FROM_1970_TO_2001: f64 = 978_307_200.0;
    // `Date.distantPast`, the first instant of year 1 in Foundation's calendar.
    const DISTANT_PAST: f64 = -63_114_076_800.0;

    let since_2001 = platform::seconds(time) - SECONDS_FROM_1970_TO_2001;
    if since_2001 == DISTANT_PAST {
      return 0;
    }

    let seconds = since_2001 + SECONDS_FROM_1970_TO_2001;
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
}

fn lo(time: SystemTime) -> u32 {
  FileTimestamps::fs(time) as u32
}

fn hi(time: SystemTime) -> u32 {
  (FileTimestamps::fs(time) >> 32) as u32
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
