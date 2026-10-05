//! `Options`' nested types.

use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `Options.Compression`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Compression {
  Store,
  Deflate,
}

raw_values!(
  /// `Options.XattrFormat`.
  XattrFormat {
    Schily = "schily",
    Libarchive = "libarchive",
    All = "all",
  }
);

impl XattrFormat {
  /// `Options.XattrFormat.description`, such as `SCHILY`.
  pub fn description(self) -> Result<String, Error> {
    platform::outcome(
      ffi::cz_xattr_format_description(self.raw_value()),
      "describe an extended attribute format",
    )
    .map(|outcome| outcome.text())
  }
}
