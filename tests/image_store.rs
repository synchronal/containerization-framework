#![cfg(feature = "integration")]

//! `ImageStore` and `Image`, over the suite's store.

mod support;

#[test]
fn gets_lists_tags_and_deletes_images() {
  let store = support::store::image_store();
  let image = support::store::image(&store);
  let tag = "containerization-framework/test-tag:image-store";

  assert_eq!(image.reference(), support::store::IMAGE);
  assert!(
    store
      .list()
      .expect("the store's images")
      .iter()
      .any(|listed| listed.reference() == support::store::IMAGE)
  );

  let tagged = store.tag(support::store::IMAGE, tag).expect("a new tag");
  assert_eq!(tagged.digest(), image.digest(), "a tag names the same image");
  assert_eq!(store.get(tag, false).expect("the tag").reference(), tag);

  store.delete(tag, false).expect("the tag should delete");
  assert!(store.get(tag, false).is_err(), "a deleted tag should be gone");
  assert!(
    store.get(support::store::IMAGE, false).is_ok(),
    "and the image it named should not"
  );
}

#[test]
fn says_which_image_it_could_not_get() {
  let error = support::store::image_store()
    .get("containerization-framework/never-pulled:latest", false)
    .err()
    .expect("an image the store doesn't hold");

  assert!(
    error
      .to_string()
      .contains("containerization-framework/never-pulled:latest"),
    "{error}"
  );
}

#[test]
fn unpacks_an_init_image() {
  let store = support::store::image_store();
  let directory = tempfile::tempdir().expect("a temporary directory");
  let at = directory.path().join("initfs.ext4");

  let init_image = store
    .get_init_image(support::store::INITFS_REFERENCE)
    .expect("the init image");
  let mount = init_image
    .init_block(
      &at,
      containerization_framework::containerization::SystemPlatform::LINUX_ARM,
    )
    .expect("the init image should unpack");

  assert!(at.is_file(), "{}", at.display());
  assert!(mount.source.ends_with("/initfs.ext4"), "{}", mount.source);
  assert_eq!(mount.destination, "/");
}
