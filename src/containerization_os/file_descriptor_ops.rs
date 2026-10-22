//! `FileDescriptorOps`, symlink-safe filesystem operations relative to a
//! directory descriptor.
//!
//! Each function borrows the descriptors it is given and never closes them. A
//! descriptor that a function opens is returned as an [`OwnedFd`], which the
//! caller owns. A name is one path component, and a relative path must not be
//! absolute or contain `..`.
//!
//! Rust has no overloading, so where Swift overloads a name, the Rust name
//! adds a suffix for the argument label that tells the overloads apart, as in
//! [`status_of`] and [`status_relative_path`].
//!
//! A function that throws a `FileDescriptorOps.Error` in Swift fails with
//! [`error::Error::FileDescriptorOps`], which carries it as an [`Error`].

use crate::error;
use crate::platform;
use crate::platform::ffi;
use std::ffi::OsStr;
use std::fmt;
use std::os::fd::AsRawFd;
use std::os::fd::BorrowedFd;
use std::os::fd::OwnedFd;
use std::path::Path;
use std::path::PathBuf;

/// `FileDescriptorOps.maximumSymlinksFollowed`, the most symlinks that
/// [`SymlinkPolicy::FollowBeneath`] follows while resolving one path.
pub const MAXIMUM_SYMLINKS_FOLLOWED: usize = 40;

/// `FileDescriptorOps.Error`. Its [`fmt::Display`] is Swift's `description`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
  /// The path is absolute, or contains a `..` component.
  InvalidRelativePath,
  /// An intermediate path component is missing or is not a directory.
  InvalidPathComponent,
  /// The entry is a symlink, which these operations don't follow.
  CannotFollowSymlink,
  /// The entry doesn't exist.
  NotFound,
  /// The entry already exists.
  AlreadyExists,
  /// The entry has a type that the operation can't use.
  Conflict(EntryType),
  /// A system call failed. It holds what was being done and the `errno`.
  SystemError(String, i32),
}

impl fmt::Display for Error {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InvalidRelativePath => formatter.write_str("invalid relative path supplied to file descriptor operation"),
      Self::InvalidPathComponent => {
        formatter.write_str("an intermediate path component is missing or is not a directory")
      }
      Self::CannotFollowSymlink => formatter.write_str("cannot follow a symlink in a file descriptor operation"),
      Self::NotFound => formatter.write_str("no such entry in file descriptor operation"),
      Self::AlreadyExists => formatter.write_str("entry already exists in file descriptor operation"),
      Self::Conflict(entry_type) => {
        write!(
          formatter,
          "a {entry_type} entry is in the way of a file descriptor operation"
        )
      }
      Self::SystemError(operation, errno) => write!(formatter, "{operation} returned error: {errno}"),
    }
  }
}

impl std::error::Error for Error {}

/// `FileDescriptorOps.EntryType`, the type of a directory entry. Its
/// [`fmt::Display`] is the name of Swift's case.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EntryType {
  Regular,
  Directory,
  Symlink,
  /// Anything else, such as a device node, a named pipe or a socket.
  Other,
}

impl fmt::Display for EntryType {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(match self {
      Self::Regular => "regular",
      Self::Directory => "directory",
      Self::Symlink => "symlink",
      Self::Other => "other",
    })
  }
}

same_variants!(
  EntryType,
  ffi::FdOpsEntryType {
    Regular,
    Directory,
    Symlink,
    Other
  }
);

/// `FileDescriptorOps.FileStatus`, an entry's metadata as `stat(2)` reports
/// it without following a symlink.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FileStatus {
  pub r#type: EntryType,
  /// The permission bits, including setuid, setgid and sticky.
  pub permissions: u16,
  pub size: i64,
  pub user_id: u32,
  pub group_id: u32,
  pub modification_seconds: i64,
  pub modification_nanoseconds: i64,
}

/// `FileDescriptorOps.SymlinkPolicy`, what to do about a symlink found while
/// resolving a path.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SymlinkPolicy {
  /// Refuse any symlink in the path, including the last component.
  Refuse,
  /// Follow a symlink, but only to a place beneath the directory the walk
  /// started from, and through at most [`MAXIMUM_SYMLINKS_FOLLOWED`] links.
  FollowBeneath,
}

