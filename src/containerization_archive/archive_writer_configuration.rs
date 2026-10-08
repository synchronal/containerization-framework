use super::options;

raw_values!(
  /// `Format`.
  Format {
    Ustar = "ustar",
    Gnutar = "gnutar",
    Pax = "pax",
    PaxRestricted = "paxRestricted",
    Cpio = "cpio",
    CpioNewc = "cpioNewc",
    Zip = "zip",
    Shar = "shar",
    SharDump = "sharDump",
    Iso9660 = "iso9660",
    SevenZip = "sevenZip",
    ArBSD = "arBSD",
    ArGNU = "arGNU",
    Mtree = "mtree",
    Xar = "xar",
  }
);

raw_values!(
  /// `Filter`.
  Filter {
    None = "none",
    Gzip = "gzip",
    Bzip2 = "bzip2",
    Compress = "compress",
    Lzma = "lzma",
    Xz = "xz",
    Uu = "uu",
    Rpm = "rpm",
    Lzip = "lzip",
    Lrzip = "lrzip",
    Lzop = "lzop",
    Grzip = "grzip",
    Lz4 = "lz4",
    Zstd = "zstd",
  }
);

/// `Options`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Options {
  CompressionLevel(u32),
  Compression(options::Compression),
  Xattrformat(options::XattrFormat),
}

/// `ArchiveWriterConfiguration`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveWriterConfiguration {
  pub format: Format,
  pub filter: Filter,
  pub options: Vec<Options>,
  pub locales: Vec<String>,
}

impl ArchiveWriterConfiguration {
  /// `ArchiveWriterConfiguration.defaultLocales`.
  pub const DEFAULT_LOCALES: &[&str] = &["en_US.UTF-8", "C.UTF-8"];

  /// `ArchiveWriterConfiguration(format:filter:options:locales:)`, with
  /// Swift's defaults: no options, and [`Self::DEFAULT_LOCALES`].
  pub fn new(format: Format, filter: Filter) -> Self {
    Self {
      format,
      filter,
      options: Vec::new(),
      locales: Self::default_locales(),
    }
  }

  /// [`Self::DEFAULT_LOCALES`], owned.
  pub(super) fn default_locales() -> Vec<String> {
    Self::DEFAULT_LOCALES
      .iter()
      .map(|locale| locale.to_string())
      .collect()
  }
}
