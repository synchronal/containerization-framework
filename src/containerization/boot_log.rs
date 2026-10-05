use std::os::fd::RawFd;
use std::path::PathBuf;

/// `BootLog`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BootLog {
  /// `BootLog.file(path:append:)`.
  File { path: PathBuf, append: bool },
  /// `BootLog.fileHandle(_:)`. Swift writes to a duplicate, so the caller keeps
  /// this descriptor.
  FileHandle(RawFd),
}

impl BootLog {
  /// `BootLog.file(path:)`, appending.
  pub fn file(path: impl Into<PathBuf>) -> Self {
    Self::File {
      path: path.into(),
      append: true,
    }
  }
}
