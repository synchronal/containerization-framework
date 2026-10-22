use super::Image;
use crate::containerization::container;
use crate::containerization_archive;
use crate::containerization_ext4::ext4;
use crate::containerization_extras;
use crate::containerization_oci;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::path::Path;

/// `EXT4Unpacker`. It is a value because Swift's is a struct, and its fields
/// aren't public because Swift's aren't.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EXT4Unpacker {
  pub(crate) capacity_in_bytes: u64,
  pub(crate) journal: Option<ext4::JournalConfig>,
}

impl EXT4Unpacker {
  /// `EXT4Unpacker(capacityInBytes:journal:)`. Swift's `journal` defaults to
  /// `nil`.
  pub fn new(capacity_in_bytes: u64, journal: Option<ext4::JournalConfig>) -> Self {
    Self {
      capacity_in_bytes,
      journal,
    }
  }

  /// `EXT4Unpacker.unpack(_:for:at:progress:)`. Swift's `progress` defaults
  /// to `nil`.
  pub fn unpack(
    &self,
    image: &Image,
    platform: &containerization_oci::image::Platform,
    at: &Path,
    progress: Option<containerization_extras::ProgressHandler>,
  ) -> Result<container::Mount, Error> {
    let path = platform::path(at)?;

    platform::outcome(
      platform::progressing(progress, |progress| {
        ffi::cz_ext4_unpack(*self, image.handle.duplicate(), platform.clone(), path, progress)
      }),
      format!("unpack {} to {}", image.reference(), at.display()),
    )
    .map(|outcome| outcome.mount())
  }

  /// `EXT4Unpacker.unpack(archive:compression:at:)`.
  pub fn unpack_archive(
    &self,
    archive: &Path,
    compression: containerization_archive::Filter,
    at: &Path,
  ) -> Result<(), Error> {
    platform::outcome(
      ffi::cz_ext4_unpack_archive(
        *self,
        platform::path(archive)?,
        compression.raw_value(),
        platform::path(at)?,
      ),
      format!("unpack {} to {}", archive.display(), at.display()),
    )
    .map(drop)
  }
}
