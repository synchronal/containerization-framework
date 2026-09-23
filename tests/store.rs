#![cfg(feature = "integration")]

//! Provisioning, against a real store.
//!
//! Unit tests say what `ready` makes of a directory laid out one way or
//! another; only a store `provision` really filled says the two agree.

mod support;

use containerization_framework::{Builder, INITFS_REFERENCE};

#[test]
fn provisions_a_bootable_store() {
  let store = support::provisioned();

  assert_eq!(store.ready(), Ok(()));
  assert!(store.kernel().is_file(), "the kernel should be in the store");
  assert!(
    store.holds(INITFS_REFERENCE),
    "the index should name {INITFS_REFERENCE}, and holds {:?}",
    store.images().expect("a readable index")
  );
}

#[test]
fn provisions_idempotently() {
  let store = support::provisioned();

  Builder::new(store.clone())
    .provision()
    .expect("a second provision should be a cheap no-op");

  assert_eq!(store.ready(), Ok(()));
}
