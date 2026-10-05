use super::Descriptor;
use super::MediaTypes;
use std::collections::BTreeMap;

/// `Manifest`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Manifest {
  pub schema_version: isize,
  pub media_type: Option<String>,
  pub config: Descriptor,
  pub layers: Vec<Descriptor>,
  pub annotations: Option<BTreeMap<String, String>>,
  pub subject: Option<Descriptor>,
  pub artifact_type: Option<String>,
}

impl Manifest {
  /// `Manifest(config:layers:)`, its other arguments at their defaults:
  /// schema version 2, `MediaTypes.imageManifest`, and nothing else.
  pub fn new(config: Descriptor, layers: Vec<Descriptor>) -> Self {
    Self {
      schema_version: 2,
      media_type: Some(MediaTypes::IMAGE_MANIFEST.to_string()),
      config,
      layers,
      annotations: None,
      subject: None,
      artifact_type: None,
    }
  }
}
