use super::Content;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::path::Path;

/// `LocalContentStore`.
pub struct LocalContentStore {
  handle: ffi::CzLocalContentStore,
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

  /// `LocalContentStore.totalAllocatedSize()`.
  pub fn total_allocated_size(&self) -> Result<u64, Error> {
    platform::outcome(self.handle.total_allocated_size(), "size the content store").map(|outcome| outcome.number())
  }
}
