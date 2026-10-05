use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::path::PathBuf;

/// `Content`. Swift's generic `decode()` doesn't cross the bridge; decode
/// [`Content::data`] instead.
pub struct Content {
  pub(crate) handle: ffi::CzContent,
}

// Swift's `Content` is `Sendable`.
unsafe impl Send for Content {}
unsafe impl Sync for Content {}

impl Content {
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
