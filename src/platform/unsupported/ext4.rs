//! Stand-ins for `Ext4.swift`: `EXT4.EXT4Reader`, `EXT4.Formatter` and the
//! values they read and write.

use super::CzArchiveReader;
use super::CzImage;
use super::CzOutcome;
use crate::containerization::image;
use crate::containerization_ext4::ext4;
use crate::containerization_oci;
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
);

handles!(CzExt4Reader, CzExt4Formatter);

failing!(
  cz_ext4_reader_new(&str),
  cz_ext4_reader_read_inline_extended_attributes(Vec<u8>),
  cz_ext4_reader_read_block_extended_attributes(Vec<u8>),
  cz_ext4_file_xattrs_state_read(Vec<u8>, usize, usize),
  cz_ext4_inode_root(),
  cz_ext4_extended_attribute_compress_name(&str),
  cz_ext4_extended_attribute_decompress_name(isize, &str),
  cz_ext4_formatter_new(&str, ext4::formatter::FormatterOptions),
  cz_ext4_formatter_scan_archive_headers(&str, &str, &str),
);

pub(crate) fn cz_ext4_unpack(
  _unpacker: image::EXT4Unpacker,
  image: CzImage,
  _platform: containerization_oci::image::Platform,
  _at: &str,
  _progress: platform::Progress,
) -> CzOutcome {
  match image.0 {}
}

uninhabited!(CzExt4Reader {
  fn super_block() -> Vec<u8>;
  fn exists(&str, bool) -> bool;
  fn stat(&str, bool) -> CzOutcome;
  fn list_directory(&str) -> CzOutcome;
  fn read_file(&str, u64, Option<usize>, bool) -> CzOutcome;
  fn export(&str) -> CzOutcome;
});

uninhabited!(CzExt4Formatter {
  fn link(&str, &str) -> CzOutcome;
  fn unlink(&str, bool) -> CzOutcome;
  fn create(
    &str,
    Option<String>,
    u16,
    f64,
    f64,
    f64,
    f64,
    bool,
    Vec<u8>,
    Option<u32>,
    Option<u32>,
    bool,
    Vec<String>,
    Vec<u64>,
    Vec<u8>,
    bool,
  ) -> CzOutcome;
  fn close() -> CzOutcome;
  fn unpack(&str, &str, &str, platform::Progress) -> CzOutcome;
  fn unpack_reader(CzArchiveReader, platform::Progress) -> CzOutcome;
});
