//! The suite's store, and the image built into it.
//!
//! Provisioning downloads a kernel and building pulls a base, so the store is
//! filled once and kept between runs rather than thrown away per test.

use super::lock::Lock;
use super::network;
use containerization_framework as cfw;
use std::path::PathBuf;

/// Kept apart from any store a developer's own tools use: these tests write
/// containers into it.
const STORE_IN_HOME: &str = ".cache/containerization-framework-tests";

/// Registry-qualified: nothing expands a short reference.
///
/// Alpine rather than a Debian: every boot clones the unpacked rootfs, so the
/// base's size is paid per container. What these tests assert — exit codes,
/// descriptors, a file a step wrote — needs a shell and busybox, not a distro.
pub const BASE_IMAGE: &str = "docker.io/library/alpine:3";
/// A local tag, so nothing tries to pull it. Named for the base, so a change of
/// base is a different image rather than a stale one a store still holds.
pub const TEST_IMAGE: &str = "containerization-framework/test-base:alpine3";

/// Written by the image's one step, read back from a container booted from it.
pub const MARKER_PATH: &str = "/etc/containerization-framework-test";
pub const MARKER: &str = "built-by-the-integration-suite";

const PROVISION_LOCK: &str = ".provision.lock";
const IMAGE_LOCK: &str = ".image.lock";

pub fn home() -> PathBuf {
  PathBuf::from(std::env::var("HOME").expect("a test runs with a home"))
}

/// The suite's store, named but not touched. Kernel and init image paths carry
/// their versions, so a pin change fetches new ones.
pub fn store() -> cfw::Store {
  let root = home().join(STORE_IN_HOME);

  cfw::Store::at(
    &root,
    root
      .join("kernels")
      .join(format!("vmlinux-{}", cfw::KERNEL_VERSION)),
    cfw::INITFS_REFERENCE,
    root
      .join("initfs")
      .join(format!("vminit-{}.ext4", cfw::INITFS_VERSION)),
  )
}

/// The same store with a kernel and the init image in it.
///
/// Cheap once there is nothing to do, so every test that needs a store calls
/// it. The lock is for a first run, where two would download into the same
/// directory at once.
pub fn provisioned() -> cfw::Store {
  let store = store();
  std::fs::create_dir_all(store.root()).expect("the store directory should be creatable");

  if store.ready().is_ok() {
    return store;
  }

  let _lock = Lock::take(store.root().join(PROVISION_LOCK));

  if store.ready().is_ok() {
    return store;
  }

  cfw::Builder::new(store.clone())
    .provision()
    .expect("the store should provision");

  store
}

/// The same store with [`TEST_IMAGE`] in it, built if it is not there already.
pub fn image() -> cfw::Store {
  let store = provisioned();

  if store.holds(TEST_IMAGE) {
    return store;
  }

  let _lock = Lock::take(store.root().join(IMAGE_LOCK));

  if store.holds(TEST_IMAGE) {
    return store;
  }

  build_image(&store);

  store
}

/// [`BASE_IMAGE`] plus one step leaving something a container can read back.
fn build_image(store: &cfw::Store) {
  let mut plan = cfw::BuildPlan::new(
    "cfw-test-builder",
    BASE_IMAGE,
    TEST_IMAGE,
    network::interface("cfw-test-builder"),
    "base-0",
  );

  plan.cpus = super::TEST_CPUS;
  plan.memory_in_bytes = super::TEST_MEMORY_IN_BYTES;
  plan.vm = super::vm();
  // Alpine carries no bash, which the default shell is.
  plan.shell = cfw::Shell(["/bin/sh", "-ec"].map(String::from).to_vec());
  plan.rootfs_size_in_bytes = super::TEST_ROOTFS_SIZE_IN_BYTES;
  plan.steps = vec![cfw::BuildStep {
    name: "marker".to_string(),
    script: format!("echo {MARKER} > {MARKER_PATH}"),
    user: None,
    cache_key: "marker-0".to_string(),
  }];

  cfw::Builder::new(store.clone())
    .build(&plan)
    .expect("the test image should build");
}

/// The digest the store's index records for `reference`.
pub fn digest(store: &cfw::Store, reference: &str) -> String {
  let index = std::fs::read_to_string(store.root().join("state.json")).expect("a readable index");
  let references: serde_json::Value = serde_json::from_str(&index).expect("a parseable index");

  references[reference]["digest"]
    .as_str()
    .unwrap_or_else(|| panic!("the index should record a digest for {reference}"))
    .to_string()
}
