//! Stand-ins for `Archive.swift`: ContainerizationArchive's writers, readers
//! and entries.

use super::CzOutcome;
use crate::containerization::image;
use crate::containerization_archive;
use std::convert::Infallible;

taken!(
  archive_writer -> CzArchiveWriter,
  archive_reader -> CzArchiveReader,
  write_entry -> CzWriteEntry,
  entry_data -> Vec<u8>,
  archive_entry_reader -> CzArchiveEntryReader,
  data_map_keys -> Vec<String>,
);

handles!(
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
  cz_ext4_unpack_archive(image::EXT4Unpacker, &str, &str, &str),
  cz_xattr_format_description(&str),
  cz_write_entry_new(),
  cz_archive_writer_new(containerization_archive::ArchiveWriterConfiguration),
  cz_archive_writer_with_file(containerization_archive::ArchiveWriterConfiguration, &str),
  cz_archive_reader_new(&str),
  cz_archive_reader_with_format(&str, &str, &str),
  cz_archive_reader_with_bundle(&str, Vec<u8>, Option<String>),
);

uninhabited!(CzWriteEntry {
  fn duplicate() -> CzWriteEntry;
  fn has_size() -> bool;
  fn size() -> i64;
  fn set_size(bool, i64);
  fn permissions() -> u16;
  fn set_permissions(u16);
  fn has_owner() -> bool;
  fn owner() -> u32;
  fn set_owner(bool, u32);
  fn has_group() -> bool;
  fn group() -> u32;
  fn set_group(bool, u32);
  fn hardlink() -> Option<String>;
  fn set_hardlink(Option<String>);
  fn hardlink_utf8() -> Option<String>;
  fn set_hardlink_utf8(Option<String>);
  fn strmode() -> Option<String>;
  fn file_type() -> String;
  fn set_file_type(&str);
  fn has_content_access_date() -> bool;
  fn content_access_date() -> f64;
  fn set_content_access_date(bool, f64);
  fn has_creation_date() -> bool;
  fn creation_date() -> f64;
  fn set_creation_date(bool, f64);
  fn has_modification_date() -> bool;
  fn modification_date() -> f64;
  fn set_modification_date(bool, f64);
  fn path() -> Option<String>;
  fn set_path(Option<String>);
  fn path_utf8() -> Option<String>;
  fn set_path_utf8(Option<String>);
  fn symlink_target() -> Option<String>;
  fn set_symlink_target(Option<String>);
  fn xattrs() -> CzOutcome;
  fn set_xattrs(Vec<String>, Vec<u64>, Vec<u8>);
});

uninhabited!(CzArchiveWriter {
  fn new_entry() -> CzWriteEntry;
  fn open(&str) -> CzOutcome;
  fn open_with_file_descriptor(i32) -> CzOutcome;
  fn finish_encoding() -> CzOutcome;
  fn make_transaction_writer() -> CzArchiveWriterTransaction;
  fn write_entry(CzWriteEntry, bool, Vec<u8>) -> CzOutcome;
  fn archive_directory(&str) -> CzOutcome;
  fn archive(Vec<String>, &str) -> CzOutcome;
});

uninhabited!(CzArchiveWriterTransaction {
  fn write_header(CzWriteEntry) -> CzOutcome;
  fn write_chunk(Vec<u8>) -> CzOutcome;
  fn finish() -> CzOutcome;
});

uninhabited!(CzArchiveReader {
  fn duplicate() -> CzArchiveReader;
  fn make_iterator() -> CzArchiveIterator;
  fn make_streaming_iterator() -> CzStreamingIterator;
  fn throw_if_stream_failed() -> CzOutcome;
  fn extract_contents(&str) -> CzOutcome;
  fn extract_file(&str) -> CzOutcome;
});

uninhabited!(CzArchiveIterator {
  fn next() -> CzOutcome;
});

uninhabited!(CzStreamingIterator {
  fn next() -> CzOutcome;
});

uninhabited!(CzArchiveEntryReader {
  fn read(usize) -> CzOutcome;
});
