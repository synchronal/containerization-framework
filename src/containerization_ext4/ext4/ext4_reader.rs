//! `EXT4.EXT4Reader`, and the options struct `readFile` takes.

use super::ExtendedAttribute;
use super::Inode;
use super::InodeNumber;
use super::SuperBlock;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::fmt;
use std::path::Path;

/// `EXT4.EXT4Reader.readFile(at:offset:count:followSymlinks:)`'s defaulted
/// arguments. [`Default`] is Swift's defaults: from the start, to the end,
/// following symlinks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadFileOptions {
  pub offset: u64,
  pub count: Option<usize>,
  pub follow_symlinks: bool,
}

impl Default for ReadFileOptions {
  fn default() -> Self {
    Self {
      offset: 0,
      count: None,
      follow_symlinks: true,
    }
  }
}

/// `EXT4.EXT4Reader`.
pub struct EXT4Reader {
  handle: ffi::CzExt4Reader,
}

impl fmt::Debug for EXT4Reader {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.debug_struct("EXT4Reader").finish_non_exhaustive()
  }
}

// Swift's `EXT4Reader` is a class that isn't `Sendable`; Rust's isn't `Sync`,
// so one call runs at a time.
unsafe impl Send for EXT4Reader {}

impl EXT4Reader {
  /// `EXT4.EXT4Reader(blockDevice:)`.
  pub fn new(block_device: &Path) -> Result<Self, Error> {
    let outcome = platform::outcome(
      ffi::cz_ext4_reader_new(&block_device.display().to_string()),
      format!("read the filesystem in {}", block_device.display()),
    )?;

    Ok(Self {
      handle: outcome.ext4_reader(),
    })
  }

  /// `EXT4.EXT4Reader.superBlock`.
  pub fn super_block(&self) -> SuperBlock {
    platform::from_bytes(&self.handle.super_block())
  }

  /// `EXT4.EXT4Reader.exists(_:followSymlinks:)`. Swift's `followSymlinks`
  /// defaults to `true`.
  pub fn exists(&self, path: &Path, follow_symlinks: bool) -> bool {
    self
      .handle
      .exists(&path.display().to_string(), follow_symlinks)
  }

  /// `EXT4.EXT4Reader.stat(_:followSymlinks:)`. Swift's `followSymlinks`
  /// defaults to `true`.
  pub fn stat(&self, path: &Path, follow_symlinks: bool) -> Result<(InodeNumber, Inode), Error> {
    platform::outcome(
      self
        .handle
        .stat(&path.display().to_string(), follow_symlinks),
      format!("stat {}", path.display()),
    )
    .map(|outcome| (outcome.inode_number(), outcome.inode()))
  }

  /// `EXT4.EXT4Reader.listDirectory(_:)`: the entries' names, sorted, without
  /// `.` and `..`.
  pub fn list_directory(&self, path: &Path) -> Result<Vec<String>, Error> {
    platform::outcome(
      self.handle.list_directory(&path.display().to_string()),
      format!("list {}", path.display()),
    )
    .map(|outcome| outcome.strings())
  }

  /// `EXT4.EXT4Reader.readFile(at:offset:count:followSymlinks:)`.
  pub fn read_file(&self, at: &Path, options: ReadFileOptions) -> Result<Vec<u8>, Error> {
    platform::outcome(
      self.handle.read_file(
        &at.display().to_string(),
        options.offset,
        options.count,
        options.follow_symlinks,
      ),
      format!("read {}", at.display()),
    )
    .map(|outcome| outcome.bytes())
  }

  /// `EXT4.EXT4Reader.export(archive:)`.
  pub fn export(&self, archive: &Path) -> Result<(), Error> {
    platform::outcome(
      self.handle.export(&archive.display().to_string()),
      format!("export the filesystem to {}", archive.display()),
    )
    .map(drop)
  }

  /// `EXT4.EXT4Reader.readInlineExtendedAttributes(from:)`. Like Swift's, it
  /// traps on a buffer shorter than its 4-byte header.
  pub fn read_inline_extended_attributes(buffer: &[u8]) -> Result<Vec<ExtendedAttribute>, Error> {
    attributes(
      ffi::cz_ext4_reader_read_inline_extended_attributes(buffer.to_vec()),
      "read inline extended attributes",
    )
  }

  /// `EXT4.EXT4Reader.readBlockExtendedAttributes(from:)`. Like Swift's, it
  /// traps on a buffer shorter than its 4-byte header.
  pub fn read_block_extended_attributes(buffer: &[u8]) -> Result<Vec<ExtendedAttribute>, Error> {
    attributes(
      ffi::cz_ext4_reader_read_block_extended_attributes(buffer.to_vec()),
      "read a block's extended attributes",
    )
  }
}

fn attributes(outcome: ffi::CzOutcome, action: &str) -> Result<Vec<ExtendedAttribute>, Error> {
  platform::outcome(outcome, action).map(|outcome| {
    (0..outcome.len())
      .map(|_| ExtendedAttribute::opaque())
      .collect()
  })
}
