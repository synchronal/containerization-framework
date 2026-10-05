use super::Mount;
use super::SystemPlatform;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::path::Path;

/// `InitImage`. Made by [`super::ImageStore::get_init_image`].
pub struct InitImage {
  pub(crate) handle: ffi::CzInitImage,
}

// Swift's `InitImage` is `Sendable`.
unsafe impl Send for InitImage {}
unsafe impl Sync for InitImage {}

impl InitImage {
  /// `InitImage.name`.
  pub fn name(&self) -> String {
    self.handle.name()
  }

  /// `InitImage.initBlock(at:for:)`.
  pub fn init_block(&self, at: &Path, platform: SystemPlatform) -> Result<Mount, Error> {
    platform::outcome(
      self.handle.init_block(&at.display().to_string(), platform),
      format!("unpack {} to {}", self.name(), at.display()),
    )
    .map(|outcome| outcome.mount())
  }
}
