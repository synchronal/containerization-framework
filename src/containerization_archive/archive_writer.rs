//! `ArchiveWriter`, and the options its file initializer defaults.

use super::ArchiveWriterConfiguration;
use super::Filter;
use super::Format;
use super::Options;
use super::WriteEntry;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::fmt;
use std::os::fd::RawFd;
use std::path::Path;

/// The defaulted arguments of `ArchiveWriter(format:filter:options:locales:file:)`.
/// [`Default`] is Swift's defaults: no options, and
/// `ArchiveWriterConfiguration.defaultLocales`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveWriterOptions {
  pub options: Vec<Options>,
  pub locales: Vec<String>,
}

impl Default for ArchiveWriterOptions {
  fn default() -> Self {
    Self {
      options: Vec::new(),
      locales: ArchiveWriterConfiguration::default_locales(),
    }
  }
}

/// `ArchiveWriter`.
pub struct ArchiveWriter {
  pub(crate) handle: ffi::CzArchiveWriter,
}

impl fmt::Debug for ArchiveWriter {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("ArchiveWriter")
      .finish_non_exhaustive()
  }
}

// Swift's `ArchiveWriter` is a class that isn't `Sendable`; Rust's isn't
// `Sync`, so one call runs at a time.
unsafe impl Send for ArchiveWriter {}

impl ArchiveWriter {
  /// `ArchiveWriter(configuration:)`.
  pub fn new(configuration: &ArchiveWriterConfiguration) -> Result<Self, Error> {
    let outcome = platform::outcome(
      ffi::cz_archive_writer_new(configuration.clone()),
      format!("make a {} archive writer", configuration.format.raw_value()),
    )?;

    Ok(Self {
      handle: outcome.archive_writer(),
    })
  }

  /// `ArchiveWriter(format:filter:options:locales:file:)`.
  pub fn with_file(format: Format, filter: Filter, options: ArchiveWriterOptions, file: &Path) -> Result<Self, Error> {
    let configuration = ArchiveWriterConfiguration {
      format,
      filter,
      options: options.options,
      locales: options.locales,
    };
    let outcome = platform::outcome(
      ffi::cz_archive_writer_with_file(configuration, platform::path(file)?),
      format!("write a {} archive to {}", format.raw_value(), file.display()),
    )?;

    Ok(Self {
      handle: outcome.archive_writer(),
    })
  }

  /// `ArchiveWriter.open(file:)`.
  pub fn open(&self, file: &Path) -> Result<(), Error> {
    platform::outcome(
      self.handle.open(platform::path(file)?),
      format!("open {} for an archive", file.display()),
    )
    .map(drop)
  }

  /// `ArchiveWriter.open(fileDescriptor:)`. The caller keeps `file_descriptor`
  /// open while the writer writes to it, and closes it afterwards.
  pub fn open_with_file_descriptor(&self, file_descriptor: RawFd) -> Result<(), Error> {
    platform::outcome(
      self.handle.open_with_file_descriptor(file_descriptor),
      format!("open file descriptor {file_descriptor} for an archive"),
    )
    .map(drop)
  }

  /// `ArchiveWriter.finishEncoding()`.
  pub fn finish_encoding(&self) -> Result<(), Error> {
    platform::outcome(self.handle.finish_encoding(), "finish an archive").map(drop)
  }

  /// `ArchiveWriter.makeTransactionWriter()`.
  pub fn make_transaction_writer(&self) -> ArchiveWriterTransaction {
    ArchiveWriterTransaction {
      handle: self.handle.make_transaction_writer(),
    }
  }

  /// `ArchiveWriter.writeEntry(entry:data:)`. `None` writes an entry with no
  /// data, such as a directory.
  pub fn write_entry(&self, entry: &WriteEntry, data: Option<&[u8]>) -> Result<(), Error> {
    platform::outcome(
      self.handle.write_entry(
        entry.handle.duplicate(),
        data.is_some(),
        data.unwrap_or_default().to_vec(),
      ),
      format!("write {} to an archive", entry.path().unwrap_or_default()),
    )
    .map(drop)
  }

  /// `ArchiveWriter.archiveDirectory(_:)`.
  pub fn archive_directory(&self, dir: &Path) -> Result<(), Error> {
    platform::outcome(
      self.handle.archive_directory(platform::path(dir)?),
      format!("archive {}", dir.display()),
    )
    .map(drop)
  }

  /// `ArchiveWriter.archive(_:base:)`.
  pub fn archive(&self, paths: &[&Path], base: &Path) -> Result<(), Error> {
    platform::outcome(
      self.handle.archive(
        paths
          .iter()
          .map(|path| platform::path(path).map(str::to_string))
          .collect::<Result<_, _>>()?,
        platform::path(base)?,
      ),
      format!("archive paths under {}", base.display()),
    )
    .map(drop)
  }
}

/// `ArchiveWriterTransaction`.
pub struct ArchiveWriterTransaction {
  handle: ffi::CzArchiveWriterTransaction,
}

impl fmt::Debug for ArchiveWriterTransaction {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("ArchiveWriterTransaction")
      .finish_non_exhaustive()
  }
}

// Swift's `ArchiveWriterTransaction` is a class that isn't `Sendable`; Rust's
// isn't `Sync`, so one call runs at a time.
unsafe impl Send for ArchiveWriterTransaction {}

impl ArchiveWriterTransaction {
  /// `ArchiveWriterTransaction.writeHeader(entry:)`.
  pub fn write_header(&self, entry: &WriteEntry) -> Result<(), Error> {
    platform::outcome(
      self.handle.write_header(entry.handle.duplicate()),
      format!("write the header of {} to an archive", entry.path().unwrap_or_default()),
    )
    .map(drop)
  }

  /// `ArchiveWriterTransaction.writeChunk(data:)`.
  pub fn write_chunk(&self, data: &[u8]) -> Result<(), Error> {
    platform::outcome(self.handle.write_chunk(data.to_vec()), "write a chunk to an archive").map(drop)
  }

  /// `ArchiveWriterTransaction.finish()`.
  pub fn finish(&self) -> Result<(), Error> {
    platform::outcome(self.handle.finish(), "finish an archive entry").map(drop)
  }
}
