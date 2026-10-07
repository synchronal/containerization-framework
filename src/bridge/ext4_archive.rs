//! `ContainerizationEXT4`'s and `ContainerizationArchive`'s configuration
//! values, which Swift reads.

use crate::containerization_archive::ArchiveWriterConfiguration as RustArchiveWriterConfiguration;
use crate::containerization_ext4::ext4::formatter::FormatterOptions as RustFormatterOptions;

use super::ffi::ArchiveOptionKind;
use super::ffi::JournalModeKind;

#[swift_bridge::bridge]
mod ffi {
  #[swift_bridge(already_declared)]
  enum JournalModeKind {}

  #[swift_bridge(already_declared)]
  enum ArchiveOptionKind {}

  extern "Rust" {
    type RustFormatterOptions;
    #[swift_bridge(swift_name = "blockSize")]
    fn block_size(self: &RustFormatterOptions) -> u32;
    #[swift_bridge(swift_name = "minDiskSize")]
    fn min_disk_size(self: &RustFormatterOptions) -> u64;
    #[swift_bridge(swift_name = "hasJournal")]
    fn has_journal(self: &RustFormatterOptions) -> bool;
    #[swift_bridge(swift_name = "journalSize")]
    fn journal_size(self: &RustFormatterOptions) -> Option<u64>;
    #[swift_bridge(swift_name = "hasJournalMode")]
    fn has_journal_mode(self: &RustFormatterOptions) -> bool;
    #[swift_bridge(swift_name = "journalMode")]
    fn journal_mode(self: &RustFormatterOptions) -> JournalModeKind;

    // Its enums are their `rawValue`s. An option's level and format are
    // read only for the kinds that have them.
    type RustArchiveWriterConfiguration;
    fn format(self: &RustArchiveWriterConfiguration) -> &str;
    fn filter(self: &RustArchiveWriterConfiguration) -> &str;
    #[swift_bridge(swift_name = "optionsLen")]
    fn options_len(self: &RustArchiveWriterConfiguration) -> usize;
    #[swift_bridge(swift_name = "optionKindAt")]
    fn option_kind_at(self: &RustArchiveWriterConfiguration, index: usize) -> ArchiveOptionKind;
    #[swift_bridge(swift_name = "optionCompressionLevelAt")]
    fn option_compression_level_at(self: &RustArchiveWriterConfiguration, index: usize) -> u32;
    #[swift_bridge(swift_name = "optionXattrFormatAt")]
    fn option_xattr_format_at(self: &RustArchiveWriterConfiguration, index: usize) -> &str;
    #[swift_bridge(swift_name = "localesLen")]
    fn locales_len(self: &RustArchiveWriterConfiguration) -> usize;
    #[swift_bridge(swift_name = "localesAt")]
    fn locales_at(self: &RustArchiveWriterConfiguration, index: usize) -> &str;
  }
}
