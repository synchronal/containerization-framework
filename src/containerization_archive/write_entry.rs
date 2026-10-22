use super::ArchiveWriter;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::time::SystemTime;

raw_values!(
  /// Foundation's `URLFileResourceType`, as `WriteEntry.fileType` reads and
  /// writes it.
  URLFileResourceType {
    NamedPipe = "NSURLFileResourceTypeNamedPipe",
    CharacterSpecial = "NSURLFileResourceTypeCharacterSpecial",
    Directory = "NSURLFileResourceTypeDirectory",
    BlockSpecial = "NSURLFileResourceTypeBlockSpecial",
    Regular = "NSURLFileResourceTypeRegular",
    SymbolicLink = "NSURLFileResourceTypeSymbolicLink",
    Socket = "NSURLFileResourceTypeSocket",
    Unknown = "NSURLFileResourceTypeUnknown",
  }
);

/// `WriteEntry`. Swift's is a class, so each property is read from and
/// written to Swift.
///
/// Swift's date setters drop any fraction of a second, and they move a date
/// before 1970 one second later.
///
/// Swift's `WriteEntry` is a class that isn't `Sendable`, so Rust's is neither
/// `Send` nor `Sync`. An entry made with [`WriteEntry::with_archive`] uses its
/// writer's archive, so it borrows the writer.
///
/// ```compile_fail
/// # use containerization_framework as cfw;
/// let entry = cfw::containerization_archive::WriteEntry::new()?;
/// std::thread::spawn(move || entry.path());
/// # Ok::<(), cfw::Error>(())
/// ```
pub struct WriteEntry<'a> {
  pub(crate) handle: ffi::CzWriteEntry,
  pub(crate) archive: PhantomData<&'a ArchiveWriter>,
}

opaque_debug!(WriteEntry<'_>);

impl WriteEntry<'static> {
  /// `WriteEntry()`.
  pub fn new() -> Result<Self, Error> {
    let outcome = platform::outcome(ffi::cz_write_entry_new(), "make an archive entry")?;

    Ok(Self {
      handle: outcome.write_entry(),
      archive: PhantomData,
    })
  }
}

impl<'a> WriteEntry<'a> {
  /// `WriteEntry(_:)`, for `archive`.
  pub fn with_archive(archive: &'a ArchiveWriter) -> Self {
    Self {
      handle: archive.handle.new_entry(),
      archive: PhantomData,
    }
  }

  /// `WriteEntry.size`, the entry's size in bytes.
  pub fn size(&self) -> Option<i64> {
    self.handle.has_size().then(|| self.handle.size())
  }

  /// Sets `WriteEntry.size`.
  pub fn set_size(&mut self, size: Option<i64>) {
    self
      .handle
      .set_size(size.is_some(), size.unwrap_or_default());
  }

  /// `WriteEntry.permissions`, the entry's mode.
  pub fn permissions(&self) -> u16 {
    self.handle.permissions()
  }

  /// Sets `WriteEntry.permissions`.
  pub fn set_permissions(&mut self, permissions: u16) {
    self.handle.set_permissions(permissions);
  }

  /// `WriteEntry.owner`, the entry's user id. It is `None` only when the
  /// stored id doesn't fit in a `u32`, by being negative or above
  /// `u32::MAX`, and [`Self::set_owner`] never stores such an id.
  pub fn owner(&self) -> Option<u32> {
    self.handle.has_owner().then(|| self.handle.owner())
  }

  /// Sets `WriteEntry.owner`. Swift stores `None` as 0, so after setting
  /// `None`, [`Self::owner`] returns `Some(0)`.
  pub fn set_owner(&mut self, owner: Option<u32>) {
    self
      .handle
      .set_owner(owner.is_some(), owner.unwrap_or_default());
  }

  /// `WriteEntry.group`, the entry's group id. It is `None` only when the
  /// stored id doesn't fit in a `u32`, by being negative or above
  /// `u32::MAX`, and [`Self::set_group`] never stores such an id.
  pub fn group(&self) -> Option<u32> {
    self.handle.has_group().then(|| self.handle.group())
  }

  /// Sets `WriteEntry.group`. Swift stores `None` as 0, so after setting
  /// `None`, [`Self::group`] returns `Some(0)`.
  pub fn set_group(&mut self, group: Option<u32>) {
    self
      .handle
      .set_group(group.is_some(), group.unwrap_or_default());
  }

  /// `WriteEntry.hardlink`, the path of the file the entry is a hard link
  /// to.
  pub fn hardlink(&self) -> Option<String> {
    self.handle.hardlink()
  }

  /// Sets `WriteEntry.hardlink`.
  pub fn set_hardlink(&mut self, hardlink: Option<&str>) {
    self.handle.set_hardlink(hardlink.map(str::to_string));
  }

