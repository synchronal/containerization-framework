#![cfg(feature = "integration")]

//! `LocalContentStore` and `Content`, over an image the suite pulled.

mod support;

use containerization_framework as cfw;

/// Well formed, and the hash of nothing this suite stores.
const ABSENT: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn content_store() -> cfw::containerization_oci::LocalContentStore {
  cfw::containerization_oci::LocalContentStore::new(&support::store::content_store_path())
    .expect("the suite's content store should open")
}

#[test]
fn gets_each_blob_an_image_is_made_of() {
  let image = support::store::image(&support::store::image_store());
  let store = content_store();

  let digests = image
    .referenced_digests()
    .expect("a pulled image's digests");

  assert!(digests.len() > 1, "an image is more than its index: {digests:?}");
  assert_eq!(
    format!("sha256:{}", digests[0]),
    image.digest(),
    "the image's own digest leads"
  );

  for digest in &digests {
    let content = store
      .get(digest)
      .expect("a lookup")
      .unwrap_or_else(|| panic!("a pulled image's blob {digest} should be in the store"));

    assert!(content.path().is_file(), "{}", content.path().display());
    assert_eq!(content.digest().expect("a digest"), format!("sha256:{digest}"));
    assert_eq!(
      content.size().expect("a size"),
      content.data().expect("the blob's bytes").len() as u64
    );
  }
}

#[test]
fn reads_part_of_a_blob() {
  let image = support::store::image(&support::store::image_store());
  let content = image
    .get_content(&image.digest())
    .expect("the image's index");
  let whole = content.data().expect("the index's bytes");

  assert_eq!(content.data_range(1, 4).expect("a read"), Some(whole[1..5].to_vec()));
}

#[test]
fn finds_no_blob_the_store_never_held() {
  assert!(content_store().get(ABSENT).expect("a lookup").is_none());
}

#[test]
fn says_which_blob_it_could_not_get() {
  let error = content_store()
    .get("not-a-digest")
    .err()
    .expect("a malformed digest");

  assert!(error.to_string().contains("not-a-digest"), "{error}");
}
