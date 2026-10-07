#![cfg(feature = "integration")]

//! `LocalContentStore` and `Content`, over an image the suite pulled.

mod support;

use containerization_framework as cfw;

/// Well formed, and the hash of nothing this suite stores.
const ABSENT: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn content_store() -> cfw::containerization_oci::content::LocalContentStore {
  cfw::containerization_oci::content::LocalContentStore::new(&support::store::content_store_path())
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

/// A file outside any ingest directory, and a store of its own.
fn blob_and_store(contents: &str) -> (tempfile::TempDir, cfw::containerization_oci::content::LocalContentStore) {
  let directory = tempfile::tempdir().expect("a temporary directory");
  std::fs::write(directory.path().join("blob"), contents).expect("the blob should write");
  let store = cfw::containerization_oci::content::LocalContentStore::new(&directory.path().join("content"))
    .expect("a new content store should open");

  (directory, store)
}

#[test]
fn ingests_what_a_content_writer_wrote() {
  let (directory, store) = blob_and_store("ingested");
  let blob = directory.path().join("blob");
  let (written, writer_saw) = std::sync::mpsc::channel();

  let ingested = store
    .ingest(move |ingest_directory| {
      let created = cfw::containerization_oci::content::ContentWriter::new(ingest_directory)?.create(&blob)?;
      written.send(created).expect("the test is listening");
      Ok(())
    })
    .expect("the ingest should complete");
  let (size, digest) = writer_saw.recv().expect("the body ran");

  assert_eq!(size, "ingested".len() as i64);
  assert!(digest.starts_with("sha256:"), "{digest}");
  assert_eq!(ingested, [digest.trim_start_matches("sha256:")]);

  let content = store
    .get(&digest)
    .expect("a lookup")
    .expect("the ingested blob should be in the store");
  assert_eq!(content.data().expect("the blob's bytes"), b"ingested");
}

#[test]
fn returns_the_error_its_body_returned_and_ingests_nothing() {
  let (directory, store) = blob_and_store("abandoned");
  let blob = directory.path().join("blob");
  let (written, writer_saw) = std::sync::mpsc::channel();

  let error = store
    .ingest(move |ingest_directory| {
      let (_, digest) = cfw::containerization_oci::content::ContentWriter::new(ingest_directory)?.create(&blob)?;
      written.send(digest).expect("the test is listening");
      Err(cfw::Error::failed("finish the body", "it gave up"))
    })
    .err()
    .expect("a failed body");
  let digest = writer_saw.recv().expect("the body ran");

  assert_eq!(error.action(), "finish the body");
  assert!(store.get(&digest).expect("a lookup").is_none());
}

#[test]
fn runs_a_body_that_calls_back_into_swift() {
  let (_directory, store) = blob_and_store("unused");
  let (found, body_saw) = std::sync::mpsc::channel();

  store
    .ingest(move |_| {
      let store = support::store::image_store();
      found
        .send(support::store::image(&store).reference())
        .expect("the test is listening");
      Ok(())
    })
    .expect("the ingest should complete");

  assert_eq!(body_saw.recv().expect("the body ran"), support::store::IMAGE);
}

#[test]
fn completes_an_ingest_session() {
  let (_directory, store) = blob_and_store("unused");

  let (id, ingest_directory) = store.new_ingest_session().expect("a new session");
  assert!(ingest_directory.is_dir(), "{}", ingest_directory.display());
  let (size, digest) = cfw::containerization_oci::content::ContentWriter::new(&ingest_directory)
    .expect("a writer into the session")
    .write(b"written")
    .expect("the data should write");
  let ingested = store
    .complete_ingest_session(&id)
    .expect("the session should complete");

  assert_eq!(size, "written".len() as i64);
  assert_eq!(ingested, [digest.trim_start_matches("sha256:")]);
  assert_eq!(
    store
      .get(&digest)
      .expect("a lookup")
      .expect("the ingested blob should be in the store")
      .data()
      .expect("the blob's bytes"),
    b"written"
  );
  assert!(
    store.complete_ingest_session(&id).is_err(),
    "a completed session is over"
  );
}

#[test]
fn cancels_an_ingest_session() {
  let (_directory, store) = blob_and_store("unused");

  let (id, ingest_directory) = store.new_ingest_session().expect("a new session");
  let (_, digest) = cfw::containerization_oci::content::ContentWriter::new(&ingest_directory)
    .expect("a writer into the session")
    .write(b"cancelled")
    .expect("the data should write");
  store
    .cancel_ingest_session(&id)
    .expect("the session should cancel");

  assert!(!ingest_directory.exists(), "{}", ingest_directory.display());
  assert!(store.get(&digest).expect("a lookup").is_none());
}

#[test]
fn copies_a_file_and_reads_it_without_a_store() {
  let (directory, _store) = blob_and_store("copied");
  let destination = directory.path().join("copy");

  let (size, digest) =
    cfw::containerization_oci::content::ContentWriter::copy(&directory.path().join("blob"), &destination)
      .expect("the blob should copy");
  let content = cfw::containerization_oci::content::Content::open(&destination).expect("the copy as content");

  assert_eq!(size, "copied".len() as i64);
  assert_eq!(content.path(), destination);
  assert_eq!(content.digest().expect("a digest"), digest);
  assert_eq!(content.data().expect("the copy's bytes"), b"copied");
  assert!(
    cfw::containerization_oci::content::ContentWriter::copy(&directory.path().join("blob"), &destination).is_err(),
    "a copy refuses to overwrite"
  );
}

#[test]
fn says_which_file_it_could_not_open_as_content() {
  let error = cfw::containerization_oci::content::Content::open("/nowhere/at/all".as_ref())
    .err()
    .expect("a file that doesn't exist");

  assert!(error.to_string().contains("/nowhere/at/all"), "{error}");
  assert!(error.is_code(cfw::containerization_error::Code::NotFound), "{error}");
}

#[test]
fn says_which_directory_a_content_writer_could_not_write_into() {
  let error = cfw::containerization_oci::content::ContentWriter::new("/nowhere/at/all".as_ref())
    .err()
    .expect("a directory that doesn't exist");

  assert!(error.to_string().contains("/nowhere/at/all"), "{error}");
}

#[test]
fn says_which_blob_it_could_not_get() {
  let error = content_store()
    .get("not-a-digest")
    .err()
    .expect("a malformed digest");

  assert!(error.to_string().contains("not-a-digest"), "{error}");
}
