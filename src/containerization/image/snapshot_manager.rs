//! `SnapshotManager`, and the options for its `unpack`: [`UnpackOptions`].

use super::Image;
use crate::containerization::container;
use crate::containerization_extras;
use crate::containerization_oci;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;

/// `SnapshotManager.unpack(_:for:progress:)`'s defaulted arguments.
/// [`Default`] uses Swift's defaults: `Platform.current` and no progress
/// handler.
#[derive(Default)]
pub struct UnpackOptions {
  pub platform: Option<containerization_oci::image::Platform>,
  pub progress: Option<containerization_extras::ProgressHandler>,
}

/// Leaves out the progress handler, which is a closure.
impl fmt::Debug for UnpackOptions {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("UnpackOptions")
      .field("platform", &self.platform)
      .finish_non_exhaustive()
  }
}

/// `SnapshotManager`, which keeps each image's root filesystem unpacked once,
/// as an ext4 snapshot, so that containers can start from a copy-on-write
/// clone of it.
///
/// On macOS a clone uses `clonefile(2)`, so the snapshots' directory must be
/// on the same APFS volume as the clones, for example in a `snapshots`
/// directory under the image store's path.
pub struct SnapshotManager {
  pub(crate) handle: ffi::CzSnapshotManager,
}

opaque_debug!(SnapshotManager);

// Swift's `SnapshotManager` is `Sendable`.
unsafe impl Send for SnapshotManager {}
unsafe impl Sync for SnapshotManager {}

impl SnapshotManager {
  /// `SnapshotManager(path:capacityInBytes:logger:)`, which creates the
  /// directory if it is missing. Swift's `capacityInBytes`, the least
  /// capacity of each new snapshot's sparse filesystem, defaults to 512 GiB.
  pub fn new(path: &Path, capacity_in_bytes: u64) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_snapshot_manager_new(platform::path(path)?, capacity_in_bytes),
      format!("make a snapshot manager in {}", path.display()),
    )
    .map(|outcome| Self {
      handle: outcome.snapshot_manager(),
    })
  }

  /// `SnapshotManager.path`, the directory that holds the snapshots.
  pub fn path(&self) -> PathBuf {
    PathBuf::from(self.handle.path())
  }

  /// `SnapshotManager.unpack(_:for:progress:)`, which returns the path of the
  /// image's snapshot and unpacks the image first if it has none.
  pub fn unpack(&self, image: &Image, options: UnpackOptions) -> Result<PathBuf, Error> {
    let (has_platform, platform) = containerization_oci::image::Platform::crossing(options.platform.as_ref());

    platform::outcome(
      platform::progressing(options.progress, |progress| {
        self
          .handle
          .unpack(image.handle.duplicate(), has_platform, platform, progress)
      }),
      format!("unpack a snapshot of {}", image.reference()),
    )
    .map(|outcome| PathBuf::from(outcome.text()))
  }

  /// `SnapshotManager.get(_:for:)`, the path of the image's snapshot if it
  /// has one. A `None` platform uses Swift's default, `Platform.current`.
  pub fn get(
    &self,
    image: &Image,
    platform: Option<&containerization_oci::image::Platform>,
  ) -> Result<Option<PathBuf>, Error> {
    let (has_platform, platform) = containerization_oci::image::Platform::crossing(platform);

    platform::outcome(
      self
        .handle
        .get(image.handle.duplicate(), has_platform, platform),
      format!("find the snapshot of {}", image.reference()),
    )
    .map(|outcome| outcome.is_some().then(|| PathBuf::from(outcome.text())))
  }

  /// `SnapshotManager.key(of:for:)`, the key of the image's snapshot whether
  /// or not the snapshot exists. A `None` platform uses Swift's default,
  /// `Platform.current`.
  pub fn key(
    image: &Image,
    platform: Option<&containerization_oci::image::Platform>,
  ) -> Result<containerization_oci::content::ParsedDigest, Error> {
    let (has_platform, platform) = containerization_oci::image::Platform::crossing(platform);

    platform::outcome(
      ffi::cz_snapshot_manager_key(image.handle.duplicate(), has_platform, platform),
      format!("key the snapshot of {}", image.reference()),
    )
    .map(|outcome| containerization_oci::content::ParsedDigest {
      encoded: outcome.parsed_digest_encoded(),
    })
  }

  /// `SnapshotManager.clone(_:for:to:)`, which clones the image's snapshot
  /// to `to` and returns its mount. It isn't named `clone` because that would
  /// hide [`Clone::clone`]. A `None` platform uses Swift's default,
  /// `Platform.current`.
  pub fn clone_to(
    &self,
    image: &Image,
    platform: Option<&containerization_oci::image::Platform>,
    to: &Path,
  ) -> Result<container::Mount, Error> {
    let (has_platform, platform) = containerization_oci::image::Platform::crossing(platform);

    platform::outcome(
      self
        .handle
        .clone_to(image.handle.duplicate(), has_platform, platform, platform::path(to)?),
      format!("clone the snapshot of {} to {}", image.reference(), to.display()),
    )
    .map(|outcome| outcome.mount())
  }

  /// `SnapshotManager.delete(keeping:)`, which deletes every snapshot that
  /// none of `keeping` references. It returns the keys of the snapshots it
  /// deleted and the bytes it freed, which Swift labels `deleted` and `freed`.
  pub fn delete(&self, keeping: &[Image]) -> Result<(Vec<String>, u64), Error> {
    let images = ffi::cz_image_list_new();
    for image in keeping {
      images.push(image.handle.duplicate());
    }
    let outcome = platform::outcome(self.handle.delete(images), "delete snapshots")?;

    Ok((outcome.strings(), outcome.number()))
  }
}

/// Swift's `SnapshotManager` is a struct, so a copy manages the same
/// directory.
impl Clone for SnapshotManager {
  fn clone(&self) -> Self {
    Self {
      handle: self.handle.duplicate(),
    }
  }
}
