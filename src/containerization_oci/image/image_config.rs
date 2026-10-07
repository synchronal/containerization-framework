//! `ImageConfig`, `Rootfs`, `History` and the OCI `Image` config, from
//! Swift's `ImageConfig.swift`.

use std::collections::BTreeMap;

/// `ImageConfig`. Its `Default` is `ImageConfig()`, which holds nothing.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ImageConfig {
  pub user: Option<String>,
  pub env: Option<Vec<String>>,
  pub entrypoint: Option<Vec<String>>,
  pub cmd: Option<Vec<String>>,
  pub working_dir: Option<String>,
  pub labels: Option<BTreeMap<String, String>>,
  pub stop_signal: Option<String>,
}

/// `Rootfs`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rootfs {
  pub r#type: String,
  pub diff_ids: Vec<String>,
}

impl Rootfs {
  /// `Rootfs(type:diffIDs:)`.
  pub fn new(r#type: impl Into<String>, diff_ids: Vec<String>) -> Self {
    Self {
      r#type: r#type.into(),
      diff_ids,
    }
  }
}

/// `History`. Its `Default` is `History()`, which holds nothing.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct History {
  pub created: Option<String>,
  pub created_by: Option<String>,
  pub author: Option<String>,
  pub comment: Option<String>,
  pub empty_layer: Option<bool>,
}

/// `ContainerizationOCI.Image`, an image's config. It is a different type from
/// [`crate::containerization::image::Image`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Image {
  pub created: Option<String>,
  pub author: Option<String>,
  pub architecture: String,
  pub os: String,
  pub os_version: Option<String>,
  pub os_features: Option<Vec<String>>,
  pub variant: Option<String>,
  pub config: Option<ImageConfig>,
  pub rootfs: Rootfs,
  pub history: Option<Vec<History>>,
}

impl Image {
  /// `Image(architecture:os:rootfs:)`, its other arguments at their defaults,
  /// which hold nothing.
  pub fn new(architecture: impl Into<String>, os: impl Into<String>, rootfs: Rootfs) -> Self {
    Self {
      created: None,
      author: None,
      architecture: architecture.into(),
      os: os.into(),
      os_version: None,
      os_features: None,
      variant: None,
      config: None,
      rootfs,
      history: None,
    }
  }
}
