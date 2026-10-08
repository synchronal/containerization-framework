//! `File`.

use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::fmt;
use std::path::Path;

/// `FileInfo`. Each getter reads Swift's.
pub struct FileInfo {
  handle: ffi::CzFileInfo,
}

impl fmt::Debug for FileInfo {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.debug_struct("FileInfo").finish_non_exhaustive()
  }
}

// Swift's `FileInfo` is `Sendable`.
unsafe impl Send for FileInfo {}
unsafe impl Sync for FileInfo {}

/// `File.info(_:)`, which doesn't follow a symlink at `path`.
pub fn info(path: &Path) -> Result<FileInfo, Error> {
  platform::outcome(
    ffi::cz_file_info(platform::path(path)?),
    format!("read {}'s file info", path.display()),
  )
  .map(|outcome| FileInfo {
    handle: outcome.file_info(),
  })
}

impl FileInfo {
  /// `mode`.
  pub fn mode(&self) -> u16 {
    self.handle.mode()
  }

  /// `uid`.
  pub fn uid(&self) -> i64 {
    self.handle.uid()
  }

  /// `gid`.
  pub fn gid(&self) -> i64 {
    self.handle.gid()
  }

  /// `dev`.
  pub fn dev(&self) -> i64 {
    self.handle.dev()
  }

  /// `ino`.
  pub fn ino(&self) -> i64 {
    self.handle.ino()
  }

  /// `size`.
  pub fn size(&self) -> i64 {
    self.handle.size()
  }

  /// `path`.
  pub fn path(&self) -> String {
    self.handle.path()
  }

  /// `isDirectory`.
  pub fn is_directory(&self) -> bool {
    self.handle.is_directory()
  }

  /// `isPipe`.
  pub fn is_pipe(&self) -> bool {
    self.handle.is_pipe()
  }

  /// `isSocket`.
  pub fn is_socket(&self) -> bool {
    self.handle.is_socket()
  }

  /// `isLink`.
  pub fn is_link(&self) -> bool {
    self.handle.is_link()
  }

  /// `isRegularFile`.
  pub fn is_regular_file(&self) -> bool {
    self.handle.is_regular_file()
  }

  /// `isBlock`.
  pub fn is_block(&self) -> bool {
    self.handle.is_block()
  }

  /// `isChar`.
  pub fn is_char(&self) -> bool {
    self.handle.is_char()
  }
}
