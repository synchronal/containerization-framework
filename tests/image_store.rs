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
    .pull(support::store::IMAGE, Default::default())
    .expect("the image should pull");
  for digest in image.referenced_digests().expect("the image's digests") {
    assert!(
      content_store.get(&digest).expect("a lookup").is_some(),
      "{digest} should be in the shared content store"
    );
  }
}

/// The blobs are already in the shared content store, so only the index is
/// fetched from the registry.
#[test]
fn pulls_one_platform_and_reports_its_progress() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let content_store = cfw::containerization_oci::LocalContentStore::new(&support::store::content_store_path())
    .expect("the suite's content store should open");
  let store = cfw::containerization::ImageStore::with_content_store(&directory.path().join("images"), &content_store)
    .expect("an image store sharing the suite's content store");
  let platform = cfw::containerization_oci::Platform::current().expect("the current platform");
  let events = Arc::new(Mutex::new(Vec::new()));
  let recording = Arc::clone(&events);

  let image = store
    .pull(
      support::store::IMAGE,
      cfw::containerization::image_store::PullOptions {
        platform: Some(platform.clone()),
        progress: Some(Box::new(move |batch| {
          recording
            .lock()
            .expect("a recording")
            .extend_from_slice(batch)
        })),
        ..Default::default()
      },
    )
    .expect("the image should pull");

  let events = events.lock().expect("the recording");
  let sum = |pick: fn(&cfw::containerization_extras::ProgressEvent) -> Option<isize>| {
    events.iter().filter_map(pick).sum::<isize>()
  };
  let items = sum(|event| match event {
    cfw::containerization_extras::ProgressEvent::AddItems(items) => Some(*items),
    _ => None,
  });
  let total = sum(|event| match event {
    cfw::containerization_extras::ProgressEvent::AddTotalItems(total) => Some(*total),
    _ => None,
  });
  assert!(total > 0, "{events:?}");
  assert_eq!(items, total, "every blob counted should be fetched: {events:?}");
  assert!(
    image.manifest(&platform).is_ok(),
    "the platform asked for should be pulled"
  );
}

#[test]
fn saves_an_image_that_another_store_loads() {
  let store = support::store::image_store();
  let image = support::store::image(&store);
  let platform = cfw::containerization_oci::Platform::current().expect("the current platform");
  let layout = tempfile::tempdir().expect("a temporary directory");
  let directory = tempfile::tempdir().expect("a temporary directory");

  store
    .save(&[support::store::IMAGE], layout.path(), Some(&platform))
    .expect("the image should save");
  let loaded = cfw::containerization::ImageStore::new(directory.path())
    .expect("a new image store should open")
    .load(layout.path(), None)
    .expect("the layout should load");

  assert_eq!(loaded.len(), 1);
  assert_eq!(loaded[0].reference(), support::store::IMAGE);
  assert_eq!(
    loaded[0]
      .manifest(&platform)
      .expect("the loaded image's manifest")
      .layers,
    image
      .manifest(&platform)
      .expect("the saved image's manifest")
      .layers,
  );
}

/// Swift looks the image up before it reaches the registry, so nothing is
/// sent.
#[test]
fn says_which_image_it_could_not_push() {
  let reference = "example.test/containerization-framework/never-pulled:latest";
  let error = support::store::image_store()
    .push(
      reference,
      cfw::containerization::image_store::PushOptions {
        auth: Some(cfw::containerization_oci::Authentication::basic("user", "password").expect("an authentication")),
        ..Default::default()
      },
    )
    .err()
    .expect("an image the store doesn't hold");

  assert!(error.to_string().contains(reference), "{error}");
  assert!(error.is_code(cfw::containerization_error::Code::NotFound), "{error}");
}

#[test]
fn pushes_to_one_registry_at_a_time() {
  let error = support::store::image_store()
    .push_all(
      &["a.example.test/image:latest", "b.example.test/image:latest"],
      Default::default(),
    )
    .err()
    .expect("references to two registries");

  assert!(
    error.is_code(cfw::containerization_error::Code::InvalidArgument),
    "{error}"
  );
}

