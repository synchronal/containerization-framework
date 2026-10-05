//! The suite's image store, the kernel and init image its VMs boot, and the
//! image its containers run.
//!
//! A kernel download and a pull are slow, so the store is filled once and kept
//! between runs rather than thrown away per test.

use super::lock::Lock;
use containerization_framework as cfw;
use std::path::PathBuf;
use std::process::Command;

/// Kept apart from any store a developer's own tools use: these tests write
/// containers into it.
const STORE_IN_HOME: &str = ".cache/containerization-framework-tests";

/// Registry-qualified: nothing expands a short reference. Alpine, because
/// every container unpacks it, and what these tests run needs only busybox.
pub const IMAGE: &str = "docker.io/library/alpine:3";

/// The init image matching the Containerization release in
/// `swift/Package.swift`: they share a protocol, and a mismatch fails only at
/// runtime.
pub const INITFS_REFERENCE: &str = "ghcr.io/apple/containerization/vminit:0.48.0";

/// The kernel Containerization's Makefile pins: Kata Containers' static build,
/// since no one publishes a kernel as an image.
const KERNEL_VERSION: &str = "3.17.0";
const KERNEL_IN_ARCHIVE: &str = "opt/kata/share/kata-containers/vmlinux.container";

const KERNEL_LOCK: &str = ".kernel.lock";
const IMAGE_LOCK: &str = ".image.lock";
const INITFS_LOCK: &str = ".initfs.lock";

pub fn root() -> PathBuf {
  PathBuf::from(std::env::var("HOME").expect("a test runs with a home")).join(STORE_IN_HOME)
}

/// The suite's store, made if missing.
pub fn image_store() -> cfw::containerization::ImageStore {
  std::fs::create_dir_all(root()).expect("the store directory should be creatable");

  cfw::containerization::ImageStore::new(&root()).expect("the image store should open")
}

/// Where `ImageStore(path:)` keeps its `LocalContentStore`.
pub fn content_store_path() -> PathBuf {
  root().join("content")
}

/// [`IMAGE`], pulled on a first run. The lock keeps concurrent first runs from
/// pulling it at once.
pub fn image(store: &cfw::containerization::ImageStore) -> cfw::containerization::Image {
  if let Ok(image) = store.get(IMAGE, false) {
    return image;
  }

  let _lock = Lock::take(root().join(IMAGE_LOCK));

  store
    .get(IMAGE, true)
    .unwrap_or_else(|error| panic!("{IMAGE} should pull: {error}"))
}

/// [`IMAGE`], unpacked by Rust into an ext4 file at `at`.
pub fn unpack(at: &std::path::Path) -> cfw::containerization::Mount {
  let store = image_store();
  let platform = cfw::containerization_oci::Platform::current().expect("the current platform");

  cfw::containerization::Ext4Unpacker::new(super::TEST_ROOTFS_SIZE_IN_BYTES, None)
    .unpack(&image(&store), &platform, at, None)
    .unwrap_or_else(|error| panic!("{IMAGE} should unpack to {}: {error}", at.display()))
}

/// The kernel, downloaded on a first run.
pub fn kernel() -> cfw::containerization::Kernel {
  let path = root()
    .join("kernels")
    .join(format!("vmlinux-{KERNEL_VERSION}"));

  if !path.is_file() {
    let _lock = Lock::take(root().join(KERNEL_LOCK));

    if !path.is_file() {
      download_kernel(&path);
    }
  }

  cfw::containerization::Kernel::new(path, cfw::containerization::SystemPlatform::LINUX_ARM)
}

/// A manager booting [`kernel`] and [`INITFS_REFERENCE`]. The first one unpacks
/// the init image into the store, so the lock keeps two from doing it at once.
pub fn manager(store: &cfw::containerization::ImageStore) -> cfw::containerization::ContainerManager {
  let kernel = kernel();
  let _lock = Lock::take(root().join(INITFS_LOCK));

  cfw::containerization::ContainerManager::with_initfs_reference(&kernel, INITFS_REFERENCE, store, false, false)
    .unwrap_or_else(|error| panic!("a manager booting {INITFS_REFERENCE} should be made: {error}"))
}

/// Fetches Kata's release and extracts the kernel to `path`, by way of a
/// partial file so an interrupted download is never found.
fn download_kernel(path: &std::path::Path) {
  let directory = path.parent().expect("the kernel has a directory");
  std::fs::create_dir_all(directory).expect("the kernel directory should be creatable");

  let url = format!(
    "https://github.com/kata-containers/kata-containers/releases/download/{KERNEL_VERSION}/kata-static-{KERNEL_VERSION}-arm64.tar.xz"
  );
  let staging = tempfile::tempdir_in(directory).expect("a staging directory");
  let script = format!(
    "curl --fail --location --silent --show-error {url} | tar -xJf - -C {} {KERNEL_IN_ARCHIVE}",
    staging.path().display()
  );
  let status = Command::new("/bin/sh")
    .args(["-c", &script])
    .status()
    .expect("curl and tar should run");

  assert!(status.success(), "the kernel should download from {url}");

  std::fs::rename(staging.path().join(KERNEL_IN_ARCHIVE), path).expect("the kernel should move into place");
}
