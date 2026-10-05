//! `Image`, and its nested `Image.Description`.

use crate::containerization_oci::Content;
use crate::containerization_oci::Descriptor;
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

/// `Image`. Made by [`super::ImageStore`].
pub struct Image {
  pub(crate) handle: ffi::CzImage,
}

// Swift's `Image` is `Sendable`.
unsafe impl Send for Image {}
unsafe impl Sync for Image {}

impl Image {
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
