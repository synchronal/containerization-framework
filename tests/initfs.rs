#![cfg(feature = "integration")]

//! Which init image a VM boots.
//!
//! As in Containerization, a store unpacks its init image once, to
//! `initfs.ext4`, and boots that file from then on.

mod support;

use containerization_framework as cfw;
use support::Container;

#[test]
fn boots_the_init_image_unpacked_into_the_store() {
  let container = Container::boot("cfw-test-initfs");

  assert_eq!(container.exec("true", &["/bin/true"]), 0);
  assert!(
    container.session().store().initfs().is_file(),
    "the init image should be unpacked where Containerization puts it"
  );
}

/// The default image by digest: the only other reference available without
/// publishing our own.
#[test]
fn boots_the_init_image_its_store_is_given() {
  let store = support::image();
  let digest = support::digest(&store, cfw::INITFS_REFERENCE);
  let (repository, _) = cfw::INITFS_REFERENCE
    .rsplit_once(':')
    .expect("the default init image is tagged");
  let by_digest = format!("{repository}@{digest}");

  let pinned = store.clone().with_initfs_reference(&by_digest);

  cfw::Builder::new(pinned.clone())
    .provision()
    .unwrap_or_else(|error| panic!("{by_digest} should provision: {error}"));
  assert_eq!(
    pinned.ready(),
    Ok(()),
    "a store is ready once it holds the image it boots"
  );

  let container = Container::try_boot_in(pinned, "cfw-test-initfs-pinned", |_| {})
    .unwrap_or_else(|error| panic!("a store should boot {by_digest}: {error}"));

  assert_eq!(container.exec("true", &["/bin/true"]), 0);
}
