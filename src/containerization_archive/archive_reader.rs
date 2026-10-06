//! `ArchiveReader`, and its nested iterators.

use super::ArchiveEntryReader;
use super::Filter;
use super::Format;
use super::WriteEntry;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::marker::PhantomData;
use std::os::fd::IntoRawFd;
use std::os::fd::OwnedFd;
use std::path::Path;

/// `ArchiveReader`. Swift's `Sequence` conformance is
/// `IntoIterator for &ArchiveReader`.
pub struct ArchiveReader {
  pub(crate) handle: ffi::CzArchiveReader,
}

// Swift's `ArchiveReader` is a class that isn't `Sendable`; Rust's isn't
// `Sync`, so one call runs at a time.
unsafe impl Send for ArchiveReader {}

impl ArchiveReader {
  /// `ArchiveReader(file:)`, which detects the format and filter.
  pub fn new(file: &Path) -> Result<Self, Error> {
    Self::opened(
      ffi::cz_archive_reader_new(&file.display().to_string()),
      format!("read the archive {}", file.display()),
    )
  }

  /// `ArchiveReader(format:filter:file:)`.
  pub fn with_format(format: Format, filter: Filter, file: &Path) -> Result<Self, Error> {
    Self::opened(
      ffi::cz_archive_reader_with_format(format.raw_value(), filter.raw_value(), &file.display().to_string()),
      format!("read the {} archive {}", format.raw_value(), file.display()),
    )
  }

  /// `ArchiveReader(format:filter:fileHandle:)`. The reader owns
  /// `file_handle`, and closes it.
  pub fn with_file_handle(format: Format, filter: Filter, file_handle: OwnedFd) -> Result<Self, Error> {
    Self::opened(
      ffi::cz_archive_reader_with_file_handle(format.raw_value(), filter.raw_value(), file_handle.into_raw_fd()),
      format!("read a {} archive from a file descriptor", format.raw_value()),
    )
  }

  /// `ArchiveReader(name:bundle:tempDirectoryBaseName:)`.
  pub fn with_bundle(name: &str, bundle: &[u8], temp_directory_base_name: Option<&str>) -> Result<Self, Error> {
    Self::opened(
      ffi::cz_archive_reader_with_bundle(name, bundle.to_vec(), temp_directory_base_name.map(str::to_string)),
      format!("read the bundled archive {name}"),
    )
  }

  fn opened(outcome: ffi::CzOutcome, action: String) -> Result<Self, Error> {
    let outcome = platform::outcome(outcome, action)?;

    Ok(Self {
      handle: outcome.archive_reader(),
    })
  }

  /// `ArchiveReader.makeIterator()`.
  pub fn make_iterator(&self) -> Iterator<'_> {
    Iterator {
      handle: self.handle.make_iterator(),
      reader: PhantomData,
    }
  }

  /// `ArchiveReader.makeStreamingIterator()`.
  pub fn make_streaming_iterator(&self) -> StreamingIterator<'_> {
    StreamingIterator {
      handle: self.handle.make_streaming_iterator(),
      reader: PhantomData,
    }
  }

  /// `ArchiveReader.throwIfStreamFailed()`.
  pub fn throw_if_stream_failed(&self) -> Result<(), Error> {
    platform::outcome(self.handle.throw_if_stream_failed(), "read an archive").map(|_| ())
  }

  /// `ArchiveReader.extractContents(to:)`: the member paths it rejected.
  pub fn extract_contents(&self, to: &Path) -> Result<Vec<String>, Error> {
    platform::outcome(
      self.handle.extract_contents(&to.display().to_string()),
      format!("extract an archive to {}", to.display()),
    )
    .map(|outcome| outcome.strings())
  }

  /// `ArchiveReader.extractFile(path:)`.
  pub fn extract_file(&self, path: &str) -> Result<(WriteEntry, Vec<u8>), Error> {
    platform::outcome(
      self.handle.extract_file(path),
      format!("extract {path} from an archive"),
    )
    .map(|outcome| read(&outcome))
  }
}

impl<'a> IntoIterator for &'a ArchiveReader {
  type Item = (WriteEntry, Vec<u8>);
  type IntoIter = Iterator<'a>;

  fn into_iter(self) -> Self::IntoIter {
    self.make_iterator()
  }
}

/// An entry and its data, as Swift's iterator and `extractFile` return them.
fn read(outcome: &ffi::CzOutcome) -> (WriteEntry, Vec<u8>) {
  (
    WriteEntry {
      handle: outcome.write_entry(),
    },
    outcome.entry_data(),
  )
}

/// `ArchiveReader.Iterator`.
pub struct Iterator<'a> {
  handle: ffi::CzArchiveIterator,
  reader: PhantomData<&'a ArchiveReader>,
}

impl std::iter::Iterator for Iterator<'_> {
  type Item = (WriteEntry, Vec<u8>);

  /// `ArchiveReader.Iterator.next()`.
  fn next(&mut self) -> Option<Self::Item> {
    let outcome = self.handle.next();

    outcome.is_some().then(|| read(&outcome))
  }
}

/// `ArchiveReader.StreamingIterator`. Each entry's reader reads from the
/// archive's current entry, so read it before taking the next.
pub struct StreamingIterator<'a> {
  handle: ffi::CzStreamingIterator,
  reader: PhantomData<&'a ArchiveReader>,
}

impl<'a> std::iter::Iterator for StreamingIterator<'a> {
  type Item = (WriteEntry, ArchiveEntryReader<'a>);

  /// `ArchiveReader.StreamingIterator.next()`.
  fn next(&mut self) -> Option<Self::Item> {
    let outcome = self.handle.next();

    outcome.is_some().then(|| {
      (
        WriteEntry {
          handle: outcome.write_entry(),
        },
        ArchiveEntryReader {
          handle: outcome.archive_entry_reader(),
          reader: PhantomData,
        },
      )
    })
  }
}
