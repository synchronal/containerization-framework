use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::path::Path;

/// `ContentWriter`.
pub struct ContentWriter {
  handle: ffi::CzContentWriter,
}

// Swift's `ContentWriter` holds only its directory and an encoder. Rust's isn't
// `Sync`, so one call runs at a time.
unsafe impl Send for ContentWriter {}

impl ContentWriter {
  /// `ContentWriter(for:)`.
  pub fn new(base: &Path) -> Result<Self, Error> {
    let outcome = platform::outcome(
      ffi::cz_content_writer_new(&base.display().to_string()),
      format!("write content into {}", base.display()),
    )?;

    Ok(Self {
      handle: outcome.content_writer(),
    })
  }

  /// `ContentWriter.create(from:)`: the size, and the digest as its
  /// `digestString`, `sha256:<hex>`.
  pub fn create(&self, from: &Path) -> Result<(i64, String), Error> {
    let outcome = platform::outcome(
      self.handle.create(&from.display().to_string()),
      format!("write {} as content", from.display()),
    )?;

    Ok((outcome.written_size(), outcome.written_digest()))
  }
}
