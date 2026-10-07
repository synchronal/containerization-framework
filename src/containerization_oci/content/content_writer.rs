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

  /// `ContentWriter.write(_:)`: the size, and the digest as its
  /// `digestString`, `sha256:<hex>`.
  pub fn write(&self, data: &[u8]) -> Result<(i64, String), Error> {
    platform::outcome(self.handle.write(data.to_vec()), "write data as content").map(|outcome| written(&outcome))
  }

  /// `ContentWriter.create(from:)`: the size, and the digest as its
  /// `digestString`, `sha256:<hex>`. The generic `create(from:)`, which
  /// encodes a value as JSON, isn't bound: serialize it and [`Self::write`]
  /// it instead.
  pub fn create(&self, from: &Path) -> Result<(i64, String), Error> {
    platform::outcome(
      self.handle.create(&from.display().to_string()),
      format!("write {} as content", from.display()),
    )
    .map(|outcome| written(&outcome))
  }

  /// `ContentWriter.copy(from:destination:)`: the size, and the digest as its
  /// `digestString`. It fails if `destination` exists.
  pub fn copy(from: &Path, destination: &Path) -> Result<(i64, String), Error> {
    platform::outcome(
      ffi::cz_content_writer_copy(&from.display().to_string(), &destination.display().to_string()),
      format!("copy {} to {}", from.display(), destination.display()),
    )
    .map(|outcome| written(&outcome))
  }
}

fn written(outcome: &ffi::CzOutcome) -> (i64, String) {
  (outcome.written_size(), outcome.written_digest())
}
