use super::Image;
use super::ImageStore;
use crate::containerization::vm;
use crate::containerization_oci;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::collections::BTreeMap;
use std::fmt;

/// `KernelImage`.
pub struct KernelImage {
  handle: ffi::CzKernelImage,
}

impl fmt::Debug for KernelImage {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("KernelImage")
      .finish_non_exhaustive()
  }
}

// Swift's `KernelImage` is `Sendable`.
unsafe impl Send for KernelImage {}
unsafe impl Sync for KernelImage {}

impl KernelImage {
  /// `KernelImage.mediaType`.
  pub const MEDIA_TYPE: &str = "application/vnd.apple.containerization.kernel";

  /// `KernelImage(image:)`.
  pub fn new(image: &Image) -> Self {
    Self {
      handle: image.handle.kernel_image(),
    }
  }

  /// `KernelImage.name`.
  pub fn name(&self) -> String {
    self.handle.name()
  }

  /// `KernelImage.kernel(for:)`.
  pub fn kernel(&self, platform: vm::SystemPlatform) -> Result<vm::Kernel, Error> {
    platform::outcome(
      self.handle.kernel(platform),
      format!("find the kernel in {}", self.name()),
    )
    .map(|outcome| outcome.kernel())
  }

  /// `KernelImage.create(reference:binaries:labels:imageStore:contentStore:)`.
  pub fn create(
    reference: &str,
    binaries: &[vm::Kernel],
    labels: &BTreeMap<String, String>,
    image_store: &ImageStore,
    content_store: &containerization_oci::content::LocalContentStore,
  ) -> Result<Self, Error> {
    platform::outcome(
      image_store.handle.create_kernel_image(
        reference,
        binaries.to_vec(),
        labels.keys().cloned().collect(),
        labels.values().cloned().collect(),
        content_store.handle.duplicate(),
      ),
      format!("create kernel image {reference}"),
    )
    .map(|outcome| Self {
      handle: outcome.kernel_image(),
    })
  }
}
