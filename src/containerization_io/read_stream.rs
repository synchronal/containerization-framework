//! `ReadStream`, and the iterator its `dataStream` becomes.

use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::fmt;
use std::marker::PhantomData;
use std::path::Path;

/// `ReadStream`. Swift's `stream`, of NIO `ByteBuffer`s, is left out: it reads
/// the same bytes as `dataStream`.
pub struct ReadStream {
  handle: ffi::CzReadStream,
}

impl fmt::Debug for ReadStream {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.debug_struct("ReadStream").finish_non_exhaustive()
  }
}

// Swift's `ReadStream` is a class that isn't `Sendable`; Rust's isn't `Sync`,
// so one call runs at a time.
unsafe impl Send for ReadStream {}

impl ReadStream {
  /// `ReadStream.bufferSize`, the default size of each chunk read: 1 MiB.
  pub const BUFFER_SIZE: usize = 1024 * 1024;

  /// `ReadStream()`, which reads nothing.
  pub fn new() -> Result<Self, Error> {
    Self::opened(ffi::cz_read_stream_new(), "make an empty read stream".to_string())
  }

  /// `ReadStream(url:bufferSize:)`. Swift's `bufferSize` defaults to
  /// [`ReadStream::BUFFER_SIZE`].
  pub fn with_url(url: &Path, buffer_size: usize) -> Result<Self, Error> {
    Self::opened(
      ffi::cz_read_stream_with_url(platform::path(url)?, buffer_size),
      format!("read {}", url.display()),
    )
  }

  /// `ReadStream(data:bufferSize:)`. Swift's `bufferSize` defaults to
  /// [`ReadStream::BUFFER_SIZE`].
  pub fn with_data(data: &[u8], buffer_size: usize) -> Result<Self, Error> {
    Self::opened(
      ffi::cz_read_stream_with_data(data.to_vec(), buffer_size),
      "read bytes as a stream".to_string(),
    )
  }

  fn opened(outcome: ffi::CzOutcome, action: String) -> Result<Self, Error> {
    platform::outcome(outcome, action).map(|outcome| Self {
      handle: outcome.read_stream(),
    })
  }

  /// `ReadStream.reset()`.
  pub fn reset(&self) -> Result<(), Error> {
    platform::outcome(self.handle.reset(), "reset a read stream").map(drop)
  }

  /// `ReadStream.dataStream`. Like Swift's, it reads the whole source when it
  /// is made, and the source can be read once until a [`ReadStream::reset`].
  pub fn data_stream(&self) -> DataStream<'_> {
    DataStream {
      handle: self.handle.data_stream(),
      stream: PhantomData,
    }
  }
}

/// `ReadStream.dataStream`'s `AsyncStream<Data>`, whose `next()` blocks.
pub struct DataStream<'a> {
  handle: ffi::CzDataStream,
  stream: PhantomData<&'a ReadStream>,
}

impl fmt::Debug for DataStream<'_> {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.debug_struct("DataStream").finish_non_exhaustive()
  }
}

impl Iterator for DataStream<'_> {
  type Item = Vec<u8>;

  fn next(&mut self) -> Option<Vec<u8>> {
    let outcome = self.handle.next();

    outcome.is_some().then(|| outcome.bytes())
  }
}
