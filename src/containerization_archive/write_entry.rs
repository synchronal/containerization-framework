use super::ArchiveWriter;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::collections::BTreeMap;
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
pub struct WriteEntry {
  pub(crate) handle: ffi::CzWriteEntry,
}

// Swift's `WriteEntry` is a class that isn't `Sendable`; Rust's isn't `Sync`,
// so one call runs at a time.
unsafe impl Send for WriteEntry {}

impl WriteEntry {
  /// `WriteEntry()`.
  pub fn new() -> Result<Self, Error> {
    let outcome = platform::outcome(ffi::cz_write_entry_new(), "make an archive entry")?;

    Ok(Self {
      handle: outcome.write_entry(),
    })
  }

  /// `WriteEntry(_:)`, for `archive`.
  pub fn with_archive(archive: &ArchiveWriter) -> Self {
    Self {
      handle: archive.handle.new_entry(),
    }
  }

  /// `size`.
  pub fn size(&self) -> Option<i64> {
    self.handle.has_size().then(|| self.handle.size())
  }

  pub fn set_size(&mut self, size: Option<i64>) {
    self
      .handle
      .set_size(size.is_some(), size.unwrap_or_default());
  }

  /// `permissions`.
  pub fn permissions(&self) -> u16 {
    self.handle.permissions()
  }

  pub fn set_permissions(&mut self, permissions: u16) {
    self.handle.set_permissions(permissions);
  }

  /// `owner`.
  pub fn owner(&self) -> Option<u32> {
    self.handle.has_owner().then(|| self.handle.owner())
  }

  pub fn set_owner(&mut self, owner: Option<u32>) {
    self
      .handle
      .set_owner(owner.is_some(), owner.unwrap_or_default());
  }

  /// `group`.
  pub fn group(&self) -> Option<u32> {
    self.handle.has_group().then(|| self.handle.group())
  }

  pub fn set_group(&mut self, group: Option<u32>) {
    self
      .handle
      .set_group(group.is_some(), group.unwrap_or_default());
  }

  /// `hardlink`.
  pub fn hardlink(&self) -> Option<String> {
    self.handle.hardlink()
  }

  pub fn set_hardlink(&mut self, hardlink: Option<&str>) {
    self.handle.set_hardlink(hardlink.map(str::to_string));
  }

  /// `hardlinkUtf8`.
  pub fn hardlink_utf8(&self) -> Option<String> {
    self.handle.hardlink_utf8()
  }

  pub fn set_hardlink_utf8(&mut self, hardlink: Option<&str>) {
    self.handle.set_hardlink_utf8(hardlink.map(str::to_string));
  }

  /// `strmode`, such as `-rw-r--r-- `.
  pub fn strmode(&self) -> Option<String> {
    self.handle.strmode()
  }

  /// `fileType`.
  pub fn file_type(&self) -> URLFileResourceType {
    URLFileResourceType::from_swift(&self.handle.file_type())
  }

  pub fn set_file_type(&mut self, file_type: URLFileResourceType) {
    self.handle.set_file_type(file_type.raw_value());
  }

  /// `contentAccessDate`.
  pub fn content_access_date(&self) -> Option<SystemTime> {
    self
      .handle
      .has_content_access_date()
      .then(|| platform::system_time(self.handle.content_access_date()))
  }

  pub fn set_content_access_date(&mut self, date: Option<SystemTime>) {
    self
      .handle
      .set_content_access_date(date.is_some(), date.map(platform::seconds).unwrap_or_default());
  }

  /// `creationDate`.
  pub fn creation_date(&self) -> Option<SystemTime> {
    self
      .handle
      .has_creation_date()
      .then(|| platform::system_time(self.handle.creation_date()))
  }

  pub fn set_creation_date(&mut self, date: Option<SystemTime>) {
    self
      .handle
      .set_creation_date(date.is_some(), date.map(platform::seconds).unwrap_or_default());
  }

  /// `modificationDate`.
  pub fn modification_date(&self) -> Option<SystemTime> {
    self
      .handle
      .has_modification_date()
      .then(|| platform::system_time(self.handle.modification_date()))
  }

  pub fn set_modification_date(&mut self, date: Option<SystemTime>) {
    self
      .handle
      .set_modification_date(date.is_some(), date.map(platform::seconds).unwrap_or_default());
  }

  /// `path`.
  pub fn path(&self) -> Option<String> {
    self.handle.path()
  }

  pub fn set_path(&mut self, path: Option<&str>) {
    self.handle.set_path(path.map(str::to_string));
  }

  /// `pathUtf8`.
  pub fn path_utf8(&self) -> Option<String> {
    self.handle.path_utf8()
  }

  pub fn set_path_utf8(&mut self, path: Option<&str>) {
    self.handle.set_path_utf8(path.map(str::to_string));
  }

  /// `symlinkTarget`.
  pub fn symlink_target(&self) -> Option<String> {
    self.handle.symlink_target()
  }

  pub fn set_symlink_target(&mut self, target: Option<&str>) {
    self.handle.set_symlink_target(target.map(str::to_string));
  }

  /// `xattrs`.
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

  pub fn set_xattrs(&mut self, xattrs: &BTreeMap<String, Vec<u8>>) {
    let (names, lengths, values) = platform::xattrs(xattrs);

    self.handle.set_xattrs(names, lengths, values);
  }
}
