//! The kernel and init image releases this crate boots by default.
//!
//! Pinned separately from each other and from the Swift package, so a mismatch
//! fails only at runtime; [`crate::Session::version`] names both.

/// The init image release this crate is pinned to: the initfs carrying
/// `vminitd`, the agent the library talks to over vsock.
///
/// Must match the `containerization` release in
/// this crate's `swift/Package.swift`: they share a protocol,
/// and a mismatch fails at runtime, not build time.
macro_rules! initfs_version {
  () => {
    "0.48.0"
  };
}

macro_rules! kernel_version {
  () => {
    "3.17.0"
  };
}

pub const INITFS_VERSION: &str = initfs_version!();
pub const INITFS_REFERENCE: &str = concat!("ghcr.io/apple/containerization/vminit:", initfs_version!());

/// The kernel `provision` fetches, and its path inside the archive.
///
/// Kata Containers' static build, the release Containerization's Makefile
/// pins. Downloaded, since no one publishes a kernel as an image.
pub const KERNEL_VERSION: &str = kernel_version!();

/// The archive [`KERNEL_VERSION`] is published in.
pub const KERNEL_URL: &str = concat!(
  "https://github.com/kata-containers/kata-containers/releases/download/",
  kernel_version!(),
  "/kata-static-",
  kernel_version!(),
  "-arm64.tar.xz"
);
/// The kernel's path inside [`KERNEL_URL`]'s archive.
pub const KERNEL_IN_ARCHIVE: &str = "opt/kata/share/kata-containers/vmlinux.container";
