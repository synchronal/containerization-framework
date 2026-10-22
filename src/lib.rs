#![cfg_attr(feature = "strict", deny(warnings))]

//! Rust bindings for Apple's [Containerization] framework, which runs each
//! Linux container in its own lightweight VM, in-process and with no daemon.
//!
//! # Shape
//!
//! The Rust API mirrors Containerization's Swift API as closely as it can.
//! Modules are named after the Swift modules, types after the Swift types, and
//! methods after the Swift methods, in snake case. Within a module, the types
//! are grouped by what they work on, so `ImageStore` is in
//! `containerization::image` and `LinuxContainer` is in
//! `containerization::container`. A Swift type nested in another, like
//! `LinuxContainer.Configuration`, is in a module named after its parent, in
//! the parent's group: `container::linux_container::Configuration`. When the
//! parent has the same name as its group, the parent and its nested types are
//! in the group's module, as `Image` and `Image.Description` are in
//! `containerization::image`.
//!
//! Swift's `async` methods block until they finish, and errors they throw are
//! returned as [`Error`].
//!
//! ```no_run
//! use containerization_framework as cfw;
//! use cfw::containerization as cz;
//! # fn main() -> Result<(), cfw::Error> {
//! let store = cz::image::ImageStore::new("/path/to/store".as_ref())?;
//! let kernel = cz::vm::Kernel::new("/path/to/vmlinux", cz::vm::SystemPlatform::LINUX_ARM);
//! let mut manager = cz::container::ContainerManager::with_initfs_reference(
//!   &kernel,
//!   cfw::versions::INIT_IMAGE,
//!   &store,
//!   Default::default(),
//! )?;
//! let image = store.get("docker.io/library/alpine:3", true)?;
//! let options = cz::container::container_manager::CreateOptions {
//!   networking: false,
//!   ..Default::default()
//! };
//! let container = manager.create("demo", &image, options, |config| {
//!   config.process.arguments = vec!["/bin/sleep".into(), "infinity".into()];
//!   Ok(())
//! })?;
//! container.create()?;
//! container.start()?;
//! let process = container.exec("ls", cz::process::LinuxProcessConfiguration::new(&["/bin/ls", "/"]))?;
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
//! rebuild drops the signature, so after every build, sign the binary again
//! and pass this crate's `containerization.entitlements` to
//! `codesign --entitlements`.
//!
//! Elsewhere this crate compiles, so a cross-platform workspace builds, but
//! every call that would reach Swift fails with [`Error::Unavailable`]. Values
//! built purely in Rust work as usual.
//!
//! [Containerization]: https://github.com/apple/containerization

#![warn(missing_debug_implementations)]

/// A Swift enum backed by strings: Rust's cases, each with its `rawValue`.
/// `CaseIterable` before the name adds `ALL_CASES`, for an enum Swift
/// declares `CaseIterable`.
macro_rules! raw_values {
  ($(#[$meta:meta])* CaseIterable $name:ident { $($case:ident = $raw:literal),* $(,)? }) => {
    raw_values!(@enum $(#[$meta])* $name { $($case = $raw),* });

    impl $name {
      /// `allCases`.
      pub const ALL_CASES: &[Self] = &[$(Self::$case),*];
    }
  };
  ($(#[$meta:meta])* $name:ident { $($case:ident = $raw:literal),* $(,)? }) => {
    raw_values!(@enum $(#[$meta])* $name { $($case = $raw),* });

    impl $name {
      /// Every case, in the order the bridge lists Swift's raw values.
      #[cfg(all(test, target_os = "macos"))]
      pub(crate) const ALL: &[Self] = &[$(Self::$case),*];
    }
  };
  (@enum $(#[$meta:meta])* $name:ident { $($case:ident = $raw:literal),* $(,)? }) => {
    $(#[$meta])*
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub enum $name {
      $(#[doc = concat!("`", $raw, "`.")] $case),*
    }

    impl $name {
      /// `rawValue`.
      pub fn raw_value(self) -> &'static str {
        match self {
          $(Self::$case => $raw),*
        }
      }

      /// `init?(rawValue:)`.
      pub fn from_raw_value(raw_value: &str) -> Option<Self> {
        match raw_value {
          $($raw => Some(Self::$case),)*
          _ => None,
        }
      }

      /// A raw value from Swift. It panics if Swift has a case Rust lacks.
      /// Not every enum uses it.
      #[allow(dead_code)]
      pub(crate) fn from_swift(raw_value: &str) -> Self {
        Self::from_raw_value(raw_value)
          .unwrap_or_else(|| panic!("Swift's {} has a case Rust lacks: {raw_value:?}", stringify!($name)))
      }
    }

    /// The `rawValue`, as [`Self::raw_value`] returns it.
    impl AsRef<str> for $name {
      fn as_ref(&self) -> &str {
        self.raw_value()
      }
    }

    /// `init?(rawValue:)`, as [`Self::from_raw_value`] is, failing for a raw
    /// value Swift lacks.
    impl std::str::FromStr for $name {
      type Err = $crate::Error;

      fn from_str(raw_value: &str) -> Result<Self, Self::Err> {
        Self::from_raw_value(raw_value).ok_or_else(|| {
          $crate::Error::failed(
            concat!("parse a raw value of ", stringify!($name)),
            format!("{raw_value:?} is not one of Swift's cases"),
          )
        })
      }
    }
  };
}

/// A `Debug` that prints only the type's name, for a handle on a Swift value
/// whose fields Rust can't read. A lifetime follows the name of a type that
/// borrows, as in `opaque_debug!(ArchiveEntryReader<'_>)`.
macro_rules! opaque_debug {
  ($(#[$meta:meta])* $name:ident $(<$lifetime:lifetime>)?) => {
    $(#[$meta])*
    impl std::fmt::Debug for $name $(<$lifetime>)? {
      fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct(stringify!($name)).finish_non_exhaustive()
      }
    }
  };
}

/// The two `From`s between a fieldless Rust enum and the bridge's enum for it,
/// as in `same_variants!(Mode, ffi::VmnetMode { Shared, Host, Bridged })`. A
/// variant the bridge names differently is written `Rust => Bridge`.
macro_rules! same_variants {
  ($rust:ty, $bridge:ty { $($variant:ident $(=> $renamed:ident)?),* $(,)? }) => {
    impl From<$rust> for $bridge {
      fn from(value: $rust) -> Self {
        type Rust = $rust;

        match value {
          $(Rust::$variant => same_variants!(@bridge $variant $($renamed)?)),*
        }
      }
    }

    impl From<$bridge> for $rust {
      fn from(value: $bridge) -> Self {
        type Bridge = $bridge;

        match value {
          $(same_variants!(@pattern $variant $($renamed)?) => Self::$variant),*
        }
      }
    }
  };
  (@bridge $variant:ident) => { Self::$variant };
  (@bridge $variant:ident $renamed:ident) => { Self::$renamed };
  (@pattern $variant:ident) => { Bridge::$variant };
  (@pattern $variant:ident $renamed:ident) => { Bridge::$renamed };
}

pub mod cloud_hypervisor;
pub mod containerization;
pub mod containerization_archive;
pub mod containerization_error;
pub mod containerization_ext4;
pub mod containerization_extras;
pub mod containerization_io;
pub mod containerization_oci;
pub mod containerization_os;
pub mod versions;

mod error;
mod platform;

// The bridge's functions take Swift's arguments, however many there are.
#[cfg(target_os = "macos")]
#[allow(clippy::too_many_arguments)]
mod bridge;

pub use crate::error::Error;

// Compiles the README's example as a doctest, so it can't drift from the API.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