  /// `WriteEntry.hardlinkUtf8`, the hard link's target path as libarchive
  /// stores it in UTF-8 rather than in the locale's encoding.
  pub fn hardlink_utf8(&self) -> Option<String> {
    self.handle.hardlink_utf8()
  }

  /// Sets `WriteEntry.hardlinkUtf8`.
  pub fn set_hardlink_utf8(&mut self, hardlink: Option<&str>) {
    self.handle.set_hardlink_utf8(hardlink.map(str::to_string));
  }

  /// `WriteEntry.strmode`, the entry's type and permissions as `ls -l`
  /// shows them, such as `-rw-r--r-- `.
  pub fn strmode(&self) -> Option<String> {
    self.handle.strmode()
  }

  /// `WriteEntry.fileType`.
  pub fn file_type(&self) -> URLFileResourceType {
    URLFileResourceType::from_swift(&self.handle.file_type())
  }

  /// Sets `WriteEntry.fileType`.
  pub fn set_file_type(&mut self, file_type: URLFileResourceType) {
    self.handle.set_file_type(file_type.raw_value());
  }

  /// `WriteEntry.contentAccessDate`, when the entry was last read. It fails
  /// for a date `SystemTime` can't hold.
  pub fn content_access_date(&self) -> Result<Option<SystemTime>, Error> {
    self
      .handle
      .has_content_access_date()
      .then(|| platform::system_time(self.handle.content_access_date()))
      .transpose()
  }

  /// Sets `WriteEntry.contentAccessDate`.
  pub fn set_content_access_date(&mut self, date: Option<SystemTime>) {
    self
      .handle
      .set_content_access_date(date.is_some(), date.map(platform::seconds).unwrap_or_default());
  }

  /// `WriteEntry.creationDate`. It fails for a date `SystemTime` can't hold.
  pub fn creation_date(&self) -> Result<Option<SystemTime>, Error> {
    self
      .handle
      .has_creation_date()
      .then(|| platform::system_time(self.handle.creation_date()))
      .transpose()
  }

  /// Sets `WriteEntry.creationDate`.
  pub fn set_creation_date(&mut self, date: Option<SystemTime>) {
    self
      .handle
      .set_creation_date(date.is_some(), date.map(platform::seconds).unwrap_or_default());
  }

  /// `WriteEntry.modificationDate`. It fails for a date `SystemTime` can't
  /// hold.
  pub fn modification_date(&self) -> Result<Option<SystemTime>, Error> {
    self
      .handle
      .has_modification_date()
      .then(|| platform::system_time(self.handle.modification_date()))
      .transpose()
  }

  /// Sets `WriteEntry.modificationDate`.
  pub fn set_modification_date(&mut self, date: Option<SystemTime>) {
    self
      .handle
      .set_modification_date(date.is_some(), date.map(platform::seconds).unwrap_or_default());
  }

  /// `WriteEntry.path`, the entry's path in the archive.
  pub fn path(&self) -> Option<String> {
    self.handle.path()
  }

  /// Sets `WriteEntry.path`.
  pub fn set_path(&mut self, path: Option<&str>) {
    self.handle.set_path(path.map(str::to_string));
  }

  /// `WriteEntry.pathUtf8`, the entry's path as libarchive stores it in
  /// UTF-8 rather than in the locale's encoding.
  pub fn path_utf8(&self) -> Option<String> {
    self.handle.path_utf8()
  }

  /// Sets `WriteEntry.pathUtf8`.
  pub fn set_path_utf8(&mut self, path: Option<&str>) {
    self.handle.set_path_utf8(path.map(str::to_string));
  }

  /// `WriteEntry.symlinkTarget`, the path a symbolic link points to.
  pub fn symlink_target(&self) -> Option<String> {
    self.handle.symlink_target()
  }

  /// Sets `WriteEntry.symlinkTarget`.
  pub fn set_symlink_target(&mut self, target: Option<&str>) {
    self.handle.set_symlink_target(target.map(str::to_string));
  }

  /// `WriteEntry.xattrs`, the entry's extended attributes.
  pub fn xattrs(&self) -> BTreeMap<String, Vec<u8>> {
    let outcome = self.handle.xattrs();

    outcome
      .data_map_keys()
      .into_iter()
      .map(|name| {
        let value = outcome.data_map_value(&name);
        (name, value)
      })
      .collect()
  }

  /// Sets `WriteEntry.xattrs`.
  pub fn set_xattrs(&mut self, xattrs: &BTreeMap<String, Vec<u8>>) {
    let (names, lengths, values) = platform::xattrs(xattrs);

    self.handle.set_xattrs(names, lengths, values);
  }
}
