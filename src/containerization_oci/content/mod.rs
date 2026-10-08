//! Content: blobs addressed by their digest, and the store they are kept in.
//! `Content` is here.

mod content_writer;
mod local_content_store;
mod parsed_digest;

pub use self::content_writer::ContentWriter;
pub use self::local_content_store::LocalContentStore;
pub use self::parsed_digest::ParsedDigest;

use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;

/// `Content`, which in Swift is a protocol that `LocalContent` conforms to.
/// Swift's generic `decode()` doesn't cross the bridge; decode
/// [`Content::data`] instead.
pub struct Content {
  pub(crate) handle: ffi::CzContent,
}

impl fmt::Debug for Content {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.debug_struct("Content").finish_non_exhaustive()
  }
}

// Swift's `Content` is `Sendable`.
unsafe impl Send for Content {}
unsafe impl Sync for Content {}

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
