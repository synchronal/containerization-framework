//! Stand-ins for the bridge on every platform but macOS: the same functions,
//! each failing, so [`crate::Session`] and [`crate::Builder`] are written once
//! and a cross-platform workspace still compiles.
//!
//! Signatures match `bridge::ffi`'s, with the model types in place of their
//! `Rust`-prefixed aliases. [`super::checked`] words the failure.

use super::FAILED;
use crate::model;

pub(crate) fn czbridge_boot(
  _spec: model::BootSpec,
  _store_root: &str,
  _kernel_path: &str,
  _initfs_reference: &str,
  _initfs_path: &str,
) -> i32 {
  FAILED
}

pub(crate) fn czbridge_exec(
  _name: &str,
  _id: &str,
  _configuration: model::LinuxProcessConfiguration,
  _terminal: i32,
  _stdin: i32,
  _stdout: i32,
  _stderr: i32,
) -> i32 {
  FAILED
}

pub(crate) fn czbridge_build(
  _plan: model::BuildPlan,
  _store_root: &str,
  _kernel_path: &str,
  _initfs_reference: &str,
  _initfs_path: &str,
) -> i32 {
  FAILED
}

pub(crate) fn czbridge_provision(
  _store_root: &str,
  _kernel_path: &str,
  _kernel_url: &str,
  _kernel_in_archive: &str,
  _initfs_reference: &str,
  _initfs_path: &str,
) -> i32 {
  FAILED
}

pub(crate) fn czbridge_resize(_id: &str, _terminal: i32) -> i32 {
  FAILED
}

/// Nothing can be running, since nothing can boot.
pub(crate) fn czbridge_is_running(_name: &str) -> bool {
  false
}

pub(crate) fn czbridge_is_unpacked(_store_root: &str, _image_reference: &str) -> i32 {
  FAILED
}
