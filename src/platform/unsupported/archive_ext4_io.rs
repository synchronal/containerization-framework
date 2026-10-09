//! Archives, ext4 filesystems and the streams that feed them.

use super::CzOutcome;
use crate::containerization::image;
use crate::containerization_archive;
use crate::containerization_ext4::ext4;
use crate::platform;
use std::convert::Infallible;

taken!(
  ext4_reader -> CzExt4Reader,
  ext4_formatter -> CzExt4Formatter,
  scanned_size -> i64,
  scanned_items -> isize,
  inode_number -> u32,
  inode -> ext4::Inode,
  compressed_name_id -> u8,
  compressed_name_str -> String,
  len -> usize,
  read_stream -> CzReadStream,
  archive_writer -> CzArchiveWriter,
  archive_reader -> CzArchiveReader,
  write_entry -> CzWriteEntry,
  entry_data -> Vec<u8>,
  archive_entry_reader -> CzArchiveEntryReader,
  data_map_keys -> Vec<String>,
);

handles!(
  CzExt4Reader,
  CzExt4Formatter,
  CzReadStream,
  CzDataStream,
  CzWriteEntry,
  CzArchiveWriter,
  CzArchiveWriterTransaction,
  CzArchiveReader,
  CzArchiveIterator,
  CzStreamingIterator,
  CzArchiveEntryReader,
);

impl CzOutcome {
  pub(crate) fn data_map_value(&self, _key: &str) -> Vec<u8> {
    unreachable!("a failed outcome holds nothing")
  }
}

/// Swift would own the descriptor, so it is closed here instead.
pub(crate) fn cz_archive_reader_with_file_handle(_format: &str, _filter: &str, file_handle: i32) -> CzOutcome {
  // SAFETY: `ArchiveReader::with_file_handle` passes the descriptor of an
  // `OwnedFd` it gave up.
  drop(unsafe { <std::os::fd::OwnedFd as std::os::fd::FromRawFd>::from_raw_fd(file_handle) });
  CzOutcome
}

failing!(
  cz_ext4_reader_new(&str),
  cz_ext4_unpack_archive(image::EXT4Unpacker, &str, &str, &str),
  cz_ext4_reader_read_inline_extended_attributes(Vec<u8>),
  cz_ext4_reader_read_block_extended_attributes(Vec<u8>),
  cz_ext4_file_xattrs_state_read(Vec<u8>, usize, usize),
  cz_ext4_inode_root(),
  cz_ext4_extended_attribute_compress_name(&str),
  cz_ext4_extended_attribute_decompress_name(isize, &str),
  cz_ext4_formatter_new(&str, ext4::formatter::FormatterOptions),
  cz_ext4_formatter_scan_archive_headers(&str, &str, &str),
  cz_read_stream_new(),
  cz_read_stream_with_url(&str, usize),
  cz_read_stream_with_data(Vec<u8>, usize),
  cz_xattr_format_description(&str),
  cz_write_entry_new(),
  cz_archive_writer_new(containerization_archive::ArchiveWriterConfiguration),
  cz_archive_writer_with_file(containerization_archive::ArchiveWriterConfiguration, &str),
  cz_archive_reader_new(&str),
  cz_archive_reader_with_format(&str, &str, &str),
  cz_archive_reader_with_bundle(&str, Vec<u8>, Option<String>),
);

impl CzExt4Reader {
  pub(crate) fn super_block(&self) -> Vec<u8> {
    match self.0 {}
  }

  pub(crate) fn exists(&self, _path: &str, _follow_symlinks: bool) -> bool {
    match self.0 {}
  }

