use crate::platform::ffi;

/// `FileInfo`, which [`super::file::info`] reads. Each getter reads Swift's.
pub struct FileInfo {
  pub(super) handle: ffi::CzFileInfo,
}

opaque_debug!(FileInfo);

// Swift's `FileInfo` is `Sendable`.
unsafe impl Send for FileInfo {}
unsafe impl Sync for FileInfo {}

impl FileInfo {
  /// `FileInfo.mode`, the file's type and permission bits.
  pub fn mode(&self) -> u16 {
    self.handle.mode()
  }

  /// `FileInfo.uid`, the owner's user id.
  pub fn uid(&self) -> i64 {
    self.handle.uid()
  }

  /// `FileInfo.gid`, the owner's group id.
  pub fn gid(&self) -> i64 {
    self.handle.gid()
  }

  /// `FileInfo.dev`, the id of the filesystem holding the file.
  pub fn dev(&self) -> i64 {
    self.handle.dev()
  }

  /// `FileInfo.ino`, the file's inode number.
  pub fn ino(&self) -> i64 {
    self.handle.ino()
  }

  /// `FileInfo.size`, in bytes.
  pub fn size(&self) -> i64 {
    self.handle.size()
  }

  /// `FileInfo.path`.
  pub fn path(&self) -> String {
    self.handle.path()
  }

  /// `FileInfo.isDirectory`.
  pub fn is_directory(&self) -> bool {
    self.handle.is_directory()
  }

  /// `FileInfo.isPipe`, which is true for a named pipe.
  pub fn is_pipe(&self) -> bool {
    self.handle.is_pipe()
  }

  /// `FileInfo.isSocket`.
  pub fn is_socket(&self) -> bool {
    self.handle.is_socket()
  }

  /// `FileInfo.isLink`, which is true for a symbolic link.
  pub fn is_link(&self) -> bool {
    self.handle.is_link()
  }

  /// `FileInfo.isRegularFile`.
  pub fn is_regular_file(&self) -> bool {
    self.handle.is_regular_file()
  }

  /// `FileInfo.isBlock`, which is true for a block device.
  pub fn is_block(&self) -> bool {
    self.handle.is_block()
  }

  /// `FileInfo.isChar`, which is true for a character device.
  pub fn is_char(&self) -> bool {
    self.handle.is_char()
  }
}
