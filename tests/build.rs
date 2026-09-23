#![cfg(feature = "integration")]

//! Building an image, against the store a session really boots from.
//!
//! Only that a plan becomes an image the store holds. That the image boots, and
//! that a step's effect survives into a container, is `session.rs` — the same
//! build asserted from the other end.

mod support;

#[test]
fn builds_an_image_into_the_store() {
  let store = support::image();

  assert!(
    store.holds(support::TEST_IMAGE),
    "the index should name {}, and holds {:?}",
    support::TEST_IMAGE,
    store.images().expect("a readable index")
  );
  assert!(
    store
      .images()
      .expect("a readable index")
      .contains(&support::BASE_IMAGE.to_string()),
    "the base a build pulled should be in the store beside what it built"
  );
}
