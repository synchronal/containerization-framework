//! Images, the store they are kept in, and unpacking them into a root
//! filesystem. `Image` is here, with its nested `Image.Description`.

mod ext4_unpacker;
pub mod image_store;
mod init_image;
mod kernel_image;

pub use self::ext4_unpacker::EXT4Unpacker;
pub use self::image_store::ImageStore;
pub use self::init_image::InitImage;
pub use self::kernel_image::KernelImage;

use crate::containerization_oci;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::fmt;

/// `Image.Description`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Description {
  pub reference: String,
  pub descriptor: containerization_oci::image::Descriptor,
}

impl Description {
  /// `Image.Description(reference:descriptor:)`.
  pub fn new(reference: impl Into<String>, descriptor: containerization_oci::image::Descriptor) -> Self {
    Self {
      reference: reference.into(),
      descriptor,
    }
  }

  /// `Image.Description.digest`: the descriptor's.
  pub fn digest(&self) -> &str {
    &self.descriptor.digest
  }

  /// `Image.Description.mediaType`: the descriptor's.
  pub fn media_type(&self) -> &str {
    &self.descriptor.media_type
  }
}

/// A platform in an error's action, without asking Swift for its description.
fn named(platform: &containerization_oci::image::Platform) -> String {
  format!("{}/{}", platform.os, platform.architecture)
}

/// `Image`.
pub struct Image {
  pub(crate) handle: ffi::CzImage,
}

impl fmt::Debug for Image {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.debug_struct("Image").finish_non_exhaustive()
  }
}

// Swift's `Image` is `Sendable`.
unsafe impl Send for Image {}
unsafe impl Sync for Image {}

impl Image {
  /// `Image(description:contentStore:)`.
  pub fn new(description: &Description, content_store: &containerization_oci::content::LocalContentStore) -> Self {
    Self {
      handle: content_store.handle.image(description.clone()),
    }
  }

  /// `Image.reference`.
  pub fn reference(&self) -> String {
    self.handle.reference()
  }

  /// `Image.digest`.
  pub fn digest(&self) -> String {
    self.handle.digest()
  }

  /// `Image.mediaType`.
  pub fn media_type(&self) -> String {
    self.handle.media_type()
  }

  /// `Image.description`.
  pub fn description(&self) -> Description {
    Description::new(self.reference(), self.descriptor())
  }

  /// `Image.descriptor`.
  pub fn descriptor(&self) -> containerization_oci::image::Descriptor {
    self.handle.descriptor().descriptor()
  }

  /// `Image.index()`.
  pub fn index(&self) -> Result<containerization_oci::image::Index, Error> {
    platform::outcome(self.handle.index(), format!("read the index of {}", self.reference()))
      .map(|outcome| outcome.index())
  }

  /// `Image.manifest(for:)`.
  pub fn manifest(
    &self,
    for_platform: &containerization_oci::image::Platform,
  ) -> Result<containerization_oci::image::Manifest, Error> {
    platform::outcome(
      self.handle.manifest(for_platform.clone()),
      format!("read the manifest of {} for {}", self.reference(), named(for_platform)),
    )
    .map(|outcome| outcome.manifest())
  }

  /// `Image.descriptor(for:)`. Rust has no overloading, so the suffix names
  /// the argument label that tells it apart from [`Self::descriptor`].
  pub fn descriptor_for(
    &self,
    for_platform: &containerization_oci::image::Platform,
  ) -> Result<containerization_oci::image::Descriptor, Error> {
    platform::outcome(
      self.handle.descriptor_for(for_platform.clone()),
      format!("find the manifest of {} for {}", self.reference(), named(for_platform)),
    )
    .map(|outcome| outcome.descriptor())
  }

  /// `Image.config(for:)`.
  pub fn config(
    &self,
    for_platform: &containerization_oci::image::Platform,
  ) -> Result<containerization_oci::image::Image, Error> {
    platform::outcome(
      self.handle.config(for_platform.clone()),
      format!("read the config of {} for {}", self.reference(), named(for_platform)),
    )
    .map(|outcome| outcome.oci_image())
  }

  /// `Image.referencedDigests()`.
  pub fn referenced_digests(&self) -> Result<Vec<String>, Error> {
    platform::outcome(
      self.handle.referenced_digests(),
      format!("list what {} is made of", self.reference()),
    )
    .map(|outcome| outcome.strings())
  }

  /// `Image.getContent(digest:)`.
  pub fn get_content(&self, digest: &str) -> Result<containerization_oci::content::Content, Error> {
    platform::outcome(
      self.handle.get_content(digest),
      format!("get {digest} from {}", self.reference()),
    )
    .map(|outcome| containerization_oci::content::Content {
      handle: outcome.content(),
    })
  }
}
