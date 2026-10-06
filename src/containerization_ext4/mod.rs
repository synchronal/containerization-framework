//! Containerization's `ContainerizationEXT4` module.
//!
//! Most of its types are nested in Swift's `EXT4` namespace, so they are in
//! [`ext4`].

pub mod ext4;
pub(crate) mod file_timestamps;

pub use self::file_timestamps::FileTimestamps;
