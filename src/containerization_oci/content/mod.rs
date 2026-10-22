//! Content: blobs addressed by their digest, the digests themselves, the
//! store the blobs are kept in, and a writer of blobs to a directory.

mod content_writer;
mod local_content_store;
mod parsed_digest;

pub use self::content_writer::ContentWriter;
pub use self::local_content_store::LocalContentStore;
pub use self::parsed_digest::ParsedDigest;

use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::path::Path;
use std::path::PathBuf;

/// `Content`, which in Swift is a protocol that `LocalContent` conforms to.
/// Swift's generic `decode()` doesn't cross the bridge; decode
/// [`Content::data`] instead.
///
/// Swift's `Content` is `Sendable`, but `LocalContent` seeks one shared file
/// handle before each read. Two threads reading the same `Content` could each
/// read from the other's position, so Rust's is `Send` but not `Sync`.
///
/// ```compile_fail
/// # use containerization_framework as cfw;
/// let content = cfw::containerization_oci::content::Content::open("blob".as_ref())?;
/// std::thread::scope(|scope| {
///   scope.spawn(|| content.size());
/// });
/// # Ok::<(), cfw::Error>(())
/// ```
pub struct Content {
  pub(crate) handle: ffi::CzContent,
}

opaque_debug!(Content);

// Swift's `Content` is `Sendable`. Rust's isn't `Sync`, so one call runs at a
// time.
unsafe impl Send for Content {}

impl Content {
  /// `LocalContent.maxDecodedSize`: the largest content [`Content::data`]
  /// reads whole.
  pub const MAX_DECODED_SIZE: isize = 4 * 1024 * 1024;

  /// `LocalContent(path:)`, which reads the file at `path` without a store.
  pub fn open(path: &Path) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_local_content_open(platform::path(path)?),
      format!("open {} as content", path.display()),
    )
    .map(|outcome| Self {
      handle: outcome.content(),
    })
  }

  /// `Content.path`.
  pub fn path(&self) -> PathBuf {
    PathBuf::from(self.handle.path())
  }

  /// `Content.digest()`, as its `digestString`: `sha256:<hex>`.
  pub fn digest(&self) -> Result<String, Error> {
    platform::outcome(self.handle.digest(), format!("digest {}", self.path().display())).map(|outcome| outcome.text())
  }

  /// `Content.size()`.
  pub fn size(&self) -> Result<u64, Error> {
    platform::outcome(self.handle.size(), format!("size {}", self.path().display())).map(|outcome| outcome.number())
  }

  /// `Content.data()`.
  pub fn data(&self) -> Result<Vec<u8>, Error> {
    platform::outcome(self.handle.data(), format!("read {}", self.path().display())).map(|outcome| outcome.bytes())
  }

  /// `Content.data(offset:length:)`.
  pub fn data_range(&self, offset: u64, length: usize) -> Result<Option<Vec<u8>>, Error> {
    let outcome = platform::outcome(
      self.handle.data_range(offset, length),
      format!("read {}", self.path().display()),
    )?;

    Ok(outcome.has_bytes().then(|| outcome.bytes()))
  }
}
