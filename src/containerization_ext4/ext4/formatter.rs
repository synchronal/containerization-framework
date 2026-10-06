//! `EXT4.Formatter`, and the options structs its methods take.

use super::JournalConfig;
use crate::containerization_archive::ArchiveReader;
use crate::containerization_archive::Filter;
use crate::containerization_archive::Format;
use crate::containerization_ext4::FileTimestamps;
use crate::containerization_extras::ProgressHandler;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::collections::BTreeMap;
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
  pub ts: FileTimestamps,
  pub buf: Option<&'a [u8]>,
  pub uid: Option<u32>,
  pub gid: Option<u32>,
  pub xattrs: Option<BTreeMap<String, Vec<u8>>>,
  pub recursion: bool,
}

/// `EXT4.Formatter.unpack(source:format:compression:progress:)`'s defaulted
/// arguments. [`Default`] is Swift's defaults.
pub struct UnpackOptions {
  pub format: Format,
  pub compression: Filter,
  pub progress: Option<ProgressHandler>,
}

impl Default for UnpackOptions {
  fn default() -> Self {
    Self {
      format: Format::PaxRestricted,
      compression: Filter::Gzip,
      progress: None,
    }
  }
}

/// `EXT4.Formatter`.
pub struct Formatter {
  handle: ffi::CzExt4Formatter,
}

// Swift's `Formatter` is a class that isn't `Sendable`; Rust's isn't `Sync`,
// so one call runs at a time.
unsafe impl Send for Formatter {}

impl Formatter {
  /// `EXT4.Formatter(_:blockSize:minDiskSize:journal:)`.
  pub fn new(device_path: &Path, options: FormatterOptions) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_ext4_formatter_new(&device_path.display().to_string(), options),
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
        .link(&link.display().to_string(), &target.display().to_string()),
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
        .unlink(&path.display().to_string(), directory_whiteout),
      format!("unlink {}", path.display()),
    )
    .map(drop)
  }

  /// `EXT4.Formatter.create(path:link:mode:ts:buf:uid:gid:xattrs:recursion:)`.
  pub fn create(&self, path: &Path, mode: u16, options: CreateOptions<'_>) -> Result<(), Error> {
    let ts = options.ts;
    let xattrs = options.xattrs.as_ref();

    // The xattrs' values cross joined into one, with each one's length.
    platform::outcome(
      self.handle.create(
        &path.display().to_string(),
        options.link.map(|link| link.display().to_string()),
        mode,
        platform::seconds(ts.access),
        platform::seconds(ts.modification),
        platform::seconds(ts.creation),
        platform::seconds(ts.now),
        options.buf.is_some(),
        options.buf.unwrap_or_default().to_vec(),
        options.uid,
        options.gid,
        xattrs.is_some(),
        xattrs
          .map(|xattrs| xattrs.keys().cloned().collect())
          .unwrap_or_default(),
        xattrs
          .map(|xattrs| xattrs.values().map(|value| value.len() as u64).collect())
          .unwrap_or_default(),
        xattrs
          .map(|xattrs| xattrs.values().flatten().copied().collect())
          .unwrap_or_default(),
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
        &source.display().to_string(),
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
  pub fn unpack_reader(&self, reader: &ArchiveReader, progress: Option<ProgressHandler>) -> Result<(), Error> {
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
  pub fn scan_archive_headers(format: Format, filter: Filter, file: &Path) -> Result<(i64, isize), Error> {
    platform::outcome(
      ffi::cz_ext4_formatter_scan_archive_headers(format.raw_value(), filter.raw_value(), &file.display().to_string()),
      format!("scan the archive {}", file.display()),
    )
    .map(|outcome| (outcome.scanned_size(), outcome.scanned_items()))
  }
}
