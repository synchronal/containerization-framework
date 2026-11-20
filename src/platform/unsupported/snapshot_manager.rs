//! Stand-ins for `SnapshotManager.swift`: `SnapshotManager`, a
//! `SnapshotManager?` and the `[Image]` its `delete` keeps.

use super::CzImage;
use super::CzOutcome;
use crate::containerization_oci;
use crate::platform;
use std::convert::Infallible;

/// A `SnapshotManager?`, which is only ever `nil` here.
pub(crate) struct CzSnapshots;

pub(crate) fn cz_no_snapshots() -> CzSnapshots {
  CzSnapshots
}

/// An `[Image]`, which is only ever empty here, since no image exists.
pub(crate) struct CzImageList;

pub(crate) fn cz_image_list_new() -> CzImageList {
  CzImageList
}

impl CzImageList {
  pub(crate) fn push(&self, image: CzImage) {
    match image.0 {}
  }
}

taken!(snapshot_manager -> CzSnapshotManager);

handles!(CzSnapshotManager);

failing!(cz_snapshot_manager_new(&str, u64));

pub(crate) fn cz_snapshot_manager_key(
  image: CzImage,
  _has_platform: bool,
  _platform: containerization_oci::image::Platform,
) -> CzOutcome {
  match image.0 {}
}

uninhabited!(CzSnapshotManager {
  fn duplicate() -> CzSnapshotManager;
  fn path() -> String;
  fn unpack(CzImage, bool, containerization_oci::image::Platform, platform::Progress) -> CzOutcome;
  fn get(CzImage, bool, containerization_oci::image::Platform) -> CzOutcome;
  fn clone_to(CzImage, bool, containerization_oci::image::Platform, &str) -> CzOutcome;
  fn delete(CzImageList) -> CzOutcome;
  fn as_snapshots() -> CzSnapshots;
});
