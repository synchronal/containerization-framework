use super::Spec;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::path::Path;
use std::path::PathBuf;

/// `Bundle`, an OCI bundle on disk. Every method asks Swift.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Bundle {
  path: PathBuf,
}

impl Bundle {
  /// `Bundle.create(path:spec:)`, which writes `spec` as `config.json`.
  pub fn create(path: &Path, spec: &Spec) -> Result<Self, Error> {
    Self::at(
      ffi::cz_bundle_create(&path.display().to_string(), spec.clone()),
      format!("create a bundle at {}", path.display()),
    )
  }

  /// `Bundle.create(path:spec:)` with `Data`, written unchecked.
  pub fn create_from_data(path: &Path, spec: &[u8]) -> Result<Self, Error> {
    Self::at(
      ffi::cz_bundle_create_from_data(&path.display().to_string(), spec.to_vec()),
      format!("create a bundle at {}", path.display()),
    )
  }

  /// `Bundle.load(path:)`, which fails if nothing is at `path`.
  pub fn load(path: &Path) -> Result<Self, Error> {
    Self::at(
      ffi::cz_bundle_load(&path.display().to_string()),
      format!("load the bundle at {}", path.display()),
    )
  }

  fn at(outcome: ffi::CzOutcome, action: String) -> Result<Self, Error> {
    platform::outcome(outcome, action).map(|outcome| Self {
      path: PathBuf::from(outcome.text()),
    })
  }

  /// `Bundle.path`.
  pub fn path(&self) -> &Path {
    &self.path
  }

  /// `Bundle.configPath`.
  pub fn config_path(&self) -> Result<PathBuf, Error> {
    platform::outcome(
      ffi::cz_bundle_config_path(&self.path.display().to_string()),
      "find a bundle's config",
    )
    .map(|outcome| PathBuf::from(outcome.text()))
  }

  /// `Bundle.rootfsPath`.
  pub fn rootfs_path(&self) -> Result<PathBuf, Error> {
    platform::outcome(
      ffi::cz_bundle_rootfs_path(&self.path.display().to_string()),
      "find a bundle's rootfs",
    )
    .map(|outcome| PathBuf::from(outcome.text()))
  }

  /// `Bundle.delete()`.
  pub fn delete(&self) -> Result<(), Error> {
    platform::outcome(
      ffi::cz_bundle_delete(&self.path.display().to_string()),
      format!("delete the bundle at {}", self.path.display()),
    )
    .map(drop)
  }

  /// `Bundle.loadConfig()`.
  pub fn load_config(&self) -> Result<Spec, Error> {
    platform::outcome(
      ffi::cz_bundle_load_config(&self.path.display().to_string()),
      format!("load the config of the bundle at {}", self.path.display()),
    )
    .map(|outcome| outcome.spec())
  }
}
