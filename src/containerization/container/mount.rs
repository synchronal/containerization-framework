//! `Mount`, and its nested `Mount.RuntimeOptions`.

use crate::cloud_hypervisor;
use crate::containerization::strings;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `Mount.RuntimeOptions`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeOptions {
  Virtioblk(Vec<String>),
  Virtiofs(Vec<String>),
  Shared,
  Any(Vec<String>),
}

/// `Mount`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mount {
  pub r#type: String,
  pub source: String,
  pub destination: String,
  pub options: Vec<String>,
  pub runtime_options: RuntimeOptions,
}

impl Mount {
  /// `Mount.share(source:destination:options:runtimeOptions:)`. Swift's
  /// `options` and `runtimeOptions` default to `[]`.
  pub fn share(
    source: impl Into<String>,
    destination: impl Into<String>,
    options: &[&str],
    runtime_options: &[&str],
  ) -> Self {
    Self {
      r#type: "virtiofs".to_string(),
      source: source.into(),
      destination: destination.into(),
      options: strings(options),
      runtime_options: RuntimeOptions::Virtiofs(strings(runtime_options)),
    }
  }

  /// `Mount.block(format:source:destination:options:runtimeOptions:)`.
  /// Swift's `options` and `runtimeOptions` default to `[]`.
  pub fn block(
    format: impl Into<String>,
    source: impl Into<String>,
    destination: impl Into<String>,
    options: &[&str],
    runtime_options: &[&str],
  ) -> Self {
    Self {
      r#type: format.into(),
      source: source.into(),
      destination: destination.into(),
      options: strings(options),
      runtime_options: RuntimeOptions::Virtioblk(strings(runtime_options)),
    }
  }

  /// `Mount.any(type:source:destination:options:runtimeOptions:)`. Swift's
  /// `options` and `runtimeOptions` default to `[]`.
  pub fn any(
    r#type: impl Into<String>,
    source: impl Into<String>,
    destination: impl Into<String>,
    options: &[&str],
    runtime_options: &[&str],
  ) -> Self {
    Self {
      r#type: r#type.into(),
      source: source.into(),
      destination: destination.into(),
      options: strings(options),
      runtime_options: RuntimeOptions::Any(strings(runtime_options)),
    }
  }

  /// `Mount.sharedMount(name:destination:options:)`. Swift's `options`
  /// defaults to `[]`.
  pub fn shared_mount(name: impl Into<String>, destination: impl Into<String>, options: &[&str]) -> Self {
    Self {
      r#type: "none".to_string(),
      source: name.into(),
      destination: destination.into(),
      options: strings(options),
      runtime_options: RuntimeOptions::Shared,
    }
  }

  /// `Mount.clone(to:)`, which copies the source to `to` and returns this
  /// mount with `to` as its source. The name `clone` is taken by `Clone`.
  pub fn clone_to(&self, to: &str) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_mount_clone(self.clone(), to),
      format!("copy {} to {to}", self.source),
    )
    .map(|outcome| outcome.mount())
  }

  /// `Mount.isBlock`.
  pub fn is_block(&self) -> bool {
    matches!(self.runtime_options, RuntimeOptions::Virtioblk(_))
  }

  /// `Mount.chDiskConfig(id:)`: this mount as a cloud-hypervisor virtio-blk
  /// disk with the device id `id`, or `None` if it isn't a block mount. The
  /// disk is raw, read-only when the options hold `ro`, and opened with
  /// `O_DIRECT` when the runtime options hold `directio`.
  pub fn ch_disk_config(&self, id: &str) -> Result<Option<cloud_hypervisor::device::DiskConfig>, Error> {
    let outcome = platform::outcome(
      ffi::cz_mount_ch_disk_config(self.clone(), id),
      format!("describe {} as a cloud-hypervisor disk", self.source),
    )?;

    Ok(outcome.is_some().then(|| outcome.disk_config()))
  }

  /// `Mount.tagHash`, which hashes the source path after resolving its
  /// symlinks. It doesn't read the contents.
  pub fn tag_hash(&self) -> Result<String, Error> {
    platform::outcome(ffi::cz_mount_tag_hash(self.clone()), format!("hash {}", self.source))
      .map(|outcome| outcome.text())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn builds_mounts_of_each_kind() {
    let share = Mount::share("/Users/user/workspace", "/workspace", &["ro"], &[]);
    assert_eq!(share.r#type, "virtiofs");
    assert_eq!(share.options, ["ro"]);
    assert_eq!(share.runtime_options, RuntimeOptions::Virtiofs(vec![]));

    assert_eq!(
      Mount::block("ext4", "/images/data.ext4", "/data", &[], &["vda"]).runtime_options,
      RuntimeOptions::Virtioblk(vec!["vda".into()])
    );
    assert_eq!(
      Mount::any("tmpfs", "tmpfs", "/scratch", &[], &[]).runtime_options,
      RuntimeOptions::Any(vec![])
    );
  }
}
