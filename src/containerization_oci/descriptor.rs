use super::Platform;
use std::collections::BTreeMap;

/// `Descriptor`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Descriptor {
  pub media_type: String,
  pub digest: String,
  pub size: i64,
  pub urls: Option<Vec<String>>,
  pub annotations: Option<BTreeMap<String, String>>,
  pub platform: Option<Platform>,
  pub artifact_type: Option<String>,
}

impl Descriptor {
  /// `Descriptor(mediaType:digest:size:)`, its other arguments at their
  /// defaults.
  pub fn new(media_type: impl Into<String>, digest: impl Into<String>, size: i64) -> Self {
    Self {
      media_type: media_type.into(),
      digest: digest.into(),
      size,
      urls: None,
      annotations: None,
      platform: None,
      artifact_type: None,
    }
  }
}
