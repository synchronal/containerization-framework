//! Rust bindings for Apple's [Containerization] framework: Linux containers,
//! each in its own lightweight VM, in-process and with no daemon.
//!
//! # Requirements
//!
//! macOS 26 on Apple silicon, with Xcode 26 to build. The Swift package is
//! compiled by this crate's build script, which resolves Containerization and
//! its dependencies from the network on a first build.
//!
//! **A binary using this crate must be codesigned with the
//! `com.apple.security.virtualization` entitlement**, or every [`Session::boot`]
//! fails saying so. A plain `cargo build` drops the signature, so re-sign after
//! each one; `containerization.entitlements` in this crate is the file to pass
//! to `codesign --entitlements`.
//!
//! Elsewhere this crate compiles, and every call that needs a VM fails on
//! first use with [`Error::Unavailable`], so a cross-platform workspace still
//! builds.
//!
//! # Shape
//!
//! [`Builder`] turns a [`BuildPlan`] into an image in the [`Store`].
//! [`Session`] boots a [`BootSpec`]'s container from one and runs processes
//! in it. A container belongs to the process that booted it and dies with it;
//! reaching one from elsewhere is the caller's to arrange.
//!
//! Configuration types in [`model`] mirror Containerization's, with the same
//! names and defaults.
//!
//! [Containerization]: https://github.com/apple/containerization

// `exec` passes a process's descriptors as scalars beside its configuration,
// and swift-bridge refuses an `allow` inside its module.
#![cfg_attr(target_os = "macos", allow(clippy::too_many_arguments))]

pub mod error;
pub mod model;
mod pins;
pub mod stdio;
mod store;

mod builder;
mod platform;
mod session;

#[cfg(target_os = "macos")]
mod bridge;

pub use crate::builder::Builder;
pub use crate::error::Error;
pub use crate::model::{BootSpec, BuildPlan, BuildStep, CachePolicy, Shell};
pub use crate::pins::{INITFS_REFERENCE, INITFS_VERSION, KERNEL_IN_ARCHIVE, KERNEL_URL, KERNEL_VERSION};
pub use crate::session::Session;
pub use crate::stdio::{Stdio, UNATTACHED, is_tty, lend};
pub use crate::store::{Store, StoreError};