same_variants!(SymlinkPolicy, ffi::FdOpsSymlinkPolicy { Refuse, FollowBeneath });

/// The defaulted arguments of
/// `FileDescriptorOps.mkdir(_:_:permissions:makeIntermediates:completion:)`.
/// [`Default`] uses Swift's defaults: no permissions, so 0o755, and no
/// intermediate directories.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MkdirOptions {
  /// The permissions of each directory created. `None` uses 0o755.
  pub permissions: Option<u16>,
  /// Also create missing intermediate directories.
  pub make_intermediates: bool,
}

/// `FileDescriptorOps.entryType(_:_:)`.
pub fn entry_type(fd: BorrowedFd<'_>, name: &OsStr) -> Result<Option<EntryType>, error::Error> {
  platform::outcome(
    ffi::cz_fd_ops_entry_type(fd.as_raw_fd(), component(name)?),
    format!("read the type of {}", name.display()),
  )
  .map(|outcome| outcome.optional_fd_ops_entry_type())
}

/// `FileDescriptorOps.openDirectory(_:_:)`.
pub fn open_directory(fd: BorrowedFd<'_>, name: &OsStr) -> Result<OwnedFd, error::Error> {
  platform::owned_fd(
    ffi::cz_fd_ops_open_directory(fd.as_raw_fd(), component(name)?),
    format!("open the directory {}", name.display()),
  )
}

/// `FileDescriptorOps.makeDirectory(_:_:permissions:)`. `None` uses Swift's
/// default of 0o755.
pub fn make_directory(fd: BorrowedFd<'_>, name: &OsStr, permissions: Option<u16>) -> Result<(), error::Error> {
  platform::outcome(
    ffi::cz_fd_ops_make_directory(
      fd.as_raw_fd(),
      component(name)?,
      permissions.is_some(),
      permissions.unwrap_or_default(),
    ),
    format!("make the directory {}", name.display()),
  )
  .map(drop)
}

/// `FileDescriptorOps.unlink(_:_:)`.
pub fn unlink(fd: BorrowedFd<'_>, name: &OsStr) -> Result<(), error::Error> {
  platform::outcome(
    ffi::cz_fd_ops_unlink(fd.as_raw_fd(), component(name)?),
    format!("unlink {}", name.display()),
  )
  .map(drop)
}

/// `FileDescriptorOps.status(of:)`.
pub fn status_of(fd: BorrowedFd<'_>) -> Result<FileStatus, error::Error> {
  platform::outcome(
    ffi::cz_fd_ops_status_of(fd.as_raw_fd()),
    "read the status of a descriptor",
  )
  .map(|outcome| outcome.fd_ops_file_status())
}

/// `FileDescriptorOps.status(_:_:)`.
pub fn status(fd: BorrowedFd<'_>, name: &OsStr) -> Result<Option<FileStatus>, error::Error> {
  platform::outcome(
    ffi::cz_fd_ops_status(fd.as_raw_fd(), component(name)?),
    format!("read the status of {}", name.display()),
  )
  .map(|outcome| outcome.optional_fd_ops_file_status())
}

/// `FileDescriptorOps.status(_:relativePath:symlinks:)`.
pub fn status_relative_path(
  fd: BorrowedFd<'_>,
  relative_path: &Path,
  symlinks: SymlinkPolicy,
) -> Result<Option<FileStatus>, error::Error> {
  platform::outcome(
    ffi::cz_fd_ops_status_relative_path(fd.as_raw_fd(), platform::path(relative_path)?, symlinks.into()),
    format!("read the status of {}", relative_path.display()),
  )
  .map(|outcome| outcome.optional_fd_ops_file_status())
}

/// `FileDescriptorOps.openFile(_:_:)`.
pub fn open_file(fd: BorrowedFd<'_>, name: &OsStr) -> Result<OwnedFd, error::Error> {
  platform::owned_fd(
    ffi::cz_fd_ops_open_file(fd.as_raw_fd(), component(name)?),
    format!("open the file {}", name.display()),
  )
}

/// `FileDescriptorOps.openFile(_:relativePath:symlinks:)`.
pub fn open_file_relative_path(
  fd: BorrowedFd<'_>,
  relative_path: &Path,
  symlinks: SymlinkPolicy,
) -> Result<OwnedFd, error::Error> {
  platform::owned_fd(
    ffi::cz_fd_ops_open_file_relative_path(fd.as_raw_fd(), platform::path(relative_path)?, symlinks.into()),
    format!("open the file {}", relative_path.display()),
  )
}

