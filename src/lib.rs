//! Rust bindings for Apple's [Containerization] framework: Linux containers,
//! each in its own lightweight VM, in-process and with no daemon.
//!
//! # Shape
//!
//! The Rust API mirrors Containerization's Swift API, as much as possible.
//! Modules are named after the Swift modules, types after the Swift types, and
//! methods after their Swift methods, written in Rust's snake case. A Swift type
//! nested in another, like `LinuxContainer.Configuration`, is found in a module
//! named after its parent: `linux_container::Configuration`.
//!
//! Swift's `async` methods block until they finish, and errors they throw are
//! returned as [`Error`].
//!
//! ```no_run
//! use containerization_framework as cfw;
//! use cfw::containerization as cz;
//! # fn main() -> Result<(), cfw::Error> {
//! let store = cz::ImageStore::new("/path/to/store".as_ref())?;
//! let kernel = cz::Kernel::new("/path/to/vmlinux", cz::SystemPlatform::LINUX_ARM);
//! let mut manager = cz::ContainerManager::with_initfs_reference(
//!   &kernel,
//!   "ghcr.io/apple/containerization/vminit:0.48.0",
//!   &store,
//!   false,
//!   false,
//! )?;
//! let image = store.get("docker.io/library/alpine:3", true)?;
//! let options = cz::container_manager::CreateOptions {
//!   networking: false,
//!   ..Default::default()
//! };
//! let container = manager.create("demo", &image, options, |config| {
//!   config.process.arguments = vec!["/bin/sleep".into(), "infinity".into()];
//! })?;
//! container.create()?;
//! container.start()?;
//! let process = container.exec("ls", cz::LinuxProcessConfiguration::new(&["/bin/ls", "/"]))?;
//! process.start()?;
//! let status = process.wait(None)?;
//! process.delete()?;
//! container.stop()?;
//! manager.delete("demo")?;
//! # let _ = status;
//! # Ok(())
//! # }
//! ```
//!
//! # Requirements
//!
//! macOS 26 on Apple silicon, with Xcode 26 to build. The Swift package is
//! compiled by this crate's build script, which resolves Containerization and
//! its dependencies from the network on a first build.
//!
//! **A binary using this crate must be codesigned with the
//! `com.apple.security.virtualization` entitlement** to create a container. A
//! plain `cargo build` drops the signature, so re-sign after each one;
//! `containerization.entitlements` in this crate is the file to pass to
//! `codesign --entitlements`.
//!
//! Elsewhere this crate compiles, and every constructor fails with
//! [`Error::Unavailable`], so a cross-platform workspace still builds.
//!
//! [Containerization]: https://github.com/apple/containerization

pub mod containerization;
pub mod containerization_oci;
pub mod containerization_os;
pub mod error;

mod platform;

#[cfg(target_os = "macos")]
mod bridge;

pub use crate::error::Error;
