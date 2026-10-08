//! `EXT4.Formatter`, and the options structs its methods take.

use super::JournalConfig;
use crate::containerization_archive;
use crate::containerization_ext4;
use crate::containerization_extras;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;

/// `EXT4.Formatter(_:blockSize:minDiskSize:journal:)`'s defaulted arguments.
/// [`Default`] is Swift's defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FormatterOptions {
  pub block_size: u32,
  pub min_disk_size: u64,
  pub journal: Option<JournalConfig>,
}

impl Default for FormatterOptions {
  fn default() -> Self {
    Self {
      block_size: 4096,
      min_disk_size: 256 * 1024,
      journal: None,
    }
  }
}

/// `EXT4.Formatter.create(path:link:mode:...)`'s defaulted arguments, after
/// the mode. [`Default`] is Swift's defaults. Swift's `buf` is a
/// `ReadableStream`; here it is the file's bytes, which Swift reads through an
/// `InputStream`. Swift's `fileBuffer` is a raw scratch buffer, and is left
/// out.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CreateOptions<'a> {
  pub link: Option<PathBuf>,
  pub ts: containerization_ext4::FileTimestamps,
  pub buf: Option<&'a [u8]>,
  pub uid: Option<u32>,
  pub gid: Option<u32>,
  pub xattrs: Option<BTreeMap<String, Vec<u8>>>,
  pub recursion: bool,
}

/// `EXT4.Formatter.unpack(source:format:compression:progress:)`'s defaulted
/// arguments. [`Default`] is Swift's defaults.
pub struct UnpackOptions {
  pub format: containerization_archive::Format,
  pub compression: containerization_archive::Filter,
  pub progress: Option<containerization_extras::ProgressHandler>,
}

impl Default for UnpackOptions {
  fn default() -> Self {
    Self {
      format: containerization_archive::Format::PaxRestricted,
      compression: containerization_archive::Filter::Gzip,
      progress: None,
    }
  }
}

/// Leaves out the progress handler, which is a closure.
impl fmt::Debug for UnpackOptions {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("UnpackOptions")
      .field("format", &self.format)
      .field("compression", &self.compression)
      .finish_non_exhaustive()
  }
}

/// `EXT4.Formatter`.
pub struct Formatter {
  handle: ffi::CzExt4Formatter,
}

impl fmt::Debug for Formatter {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.debug_struct("Formatter").finish_non_exhaustive()
  }
}

// Swift's `Formatter` is a class that isn't `Sendable`; Rust's isn't `Sync`,
// so one call runs at a time.
unsafe impl Send for Formatter {}

impl Formatter {
  /// `EXT4.Formatter(_:blockSize:minDiskSize:journal:)`.
  pub fn new(device_path: &Path, options: FormatterOptions) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_ext4_formatter_new(platform::path(device_path)?, options),
      format!("format {}", device_path.display()),
    )
    .map(|outcome| Self {
      handle: outcome.ext4_formatter(),
    })
  }

  /// `EXT4.Formatter.link(link:target:)`.
  pub fn link(&self, link: &Path, target: &Path) -> Result<(), Error> {
    platform::outcome(
      self
        .handle
        .link(platform::path(link)?, platform::path(target)?),
      format!("link {} to {}", link.display(), target.display()),
    )
    .map(drop)
  }

  /// `EXT4.Formatter.unlink(path:directoryWhiteout:)`. Swift's
  /// `directoryWhiteout` defaults to `false`.
  pub fn unlink(&self, path: &Path, directory_whiteout: bool) -> Result<(), Error> {
    platform::outcome(
      self
        .handle
        .unlink(platform::path(path)?, directory_whiteout),
      format!("unlink {}", path.display()),
    )
    .map(drop)
  }

  /// `EXT4.Formatter.create(path:link:mode:ts:buf:uid:gid:xattrs:recursion:)`.
  pub fn create(&self, path: &Path, mode: u16, options: CreateOptions<'_>) -> Result<(), Error> {
    let ts = options.ts;
    let has_xattrs = options.xattrs.is_some();
    let (xattr_names, xattr_lengths, xattr_values) = options
      .xattrs
      .as_ref()
      .map(platform::xattrs)
      .unwrap_or_default();

    platform::outcome(
      self.handle.create(
        platform::path(path)?,
        options
          .link
          .as_deref()
          .map(platform::path)
          .transpose()?
          .map(str::to_string),
        mode,
        platform::seconds(ts.access),
        platform::seconds(ts.modification),
        platform::seconds(ts.creation),
        platform::seconds(ts.now),
        options.buf.is_some(),
        options.buf.unwrap_or_default().to_vec(),
        options.uid,
        options.gid,
        has_xattrs,
        xattr_names,
        xattr_lengths,
        xattr_values,
        options.recursion,
      ),
      format!("create {}", path.display()),
    )
    .map(drop)
  }

  /// `EXT4.Formatter.close()`.
  pub fn close(&self) -> Result<(), Error> {
    platform::outcome(self.handle.close(), "close a filesystem").map(drop)
  }

  /// `EXT4.Formatter.unpack(source:format:compression:progress:)`.
  pub fn unpack(&self, source: &Path, options: UnpackOptions) -> Result<(), Error> {
    platform::outcome(
      self.handle.unpack(
        platform::path(source)?,
        options.format.raw_value(),
        options.compression.raw_value(),
        platform::Progress(options.progress),
      ),
      format!("unpack {}", source.display()),
    )
    .map(drop)
  }

  /// `EXT4.Formatter.unpack(reader:progress:)`. Swift's `progress` defaults
  /// to `nil`.
  pub fn unpack_reader(
    &self,
    reader: &containerization_archive::ArchiveReader,
    progress: Option<containerization_extras::ProgressHandler>,
  ) -> Result<(), Error> {
    platform::outcome(
      self
        .handle
        .unpack_reader(reader.handle.duplicate(), platform::Progress(progress)),
      "unpack an archive",
    )
    .map(drop)
  }

  /// `EXT4.Formatter.scanArchiveHeaders(format:filter:file:)`: the total
  /// size of the regular files, and the number of entries.
  pub fn scan_archive_headers(
    format: containerization_archive::Format,
    filter: containerization_archive::Filter,
    file: &Path,
  ) -> Result<(i64, isize), Error> {
    platform::outcome(
      ffi::cz_ext4_formatter_scan_archive_headers(format.raw_value(), filter.raw_value(), platform::path(file)?),
      format!("scan the archive {}", file.display()),
    )
    .map(|outcome| (outcome.scanned_size(), outcome.scanned_items()))
  }
}
