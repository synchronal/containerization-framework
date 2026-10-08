use super::Image;
use super::ImageStore;
use crate::containerization::container;
use crate::containerization::vm;
use crate::containerization_oci;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

/// `InitImage`.
pub struct InitImage {
  pub(crate) handle: ffi::CzInitImage,
}

impl fmt::Debug for InitImage {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.debug_struct("InitImage").finish_non_exhaustive()
  }
}

// Swift's `InitImage` is `Sendable`.
unsafe impl Send for InitImage {}
unsafe impl Sync for InitImage {}

impl InitImage {
  /// `InitImage(image:)`.
  pub fn new(image: &Image) -> Self {
    Self {
      handle: image.handle.init_image(),
    }
  }

  /// `InitImage.create(reference:rootfs:platform:labels:imageStore:contentStore:)`.
  /// `rootfs` is a `.tar.gz` file.
  pub fn create(
    reference: &str,
    rootfs: &Path,
    platform: &containerization_oci::image::Platform,
    labels: &BTreeMap<String, String>,
    image_store: &ImageStore,
    content_store: &containerization_oci::content::LocalContentStore,
  ) -> Result<Self, Error> {
    platform::outcome(
      image_store.handle.create_init_image(
        reference,
        platform::path(rootfs)?,
        platform.clone(),
        labels.keys().cloned().collect(),
        labels.values().cloned().collect(),
        content_store.handle.duplicate(),
      ),
      format!("create init image {reference}"),
    )
    .map(|outcome| Self {
      handle: outcome.init_image(),
    })
  }

  /// `InitImage.name`.
  pub fn name(&self) -> String {
    self.handle.name()
  }

  /// `InitImage.initBlock(at:for:)`.
  pub fn init_block(&self, at: &Path, platform: vm::SystemPlatform) -> Result<container::Mount, Error> {
    platform::outcome(
      self.handle.init_block(platform::path(at)?, platform),
      format!("unpack {} to {}", self.name(), at.display()),
    )
    .map(|outcome| outcome.mount())
  }
}
