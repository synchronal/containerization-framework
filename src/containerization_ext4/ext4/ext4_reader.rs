use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::path::Path;

/// `EXT4.EXT4Reader`.
pub struct Ext4Reader {
  handle: ffi::CzExt4Reader,
}

// Swift's `EXT4Reader` is a class that isn't `Sendable`; Rust's isn't `Sync`,
// so one call runs at a time.
unsafe impl Send for Ext4Reader {}

impl Ext4Reader {
  /// `EXT4.EXT4Reader(blockDevice:)`.
  pub fn new(block_device: &Path) -> Result<Self, Error> {
    let outcome = platform::outcome(
      ffi::cz_ext4_reader_new(&block_device.display().to_string()),
      format!("read the filesystem in {}", block_device.display()),
    )?;

    Ok(Self {
      handle: outcome.ext4_reader(),
    })
  }

  /// `EXT4.EXT4Reader.export(archive:)`.
  pub fn export(&self, archive: &Path) -> Result<(), Error> {
    platform::outcome(
      self.handle.export(&archive.display().to_string()),
      format!("export the filesystem to {}", archive.display()),
    )
    .map(|_| ())
  }
}
