//! `Mount`, and its nested `Mount.RuntimeOptions`.

use super::strings;

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
  /// `Mount.share(source:destination:options:runtimeOptions:)`.
  pub fn share(source: impl Into<String>, destination: impl Into<String>, options: &[&str]) -> Self {
    Self {
      r#type: "virtiofs".to_string(),
      source: source.into(),
      destination: destination.into(),
      options: strings(options),
      runtime_options: RuntimeOptions::Virtiofs(Vec::new()),
    }
  }

  /// `Mount.block(format:source:destination:options:runtimeOptions:)`.
  pub fn block(
    format: impl Into<String>,
    source: impl Into<String>,
    destination: impl Into<String>,
    options: &[&str],
  ) -> Self {
    Self {
      r#type: format.into(),
      source: source.into(),
      destination: destination.into(),
      options: strings(options),
      runtime_options: RuntimeOptions::Virtioblk(Vec::new()),
    }
  }

  /// `Mount.any(type:source:destination:options:runtimeOptions:)`.
  pub fn any(
    r#type: impl Into<String>,
    source: impl Into<String>,
    destination: impl Into<String>,
    options: &[&str],
  ) -> Self {
    Self {
      r#type: r#type.into(),
      source: source.into(),
      destination: destination.into(),
      options: strings(options),
      runtime_options: RuntimeOptions::Any(Vec::new()),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn builds_mounts_of_each_kind() {
    let share = Mount::share("/Users/user/workspace", "/workspace", &["ro"]);
    assert_eq!(share.r#type, "virtiofs");
    assert_eq!(share.options, ["ro"]);
    assert_eq!(share.runtime_options, RuntimeOptions::Virtiofs(vec![]));

    assert_eq!(
      Mount::block("ext4", "/images/data.ext4", "/data", &[]).runtime_options,
      RuntimeOptions::Virtioblk(vec![])
    );
    assert_eq!(
      Mount::any("tmpfs", "tmpfs", "/scratch", &[]).runtime_options,
      RuntimeOptions::Any(vec![])
    );
  }
}
