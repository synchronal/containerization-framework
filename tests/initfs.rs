#![cfg(feature = "integration")]

//! Which kernel and init image a VM boots: the ones at the paths its store
//! names.

mod support;

use containerization_framework as cfw;
use support::container::Container;

#[test]
fn boots_the_kernel_and_init_image_at_the_stores_paths() {
  let container = Container::boot("cfw-test-initfs");
  let store = container.session().store();

  assert_eq!(container.exec("true", &["/bin/true"]), 0);
  assert!(
    store.kernel().is_file(),
    "the kernel should be at {}",
    store.kernel().display()
  );
  assert!(
    store.initfs().is_file(),
    "the init image should be unpacked to {}",
    store.initfs().display()
  );
}

/// The pinned image by digest: the only other init image we have.
#[test]
fn boots_the_init_image_where_the_caller_puts_it() {
  let unpacked = tempfile::tempdir().expect("a temporary directory");
  let initfs = unpacked.path().join("vminit-pinned.ext4");

  let store = support::store::image();
  let digest = support::store::digest(&store, cfw::INITFS_REFERENCE);
  let (repository, _) = cfw::INITFS_REFERENCE
    .rsplit_once(':')
    .expect("the pinned init image is tagged");
  let by_digest = format!("{repository}@{digest}");

  let pinned = cfw::Store::at(store.root(), store.kernel(), &by_digest, &initfs);

  cfw::Builder::new(pinned.clone())
    .provision()
    .unwrap_or_else(|error| panic!("{by_digest} should provision: {error}"));
  assert_eq!(
    pinned.ready(),
    Ok(()),
    "a store is ready once it holds the image it boots"
  );
  assert!(
    initfs.is_file(),
    "provisioning should unpack the init image to {}",
    initfs.display()
  );

  let container = Container::try_boot_in(pinned, "cfw-test-initfs-pinned", |_| {})
    .unwrap_or_else(|error| panic!("a store should boot {by_digest}: {error}"));

  assert_eq!(container.exec("true", &["/bin/true"]), 0);
}