/// Opening it makes its directory, if it isn't there yet.
#[test]
fn opens_the_default_store() {
  let store = cfw::containerization::ImageStore::default_store().expect("the default store");

  assert!(
    store
      .path()
      .ends_with("Application Support/com.apple.containerization"),
    "{}",
    store.path().display()
  );
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
fn makes_an_image_from_its_description() {
  let image = support::store::image(&support::store::image_store());
  let content_store = cfw::containerization_oci::LocalContentStore::new(&support::store::content_store_path())
    .expect("the suite's content store should open");

  let made = cfw::containerization::Image::new(&image.description(), &content_store);

  assert_eq!(made.reference(), image.reference());
  assert_eq!(
    made.referenced_digests().expect("the made image's digests"),
    image.referenced_digests().expect("the image's digests")
  );
}

/// A new image store and content store, in `directory`.
fn new_stores(
  directory: &Path,
) -> (
  cfw::containerization::ImageStore,
  cfw::containerization_oci::LocalContentStore,
) {
  let content_store = cfw::containerization_oci::LocalContentStore::new(&directory.join("content"))
    .expect("a new content store should open");
  let store = cfw::containerization::ImageStore::with_content_store(&directory.join("images"), &content_store)
    .expect("a new image store should open");

  (store, content_store)
}

#[test]
fn creates_an_init_image_from_a_rootfs() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let (store, content_store) = new_stores(directory.path());
  let platform = cfw::containerization_oci::Platform::parse("linux/arm64").expect("a platform");
  let reference = "containerization-framework/test-init:latest";
  let labels = [("purpose".to_string(), "test".to_string())].into();

  let rootfs = directory.path().join("rootfs");
  std::fs::create_dir(&rootfs).expect("a rootfs directory");
  std::fs::write(rootfs.join("sbin-init"), "#!/bin/sh\n").expect("a file in the rootfs");
  let archive = directory.path().join("rootfs.tar.gz");
  let tarred = std::process::Command::new("tar")
    .arg("-czf")
    .arg(&archive)
    .arg("-C")
    .arg(&rootfs)
    .arg(".")
    .status()
    .expect("tar should run");
  assert!(tarred.success());

  let init_image =
    cfw::containerization::InitImage::create(reference, &archive, &platform, &labels, &store, &content_store)
      .expect("the init image should be created");

  assert_eq!(init_image.name(), reference);
  let config = store
    .get(reference, false)
    .expect("the new image")
    .config(&platform)
    .expect("its config");
  assert_eq!(config.config.and_then(|config| config.labels), Some(labels));
  assert_eq!(
    cfw::containerization::InitImage::new(&store.get(reference, false).expect("the new image")).name(),
    reference
  );
}

/// Nothing boots the kernel, so any file will do.
#[test]
fn creates_a_kernel_image_and_finds_its_kernel() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let (store, content_store) = new_stores(directory.path());
  let reference = "containerization-framework/test-kernel:latest";
  let binary = directory.path().join("vmlinux");
  std::fs::write(&binary, "a kernel").expect("a kernel file");
  let kernel = cfw::containerization::Kernel::new(&binary, cfw::containerization::SystemPlatform::LINUX_ARM);

  let kernel_image = cfw::containerization::KernelImage::create(
    reference,
    &[kernel.clone()],
    &Default::default(),
    &store,
    &content_store,
  )
  .expect("the kernel image should be created");
  let found = kernel_image
    .kernel(cfw::containerization::SystemPlatform::LINUX_ARM)
    .expect("the kernel for linux/arm64");

  assert_eq!(kernel_image.name(), reference);
  assert_eq!(found.platform, kernel.platform);
  assert_eq!(found.command_line, kernel.command_line);
  assert_eq!(
    std::fs::read(&found.path).expect("the stored kernel"),
    b"a kernel",
    "{}",
    found.path.display()
  );
  assert!(
    kernel_image
      .kernel(cfw::containerization::SystemPlatform::LINUX_AMD)
      .is_err(),
    "there is no linux/amd64 kernel"
  );
  assert_eq!(
    cfw::containerization::KernelImage::new(&store.get(reference, false).expect("the new image")).name(),
    reference
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
  assert!(error.is_code(cfw::containerization_error::Code::NotFound), "{error}");
}

#[test]
fn unpacks_an_init_image() {
  let store = support::store::image_store();
  let directory = tempfile::tempdir().expect("a temporary directory");
  let at = directory.path().join("initfs.ext4");

  let init_image = store
    .get_init_image(support::store::INITFS_REFERENCE, None, None)
    .expect("the init image");
  let mount = init_image
    .init_block(&at, cfw::containerization::SystemPlatform::LINUX_ARM)
    .expect("the init image should unpack");

  assert!(at.is_file(), "{}", at.display());
  assert!(mount.source.ends_with("/initfs.ext4"), "{}", mount.source);
  assert_eq!(mount.destination, "/");
}
