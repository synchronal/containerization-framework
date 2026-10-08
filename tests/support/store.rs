//! The suite's image store, the kernel and init image its VMs boot, and the
//! image its containers run.
//!
//! A pull is slow, so the store is filled once and kept between runs rather
//! than thrown away per test. The kernel is downloaded into it before the
//! tests run, by `bin/dev/prepare-integration`.

use super::lock::Lock;
use containerization_framework as cfw;
use std::path::PathBuf;

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

const IMAGE_LOCK: &str = ".image.lock";
const INITFS_LOCK: &str = ".initfs.lock";

pub fn root() -> PathBuf {
  PathBuf::from(std::env::var("HOME").expect("a test runs with a home")).join(STORE_IN_HOME)
}

/// The suite's store, made if missing.
pub fn image_store() -> cfw::containerization::image::ImageStore {
  std::fs::create_dir_all(root()).expect("the store directory should be creatable");

  cfw::containerization::image::ImageStore::new(&root()).expect("the image store should open")
}

/// Where `ImageStore(path:)` keeps its `LocalContentStore`.
pub fn content_store_path() -> PathBuf {
  root().join("content")
}

/// The suite's content store, which [`image_store`] keeps its blobs in.
pub fn content_store() -> cfw::containerization_oci::content::LocalContentStore {
  cfw::containerization_oci::content::LocalContentStore::new(&content_store_path())
    .expect("the suite's content store should open")
}

/// [`IMAGE`], pulled on a first run. The lock keeps concurrent first runs from
/// pulling it at once.
pub fn image(store: &cfw::containerization::image::ImageStore) -> cfw::containerization::image::Image {
  if let Ok(image) = store.get(IMAGE, false) {
    return image;
  }

  let _lock = Lock::take(root().join(IMAGE_LOCK));

  store
    .get(IMAGE, true)
    .unwrap_or_else(|error| panic!("{IMAGE} should pull: {error}"))
}

/// [`IMAGE`], unpacked by Rust into an ext4 file at `at`.
pub fn unpack(at: &std::path::Path) -> cfw::containerization::container::Mount {
  let store = image_store();
  let platform = cfw::containerization_oci::image::Platform::current().expect("the current platform");

  cfw::containerization::image::Ext4Unpacker::new(super::TEST_ROOTFS_SIZE_IN_BYTES, None)
    .unpack(&image(&store), &platform, at, None)
    .unwrap_or_else(|error| panic!("{IMAGE} should unpack to {}: {error}", at.display()))
}

/// The kernel `bin/dev/prepare-integration` downloaded, which nextest runs
/// before these tests.
pub fn kernel() -> cfw::containerization::vm::Kernel {
  let path = PathBuf::from(
    std::env::var("CFW_TEST_KERNEL")
      .expect("CFW_TEST_KERNEL names the kernel; run the suite with bin/dev/test-integration"),
  );
  assert!(path.is_file(), "the kernel at {} should be a file", path.display());

  cfw::containerization::vm::Kernel::new(path, cfw::containerization::vm::SystemPlatform::LINUX_ARM)
}

/// A manager booting [`kernel`] and [`INITFS_REFERENCE`]. The first one unpacks
/// the init image into the store, so the lock keeps two from doing it at once.
pub fn manager(store: &cfw::containerization::image::ImageStore) -> cfw::containerization::container::ContainerManager {
  manager_with(store, Default::default())
}

/// The same, made with `options`.
pub fn manager_with(
  store: &cfw::containerization::image::ImageStore,
  options: cfw::containerization::container::container_manager::ManagerOptions,
) -> cfw::containerization::container::ContainerManager {
  let kernel = kernel();
  let _lock = initfs_lock();

  cfw::containerization::container::ContainerManager::with_initfs_reference(&kernel, INITFS_REFERENCE, store, options)
    .unwrap_or_else(|error| panic!("a manager booting {INITFS_REFERENCE} should be made: {error}"))
}

/// A VM manager booting [`kernel`] and the init block a container manager
/// unpacks into the store from [`INITFS_REFERENCE`].
pub fn vmm() -> cfw::containerization::vm::VzVirtualMachineManager {
  // Unpacks the init block, if no manager has yet.
  drop(manager(&image_store()));

  let initfs = cfw::containerization::container::Mount::block(
    "ext4",
    root().join("initfs.ext4").display().to_string(),
    "/",
    &["ro"],
    &[],
  );

  cfw::containerization::vm::VzVirtualMachineManager::new(&kernel(), &initfs, Default::default())
    .unwrap_or_else(|error| panic!("a VM manager should be made: {error}"))
}

/// The lock [`manager`] takes, for a test making a manager of its own.
pub fn initfs_lock() -> Lock {
  Lock::take(root().join(INITFS_LOCK))
}