/// `FileDescriptorOps.createFile(_:_:permissions:)`. `None` uses Swift's
/// default of 0o644.
pub fn create_file(fd: BorrowedFd<'_>, name: &OsStr, permissions: Option<u16>) -> Result<OwnedFd, error::Error> {
  platform::owned_fd(
    ffi::cz_fd_ops_create_file(
      fd.as_raw_fd(),
      component(name)?,
      permissions.is_some(),
      permissions.unwrap_or_default(),
    ),
    format!("create the file {}", name.display()),
  )
}

/// `FileDescriptorOps.makeSymlink(_:_:target:)`.
pub fn make_symlink(fd: BorrowedFd<'_>, name: &OsStr, target: &str) -> Result<(), error::Error> {
  platform::outcome(
    ffi::cz_fd_ops_make_symlink(fd.as_raw_fd(), component(name)?, target),
    format!("make the symlink {}", name.display()),
  )
  .map(drop)
}

/// `FileDescriptorOps.readSymlink(_:_:)`.
pub fn read_symlink(fd: BorrowedFd<'_>, name: &OsStr) -> Result<String, error::Error> {
  platform::outcome(
    ffi::cz_fd_ops_read_symlink(fd.as_raw_fd(), component(name)?),
    format!("read the symlink {}", name.display()),
  )
  .map(|outcome| outcome.text())
}

/// `FileDescriptorOps.unlinkRecursive(_:filename:)`.
pub fn unlink_recursive(fd: BorrowedFd<'_>, filename: &OsStr) -> Result<(), error::Error> {
  platform::outcome(
    ffi::cz_fd_ops_unlink_recursive(fd.as_raw_fd(), component(filename)?),
    format!("remove {}", filename.display()),
  )
  .map(drop)
}

/// `FileDescriptorOps.enumerate(_:_:)`. `body` gets each entry's path relative
/// to `fd`, its type, and the directory that holds it, which stays open only
/// for the call. An error that `body` returns stops the walk and is returned
/// as it is. Swift reads the directory through a duplicate of `fd`, which
/// shares its offset, so enumerating the same descriptor again finds nothing.
pub fn enumerate(
  fd: BorrowedFd<'_>,
  body: impl FnMut(&Path, EntryType, BorrowedFd<'_>) -> Result<(), error::Error> + 'static,
) -> Result<(), error::Error> {
  platform::enumerating(
    body,
    |body| ffi::cz_fd_ops_enumerate(fd.as_raw_fd(), body),
    "enumerate a directory",
  )
  .map(drop)
}

/// `FileDescriptorOps.getCanonicalPath(_:)`.
pub fn get_canonical_path(fd: BorrowedFd<'_>) -> Result<PathBuf, error::Error> {
  platform::outcome(
    ffi::cz_fd_ops_get_canonical_path(fd.as_raw_fd()),
    "read the canonical path of a descriptor",
  )
  .map(|outcome| PathBuf::from(outcome.text()))
}

/// `FileDescriptorOps.mkdir(_:_:permissions:makeIntermediates:completion:)`.
/// `completion` gets the directory, which stays open only for the call. Swift's
/// default `completion` does nothing, as `|_| Ok(())` does. An error that
/// `completion` returns is returned as it is.
pub fn mkdir(
  fd: BorrowedFd<'_>,
  relative_path: &Path,
  options: MkdirOptions,
  completion: impl FnOnce(BorrowedFd<'_>) -> Result<(), error::Error> + 'static,
) -> Result<(), error::Error> {
  let path = platform::path(relative_path)?;

  platform::with_descriptor(
    completion,
    |completion| {
      ffi::cz_fd_ops_mkdir(
        fd.as_raw_fd(),
        path,
        options.permissions.is_some(),
        options.permissions.unwrap_or_default(),
        options.make_intermediates,
        completion,
      )
    },
    format!("make the directory {}", relative_path.display()),
  )
  .map(drop)
}

/// `name` as Swift takes it. Swift refuses a name that isn't one component.
fn component(name: &OsStr) -> Result<&str, error::Error> {
  platform::path(Path::new(name))
}
