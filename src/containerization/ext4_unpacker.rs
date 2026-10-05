use super::Image;
use super::Mount;
use crate::containerization_archive::Filter;
use crate::containerization_ext4::ext4::JournalConfig;
use crate::containerization_extras::ProgressHandler;
use crate::containerization_oci::Platform;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::path::Path;

/// `EXT4Unpacker`. A value, as Swift's is a struct, and its fields are private,
/// as Swift's are.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ext4Unpacker {
  pub(crate) capacity_in_bytes: u64,
  pub(crate) journal: Option<JournalConfig>,
}

impl Ext4Unpacker {
  /// `EXT4Unpacker(capacityInBytes:journal:)`.
  pub fn new(capacity_in_bytes: u64, journal: Option<JournalConfig>) -> Self {
    Self {
      capacity_in_bytes,
      journal,
    }
  }

  /// `EXT4Unpacker.unpack(_:for:at:progress:)`.
  pub fn unpack(
    &self,
    image: &Image,
    platform: &Platform,
    at: &Path,
    progress: Option<ProgressHandler>,
  ) -> Result<Mount, Error> {
    platform::outcome(
      ffi::cz_ext4_unpack(
        *self,
        image.handle.duplicate(),
        platform.clone(),
        &at.display().to_string(),
        platform::Progress(progress),
      ),
      format!("unpack {} to {}", image.reference(), at.display()),
    )
    .map(|outcome| outcome.mount())
  }

  /// `EXT4Unpacker.unpack(archive:compression:at:)`.
  pub fn unpack_archive(&self, archive: &Path, compression: Filter, at: &Path) -> Result<(), Error> {
    platform::outcome(
      ffi::cz_ext4_unpack_archive(
        *self,
        &archive.display().to_string(),
        compression.raw_value(),
        &at.display().to_string(),
      ),
      format!("unpack {} to {}", archive.display(), at.display()),
    )
    .map(|_| ())
  }
}