  pub(crate) fn stat(&self, _path: &str, _follow_symlinks: bool) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn list_directory(&self, _path: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn read_file(&self, _at: &str, _offset: u64, _count: Option<usize>, _follow_symlinks: bool) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn export(&self, _archive: &str) -> CzOutcome {
    match self.0 {}
  }
}

impl CzExt4Formatter {
  pub(crate) fn link(&self, _link: &str, _target: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn unlink(&self, _path: &str, _directory_whiteout: bool) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn create(
    &self,
    _path: &str,
    _link: Option<String>,
    _mode: u16,
    _access: f64,
    _modification: f64,
    _creation: f64,
    _now: f64,
    _has_buf: bool,
    _buf: Vec<u8>,
    _uid: Option<u32>,
    _gid: Option<u32>,
    _has_xattrs: bool,
    _xattr_names: Vec<String>,
    _xattr_lengths: Vec<u64>,
    _xattr_values: Vec<u8>,
    _recursion: bool,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn close(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn unpack(
    &self,
    _source: &str,
    _format: &str,
    _compression: &str,
    _progress: platform::Progress,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn unpack_reader(&self, _reader: CzArchiveReader, _progress: platform::Progress) -> CzOutcome {
    match self.0 {}
  }
}

impl CzReadStream {
  pub(crate) fn reset(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn data_stream(&self) -> CzDataStream {
    match self.0 {}
  }
}

impl CzDataStream {
  pub(crate) fn next(&self) -> CzOutcome {
    match self.0 {}
  }
}

impl CzWriteEntry {
  pub(crate) fn duplicate(&self) -> CzWriteEntry {
    match self.0 {}
  }

  pub(crate) fn has_size(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn size(&self) -> i64 {
    match self.0 {}
  }

  pub(crate) fn set_size(&self, _is_set: bool, _size: i64) {
    match self.0 {}
  }

  pub(crate) fn permissions(&self) -> u16 {
    match self.0 {}
  }

  pub(crate) fn set_permissions(&self, _permissions: u16) {
    match self.0 {}
  }

  pub(crate) fn has_owner(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn owner(&self) -> u32 {
    match self.0 {}
  }

  pub(crate) fn set_owner(&self, _is_set: bool, _owner: u32) {
    match self.0 {}
  }

  pub(crate) fn has_group(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn group(&self) -> u32 {
    match self.0 {}
  }

  pub(crate) fn set_group(&self, _is_set: bool, _group: u32) {
    match self.0 {}
  }

  pub(crate) fn hardlink(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn set_hardlink(&self, _hardlink: Option<String>) {
    match self.0 {}
  }

  pub(crate) fn hardlink_utf8(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn set_hardlink_utf8(&self, _hardlink: Option<String>) {
    match self.0 {}
  }

  pub(crate) fn strmode(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn file_type(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn set_file_type(&self, _file_type: &str) {
    match self.0 {}
  }

  pub(crate) fn has_content_access_date(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn content_access_date(&self) -> f64 {
    match self.0 {}
  }

  pub(crate) fn set_content_access_date(&self, _is_set: bool, _seconds: f64) {
    match self.0 {}
  }

  pub(crate) fn has_creation_date(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn creation_date(&self) -> f64 {
    match self.0 {}
  }

  pub(crate) fn set_creation_date(&self, _is_set: bool, _seconds: f64) {
    match self.0 {}
  }

  pub(crate) fn has_modification_date(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn modification_date(&self) -> f64 {
    match self.0 {}
  }

  pub(crate) fn set_modification_date(&self, _is_set: bool, _seconds: f64) {
    match self.0 {}
  }

  pub(crate) fn path(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn set_path(&self, _path: Option<String>) {
    match self.0 {}
  }

  pub(crate) fn path_utf8(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn set_path_utf8(&self, _path: Option<String>) {
    match self.0 {}
  }

  pub(crate) fn symlink_target(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn set_symlink_target(&self, _target: Option<String>) {
    match self.0 {}
  }

  pub(crate) fn xattrs(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn set_xattrs(&self, _names: Vec<String>, _lengths: Vec<u64>, _values: Vec<u8>) {
    match self.0 {}
  }
}

impl CzArchiveWriter {
  pub(crate) fn new_entry(&self) -> CzWriteEntry {
    match self.0 {}
  }

  pub(crate) fn open(&self, _file: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn open_with_file_descriptor(&self, _file_descriptor: i32) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn finish_encoding(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn make_transaction_writer(&self) -> CzArchiveWriterTransaction {
    match self.0 {}
  }

  pub(crate) fn write_entry(&self, _entry: CzWriteEntry, _has_data: bool, _data: Vec<u8>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn archive_directory(&self, _dir: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn archive(&self, _paths: Vec<String>, _base: &str) -> CzOutcome {
    match self.0 {}
  }
}

impl CzArchiveWriterTransaction {
  pub(crate) fn write_header(&self, _entry: CzWriteEntry) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn write_chunk(&self, _data: Vec<u8>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn finish(&self) -> CzOutcome {
    match self.0 {}
  }
}

impl CzArchiveReader {
  pub(crate) fn duplicate(&self) -> CzArchiveReader {
    match self.0 {}
  }

  pub(crate) fn make_iterator(&self) -> CzArchiveIterator {
    match self.0 {}
  }

  pub(crate) fn make_streaming_iterator(&self) -> CzStreamingIterator {
    match self.0 {}
  }

  pub(crate) fn throw_if_stream_failed(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn extract_contents(&self, _to: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn extract_file(&self, _path: &str) -> CzOutcome {
    match self.0 {}
  }
}

impl CzArchiveIterator {
  pub(crate) fn next(&self) -> CzOutcome {
    match self.0 {}
  }
}

impl CzStreamingIterator {
  pub(crate) fn next(&self) -> CzOutcome {
    match self.0 {}
  }
}

impl CzArchiveEntryReader {
  pub(crate) fn read(&self, _max_length: usize) -> CzOutcome {
    match self.0 {}
  }
}
