//! `ArchiveWriterConfiguration`, as Swift reads it.

use crate::bridge::ffi;
use crate::containerization_archive;
use crate::containerization_archive::options;

impl containerization_archive::ArchiveWriterConfiguration {
  pub(crate) fn format(&self) -> &str {
    self.format.raw_value()
  }

  pub(crate) fn filter(&self) -> &str {
    self.filter.raw_value()
  }

  pub(crate) fn options_len(&self) -> usize {
    self.options.len()
  }

  pub(crate) fn option_kind_at(&self, index: usize) -> ffi::ArchiveOptionKind {
    match self.options[index] {
      containerization_archive::Options::CompressionLevel(_) => ffi::ArchiveOptionKind::CompressionLevel,
      containerization_archive::Options::Compression(options::Compression::Store) => {
        ffi::ArchiveOptionKind::CompressionStore
      }
      containerization_archive::Options::Compression(options::Compression::Deflate) => {
        ffi::ArchiveOptionKind::CompressionDeflate
      }
      containerization_archive::Options::Xattrformat(_) => ffi::ArchiveOptionKind::Xattrformat,
    }
  }

  pub(crate) fn option_compression_level_at(&self, index: usize) -> u32 {
    match self.options[index] {
      containerization_archive::Options::CompressionLevel(level) => level,
      _ => unreachable!("Swift asks for a level only of a compression level"),
    }
  }

  pub(crate) fn option_xattr_format_at(&self, index: usize) -> &str {
    match self.options[index] {
      containerization_archive::Options::Xattrformat(format) => format.raw_value(),
      _ => unreachable!("Swift asks for a format only of an extended attribute format"),
    }
  }

  getters! {
    list locales: str => locales_len, locales_at;
  }
}

#[cfg(test)]
mod tests {
  use crate::bridge::accessors::raw_values;
  use crate::bridge::ffi;
  use crate::containerization_archive;
  use crate::containerization_archive::options;

  /// Checks each enum's raw values, in Rust's order, against Swift's.
  #[test]
  fn copies_swifts_raw_values() {
    let copies = [
      (
        "Format",
        raw_values(
          containerization_archive::Format::ALL,
          containerization_archive::Format::raw_value,
        ),
      ),
      (
        "Filter",
        raw_values(
          containerization_archive::Filter::ALL,
          containerization_archive::Filter::raw_value,
        ),
      ),
      (
        "XattrFormat",
        raw_values(options::XattrFormat::ALL, options::XattrFormat::raw_value),
      ),
      (
        "URLFileResourceType",
        raw_values(
          containerization_archive::URLFileResourceType::ALL,
          containerization_archive::URLFileResourceType::raw_value,
        ),
      ),
    ];

    for (name, copy) in copies {
      assert_eq!(ffi::cz_archive_raw_values(name), copy, "{name}");
    }
  }

  #[test]
  fn copies_swifts_default_locales() {
    assert_eq!(
      ffi::cz_archive_default_locales(),
      containerization_archive::ArchiveWriterConfiguration::DEFAULT_LOCALES
    );
  }
}
