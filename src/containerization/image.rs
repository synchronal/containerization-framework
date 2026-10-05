//! `Image`, and its nested `Image.Description`.

use crate::containerization_oci;
use crate::containerization_oci::Content;
use crate::containerization_oci::Descriptor;
use crate::containerization_oci::Index;
use crate::containerization_oci::LocalContentStore;
use crate::containerization_oci::Manifest;
use crate::containerization_oci::Platform;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `Image.Description`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Description {
  pub reference: String,
  pub descriptor: Descriptor,
}

impl Description {
  /// `Image.Description(reference:descriptor:)`.
  pub fn new(reference: impl Into<String>, descriptor: Descriptor) -> Self {
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
fn named(platform: &Platform) -> String {
  format!("{}/{}", platform.os, platform.architecture)
}

/// `Image`.
pub struct Image {
  pub(crate) handle: ffi::CzImage,
}

// Swift's `Image` is `Sendable`.
unsafe impl Send for Image {}
unsafe impl Sync for Image {}

impl Image {
  /// `Image(description:contentStore:)`.
  pub fn new(description: &Description, content_store: &LocalContentStore) -> Self {
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
  pub fn descriptor(&self) -> Descriptor {
    self.handle.descriptor().descriptor()
  }

  /// `Image.index()`.
  pub fn index(&self) -> Result<Index, Error> {
    platform::outcome(self.handle.index(), format!("read the index of {}", self.reference()))
      .map(|outcome| outcome.index())
  }

  /// `Image.manifest(for:)`.
  pub fn manifest(&self, for_platform: &Platform) -> Result<Manifest, Error> {
    platform::outcome(
      self.handle.manifest(for_platform.clone()),
      format!("read the manifest of {} for {}", self.reference(), named(for_platform)),
    )
    .map(|outcome| outcome.manifest())
  }

  /// `Image.descriptor(for:)`. Rust has no overloading, so the suffix names
  /// the argument label that tells it apart from [`Self::descriptor`].
  pub fn descriptor_for(&self, for_platform: &Platform) -> Result<Descriptor, Error> {
    platform::outcome(
      self.handle.descriptor_for(for_platform.clone()),
      format!("find the manifest of {} for {}", self.reference(), named(for_platform)),
    )
    .map(|outcome| outcome.descriptor())
  }

  /// `Image.config(for:)`.
  pub fn config(&self, for_platform: &Platform) -> Result<containerization_oci::Image, Error> {
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
  pub fn get_content(&self, digest: &str) -> Result<Content, Error> {
    platform::outcome(
      self.handle.get_content(digest),
      format!("get {digest} from {}", self.reference()),
    )
    .map(|outcome| Content {
      handle: outcome.content(),
    })
  }
}
