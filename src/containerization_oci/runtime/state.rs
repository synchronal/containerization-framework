//! A container's runtime state, from Swift's `State.swift`.

use std::collections::BTreeMap;

/// `seccompFdName`.
pub const SECCOMP_FD_NAME: &str = "seccompFd";

raw_values!(
  /// `ContainerState`.
  ContainerState {
    Creating = "creating",
    Created = "created",
    Running = "running",
    Stopped = "stopped",
  }
);

/// `State`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
  pub oci_version: String,
  pub id: String,
  pub status: ContainerState,
  pub pid: isize,
  pub bundle: String,
  pub annotations: Option<BTreeMap<String, String>>,
}

impl State {
  /// `State(version:id:status:pid:bundle:annotations:)`.
  pub fn new(
    version: impl Into<String>,
    id: impl Into<String>,
    status: ContainerState,
    pid: isize,
    bundle: impl Into<String>,
    annotations: Option<BTreeMap<String, String>>,
  ) -> Self {
    Self {
      oci_version: version.into(),
      id: id.into(),
      status,
      pid,
      bundle: bundle.into(),
      annotations,
    }
  }
}

/// `ContainerProcessState`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContainerProcessState {
  pub oci_version: String,
  pub fds: Vec<String>,
  pub pid: isize,
  pub metadata: String,
  pub state: State,
}

impl ContainerProcessState {
  /// `ContainerProcessState(version:fds:pid:metadata:state:)`.
  pub fn new(
    version: impl Into<String>,
    fds: Vec<String>,
    pid: isize,
    metadata: impl Into<String>,
    state: State,
  ) -> Self {
    Self {
      oci_version: version.into(),
      fds,
      pid,
      metadata: metadata.into(),
      state,
    }
  }
}
