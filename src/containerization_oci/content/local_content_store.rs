use super::Content;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;

/// `LocalContentStore`.
pub struct LocalContentStore {
  pub(crate) handle: ffi::CzLocalContentStore,
}

// Swift's `LocalContentStore` is an actor.
unsafe impl Send for LocalContentStore {}
unsafe impl Sync for LocalContentStore {}

impl LocalContentStore {
  /// `LocalContentStore(path:)`.
  pub fn new(path: &Path) -> Result<Self, Error> {
    let outcome = platform::outcome(
      ffi::cz_local_content_store_new(&path.display().to_string()),
      format!("open the content store at {}", path.display()),
    )?;

    Ok(Self {
      handle: outcome.local_content_store(),
    })
  }

  /// `LocalContentStore.get(digest:)`.
  pub fn get(&self, digest: &str) -> Result<Option<Content>, Error> {
    let handle = platform::outcome(self.handle.get(digest), format!("get {digest}"))?.content();

    Ok(handle.is_some().then_some(Content { handle }))
  }

  /// `LocalContentStore.delete(digests:)`: the digests deleted, and the bytes
  /// freed.
  pub fn delete(&self, digests: Vec<String>) -> Result<(Vec<String>, u64), Error> {
    let outcome = platform::outcome(self.handle.delete_digests(digests), "delete content")?;

    Ok((outcome.strings(), outcome.number()))
  }

  /// `LocalContentStore.delete(keeping:)`: the digests deleted, and the bytes
  /// freed.
  pub fn delete_keeping(&self, keeping: Vec<String>) -> Result<(Vec<String>, u64), Error> {
    let outcome = platform::outcome(self.handle.delete_keeping(keeping), "delete content")?;

    Ok((outcome.strings(), outcome.number()))
  }

  /// `LocalContentStore.ingest(_:)`: the digests ingested.
  ///
  /// `body` runs on another thread, as Swift's closure does, with the ingest
  /// directory. If it returns an error, nothing it wrote reaches the store, and
  /// `ingest` returns that error.
  pub fn ingest(&self, body: impl FnOnce(&Path) -> Result<(), Error> + Send + 'static) -> Result<Vec<String>, Error> {
    let failure = Arc::new(Mutex::new(None));
    let slot = Arc::clone(&failure);
    let outcome = self.handle.ingest(Box::new(move |directory: String| {
      body(Path::new(&directory))
        .map_err(|error| *slot.lock().unwrap_or_else(PoisonError::into_inner) = Some(error))
        .is_ok()
    }));

    platform::outcome(outcome, "ingest content")
      .map(|outcome| outcome.strings())
      .map_err(|error| {
        failure
          .lock()
          .unwrap_or_else(PoisonError::into_inner)
          .take()
          .unwrap_or(error)
      })
  }

  /// `LocalContentStore.newIngestSession()`: the session's id, and the
  /// directory to write its content into.
  pub fn new_ingest_session(&self) -> Result<(String, PathBuf), Error> {
    platform::outcome(self.handle.new_ingest_session(), "start an ingest session").map(|outcome| {
      (
        outcome.ingest_session_id(),
        PathBuf::from(outcome.ingest_session_directory()),
      )
    })
  }

  /// `LocalContentStore.completeIngestSession(_:)`: the digests ingested.
  pub fn complete_ingest_session(&self, id: &str) -> Result<Vec<String>, Error> {
    platform::outcome(
      self.handle.complete_ingest_session(id),
      format!("complete ingest session {id}"),
    )
    .map(|outcome| outcome.strings())
  }

  /// `LocalContentStore.cancelIngestSession(_:)`.
  pub fn cancel_ingest_session(&self, id: &str) -> Result<(), Error> {
    platform::outcome(
      self.handle.cancel_ingest_session(id),
      format!("cancel ingest session {id}"),
    )
    .map(drop)
  }

  /// `LocalContentStore.totalAllocatedSize()`.
  pub fn total_allocated_size(&self) -> Result<u64, Error> {
    platform::outcome(self.handle.total_allocated_size(), "size the content store").map(|outcome| outcome.number())
  }
}
