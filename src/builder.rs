//! Building images.
//!
//! Pull the base, unpack it to a writable ext4 block, boot it, run each step,
//! export the block to a tar, and store that as a single-layer image — all
//! in-process, with no daemon and no builder image.
//!
//! The rootfs is snapshotted after each step under that step's
//! [`cache_key`](crate::BuildStep::cache_key); a rebuild resumes from the
//! deepest match. The image is one layer regardless of step count: the cache is
//! block snapshots, not layers.

use crate::error::Error;
use crate::model;
use crate::pins;
use crate::platform::checked;
use crate::platform::ffi;
use crate::store::Store;

pub struct Builder {
  store: Store,
}

impl Builder {
  pub fn new(store: Store) -> Self {
    Self { store }
  }

  pub fn store(&self) -> &Store {
    &self.store
  }

  /// Builds the plan's image, streaming the log to stderr. Returns once it is
  /// stored under `plan.tag`.
  pub fn build(&self, plan: &model::BuildPlan) -> Result<(), Error> {
    let code = ffi::czbridge_build(
      plan.clone(),
      &self.store.root().display().to_string(),
      &self.store.kernel().display().to_string(),
      self.store.initfs_reference(),
      &self.store.initfs().display().to_string(),
    );

    checked(code, format!("build {}", plan.tag)).map(|_| ())
  }

  /// Fetches the kernel to [`Store::kernel`] and unpacks the init image to
  /// [`Store::initfs`], if missing. Idempotent and cheap.
  pub fn provision(&self) -> Result<(), Error> {
    let code = ffi::czbridge_provision(
      &self.store.root().display().to_string(),
      &self.store.kernel().display().to_string(),
      pins::KERNEL_URL,
      pins::KERNEL_IN_ARCHIVE,
      self.store.initfs_reference(),
      &self.store.initfs().display().to_string(),
    );

    checked(code, "provision the image store").map(|_| ())
  }
}
