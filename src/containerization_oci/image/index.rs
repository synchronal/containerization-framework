use super::Descriptor;
use super::MediaTypes;
use std::collections::BTreeMap;

/// `Index`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Index {
  pub schema_version: isize,
  pub media_type: String,
  pub manifests: Vec<Descriptor>,
  pub annotations: Option<BTreeMap<String, String>>,
  pub subject: Option<Descriptor>,
  pub artifact_type: Option<String>,
}

impl Index {
  /// `Index(manifests:)`, its other arguments at their defaults: schema
  /// version 2, `MediaTypes.index`, and nothing else.
  pub fn new(manifests: Vec<Descriptor>) -> Self {
    Self {
      schema_version: 2,
      media_type: MediaTypes::INDEX.to_string(),
      manifests,
      annotations: None,
      subject: None,
      artifact_type: None,
    }
  }
}
