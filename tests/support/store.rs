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
pub const INITFS_REFERENCE: &str = cfw::versions::INIT_IMAGE;

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

  cfw::containerization::image::EXT4Unpacker::new(super::TEST_ROOTFS_SIZE_IN_BYTES, None)
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

/// [`INITFS_REFERENCE`]'s init block, unpacked into the store on a first run.
///
/// Swift unpacks a reference into `initfs.ext4` and reuses that file for any
/// reference, so after an update the suite would boot the old guest agent.
/// Naming the file after the reference makes a new release unpack a new
/// block. The lock keeps two tests from unpacking it at once.
pub fn initfs() -> cfw::containerization::container::Mount {
  let tag = INITFS_REFERENCE
    .rsplit_once(':')
    .map_or(INITFS_REFERENCE, |(_, tag)| tag);
  let path = root().join(format!("initfs-{tag}.ext4"));
  let _lock = initfs_lock();

  if !path.is_file() {
    // Unpack beside the block and then rename it, so a run that dies midway
    // leaves no partial block under the real name.
    let partial = path.with_extension("ext4.partial");
    let _ = std::fs::remove_file(&partial);
    image_store()
      .get_init_image(INITFS_REFERENCE, Default::default())
      .unwrap_or_else(|error| panic!("{INITFS_REFERENCE} should pull: {error}"))
      .init_block(&partial, cfw::containerization::vm::SystemPlatform::LINUX_ARM)
      .unwrap_or_else(|error| panic!("{INITFS_REFERENCE} should unpack to {}: {error}", partial.display()));
    std::fs::rename(&partial, &path).expect("the unpacked init block should be renamed into place");
  }

  cfw::containerization::container::Mount::block("ext4", path.display().to_string(), "/", &["ro"], &[])
}

/// A manager booting [`kernel`] and [`initfs`].
pub fn manager(store: &cfw::containerization::image::ImageStore) -> cfw::containerization::container::ContainerManager {
  manager_with(store, Default::default())
}

/// The same, made with `options`.
pub fn manager_with(
  store: &cfw::containerization::image::ImageStore,
  options: cfw::containerization::container::container_manager::ContainerManagerOptions,
) -> cfw::containerization::container::ContainerManager {
  cfw::containerization::container::ContainerManager::new(&kernel(), &initfs(), store, options)
    .unwrap_or_else(|error| panic!("a manager booting {INITFS_REFERENCE} should be made: {error}"))
}

/// A VM manager booting [`kernel`] and [`initfs`].
pub fn vmm() -> cfw::containerization::vm::VZVirtualMachineManager {
  cfw::containerization::vm::VZVirtualMachineManager::new(&kernel(), &initfs(), Default::default())
    .unwrap_or_else(|error| panic!("a VM manager should be made: {error}"))
}

/// The lock [`initfs`] takes, for a test making a manager of its own.
pub fn initfs_lock() -> Lock {
  Lock::take(root().join(INITFS_LOCK))
}
