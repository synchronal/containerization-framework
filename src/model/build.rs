//! An image to build: its base, its steps, and how the build caches them.

use super::strings;
use crate::model;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

/// One build step: a named `RUN`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildStep {
  /// The step's label in the build log. Not part of its cache key.
  pub name: String,
  pub script: String,
  /// The guest user, as the image names it. `None` is root.
  pub user: Option<String>,
  /// The rootfs once this step has run. Opaque here: what invalidates a build
  /// is the caller's policy, so the caller derives these and this crate only
  /// stores and compares them. Salted with the base's digest and the rootfs
  /// ceiling, which only a build knows.
  pub cache_key: String,
}

/// What runs a step's script, with the script appended as the final argument.
///
/// The default is `bash -euo pipefail -c`, not `sh -c`: a silent mid-step
/// failure would be baked into the image. It requires bash in the base; pass
/// `["/bin/sh", "-ec"]` for a base without one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Shell(pub Vec<String>);

impl Default for Shell {
  fn default() -> Self {
    Self(strings(&["/bin/bash", "-euo", "pipefail", "-c"]))
  }
}

/// How a build treats its rootfs snapshots.
///
/// Snapshots are written whether or not they are read, so a build with
/// `restore` off still leaves the next one something to resume from.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CachePolicy {
  /// Resume from the deepest matching snapshot. Off is `--no-cache`.
  pub restore: bool,
  /// Snapshots kept, most recently used first.
  pub keep: usize,
  /// Anything unused this long goes regardless.
  pub keep_for: Duration,
}

impl Default for CachePolicy {
  fn default() -> Self {
    Self {
      restore: true,
      keep: 24,
      keep_for: Duration::from_secs(14 * 24 * 60 * 60),
    }
  }
}

/// An image to build: a base, steps, and what the result runs as.
///
/// The builder is an ordinary container that resolves DNS through its
/// interface's gateway.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildPlan {
  /// The builder container's id, and its directory in the store, removed
  /// before and after the build. Distinct per concurrent build.
  pub name: String,
  /// Registry-qualified. Every step runs on top of it.
  pub base: String,
  /// What the finished image is registered as.
  pub tag: String,
  /// The builder's own limits and VM, not the image's.
  pub cpus: u32,
  pub memory_in_bytes: u64,
  pub vm: model::VmResources,
  /// Shared into every step: what a step reads instead of `COPY`.
  pub mounts: Vec<model::Mount>,
  pub steps: Vec<BuildStep>,
  /// `NAME=VALUE`, visible to every step and written into the image config,
  /// like `ENV`. The base's own are kept unless a name here overrides one.
  pub environment: Vec<String>,
  /// Written into the image config as its OCI labels. The base's carry over;
  /// these win on a key both set.
  pub labels: BTreeMap<String, String>,
  /// The user and directory the finished image runs as. `None` keeps the base's.
  pub user: Option<String>,
  pub workdir: Option<PathBuf>,
  pub interface: model::NatInterface,
  /// The rootfs before any step. See [`BuildStep::cache_key`].
  pub base_key: String,
  /// Ceiling for the builder's rootfs. Every step's result must fit.
  pub rootfs_size_in_bytes: u64,
  pub cache: CachePolicy,
  /// What runs each step's script.
  pub shell: Shell,
  /// The builder's first process, which must outlive every step. The base's
  /// own `Cmd` would exit and take the container with it.
  pub keepalive: Vec<String>,
  /// Delete unreferenced blobs and the unpacked rootfs of removed images once
  /// the build has stored its result. Off leaves the previous build's layer,
  /// which is usually the largest thing in the store.
  pub reclaim: bool,
}

impl BuildPlan {
  /// Holds the builder open while steps run as `exec`s.
  pub fn default_keepalive() -> Vec<String> {
    strings(&["/bin/sh", "-c", "while :; do sleep 86400; done"])
  }

  /// Everything else at Containerization's defaults.
  pub fn new(
    name: impl Into<String>,
    base: impl Into<String>,
    tag: impl Into<String>,
    interface: model::NatInterface,
    base_key: impl Into<String>,
  ) -> Self {
    let container = model::LinuxContainerConfiguration::default();

    Self {
      name: name.into(),
      base: base.into(),
      tag: tag.into(),
      cpus: container.cpus,
      memory_in_bytes: container.memory_in_bytes,
      vm: model::VmResources::default(),
      mounts: Vec::new(),
      steps: Vec::new(),
      environment: Vec::new(),
      labels: BTreeMap::new(),
      user: None,
      workdir: None,
      interface,
      base_key: base_key.into(),
      rootfs_size_in_bytes: model::BootSpec::DEFAULT_ROOTFS_SIZE_IN_BYTES,
      cache: CachePolicy::default(),
      shell: Shell::default(),
      keepalive: Self::default_keepalive(),
      reclaim: true,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::model::GIB;

  #[test]
  fn builds_with_bash_failing_a_step_and_reclaims_after() {
    let plan = BuildPlan::new(
      "builder",
      "docker.io/library/debian:stable-slim",
      "example/base:latest",
      model::NatInterface::new("192.168.64.7/24", "192.168.64.1"),
      "basekey",
    );

    assert_eq!(plan.shell.0, ["/bin/bash", "-euo", "pipefail", "-c"]);
    assert!(plan.cache.restore);
    assert_eq!(plan.cache.keep, 24);
    assert_eq!(plan.cache.keep_for, Duration::from_secs(14 * 24 * 60 * 60));
    assert!(plan.reclaim);
    assert_eq!(plan.rootfs_size_in_bytes, 8 * GIB);
  }
}
