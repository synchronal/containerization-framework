#![cfg(feature = "integration")]

//! `ImageStore` and `Image`, over the suite's store.

mod support;

use containerization_framework as cfw;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

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
fn shares_a_content_store() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let content_store = cfw::containerization_oci::LocalContentStore::new(&support::store::content_store_path())
    .expect("the suite's content store should open");
  let store = cfw::containerization::ImageStore::with_content_store(&directory.path().join("images"), &content_store)
    .expect("an image store sharing the suite's content store");

  assert_eq!(store.path(), directory.path().join("images"));
  assert!(
    store.get(support::store::IMAGE, false).is_err(),
    "a new image store has no references"
  );

  let image = store
    .pull(support::store::IMAGE)
    .expect("the image should pull");
  for digest in image.referenced_digests().expect("the image's digests") {
    assert!(
      content_store.get(&digest).expect("a lookup").is_some(),
      "{digest} should be in the shared content store"
    );
  }
}

const INDEX: &str = "application/vnd.oci.image.index.v1+json";
const MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
const CONFIG: &str = "application/vnd.oci.image.config.v1+json";
const LAYER: &str = "application/vnd.oci.image.layer.v1.tar";

/// Writes a one-layer linux/arm64 image's blobs into `directory` with a
/// `ContentWriter`, returning its index's descriptor. Nothing unpacks the
/// layer, so it needn't be a tar.
fn write_image(directory: &Path) -> Result<cfw::containerization_oci::Descriptor, cfw::Error> {
  let writer = cfw::containerization_oci::ContentWriter::new(directory)?;
  let scratch = tempfile::tempdir().expect("a temporary directory");
  let blob = |name: &str, contents: &str| {
    let path = scratch.path().join(name);
    std::fs::write(&path, contents).expect("the blob should write");
    writer.create(&path)
  };

  let (config_size, config) = blob(
    "config",
    r#"{"architecture":"arm64","os":"linux","rootfs":{"type":"layers","diff_ids":[]}}"#,
  )?;
  let (layer_size, layer) = blob("layer", "a layer")?;
  let (manifest_size, manifest) = blob(
    "manifest",
    &format!(
      r#"{{"schemaVersion":2,"mediaType":"{MANIFEST}","config":{{"mediaType":"{CONFIG}","digest":"{config}","size":{config_size}}},"layers":[{{"mediaType":"{LAYER}","digest":"{layer}","size":{layer_size}}}]}}"#
    ),
  )?;
  let (index_size, index) = blob(
    "index",
    &format!(
      r#"{{"schemaVersion":2,"mediaType":"{INDEX}","manifests":[{{"mediaType":"{MANIFEST}","digest":"{manifest}","size":{manifest_size},"platform":{{"architecture":"arm64","os":"linux"}}}}]}}"#
    ),
  )?;

  Ok(cfw::containerization_oci::Descriptor::new(INDEX, index, index_size))
}

#[test]
fn creates_an_image_from_ingested_blobs() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let content_store = cfw::containerization_oci::LocalContentStore::new(&directory.path().join("content"))
    .expect("a new content store should open");
  let store = cfw::containerization::ImageStore::with_content_store(&directory.path().join("images"), &content_store)
    .expect("a new image store should open");
  let (written, writer_saw) = std::sync::mpsc::channel();
  let reference = "containerization-framework/test-created:latest";

  content_store
    .ingest(move |ingest_directory| {
      written
        .send(write_image(ingest_directory)?)
        .expect("the test is listening");
      Ok(())
    })
    .expect("the image's blobs should ingest");
  let index = writer_saw.recv().expect("the body ran");

  let created = store
    .create(&cfw::containerization::image::Description::new(
      reference,
      index.clone(),
    ))
    .expect("the image should be created");

  assert_eq!(created.reference(), reference);
  assert_eq!(created.digest(), index.digest);
  assert_eq!(store.get(reference, false).expect("the new tag").digest(), index.digest);
}

#[test]
fn loads_an_oci_layout_and_reports_its_progress() {
  let layout = tempfile::tempdir().expect("a temporary directory");
  let blobs = layout.path().join("blobs").join("sha256");
  let reference = "containerization-framework/test-loaded:latest";
  std::fs::create_dir_all(&blobs).expect("the layout's blob directory");

  let index = write_image(&blobs).expect("the image's blobs should write");
  std::fs::write(layout.path().join("oci-layout"), r#"{"imageLayoutVersion":"1.0.0"}"#).expect("oci-layout");
  std::fs::write(
    layout.path().join("index.json"),
    format!(
      r#"{{"schemaVersion":2,"manifests":[{{"mediaType":"{INDEX}","digest":"{}","size":{},"annotations":{{"io.containerd.image.name":"{reference}"}}}}]}}"#,
      index.digest, index.size
    ),
  )
  .expect("index.json");

  let directory = tempfile::tempdir().expect("a temporary directory");
  let store = cfw::containerization::ImageStore::new(directory.path()).expect("a new image store should open");
  let events = Arc::new(Mutex::new(Vec::new()));
  let recording = Arc::clone(&events);

  let loaded = store
    .load(
      layout.path(),
      Some(Box::new(move |batch| {
        recording
          .lock()
          .expect("a recording")
          .extend_from_slice(batch)
      })),
    )
    .expect("the layout should load");

  assert_eq!(loaded.len(), 1);
  assert_eq!(loaded[0].reference(), reference);
  assert_eq!(loaded[0].digest(), index.digest);
  assert!(
    events
      .lock()
      .expect("the recording")
      .contains(&cfw::containerization_extras::ProgressEvent::AddTotalItems(1)),
    "the index is counted before it's fetched"
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
    .init_block(&at, cfw::containerization::SystemPlatform::LINUX_ARM)
    .expect("the init image should unpack");

  assert!(at.is_file(), "{}", at.display());
  assert!(mount.source.ends_with("/initfs.ext4"), "{}", mount.source);
  assert_eq!(mount.destination, "/");
}
