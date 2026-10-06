use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `EXT4.ExtendedAttribute`. Swift's fields are internal, so it holds nothing
/// Rust can read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtendedAttribute {
  _private: (),
}

impl ExtendedAttribute {
  /// `prefixMap`, sorted by its keys.
  pub const PREFIX_MAP: &[(isize, &str)] = &[
    (1, "user."),
    (2, "system.posix_acl_access"),
    (3, "system.posix_acl_default"),
    (4, "trusted."),
    (6, "security."),
    (7, "system."),
    (8, "system.richacl"),
  ];

  /// What Swift returns in place of each of its attributes.
  pub(crate) fn opaque() -> Self {
    Self { _private: () }
  }

  /// `compressName(_:)`: the id of the name's longest known prefix, and the
  /// rest of the name.
  pub fn compress_name(name: &str) -> Result<(u8, String), Error> {
    platform::outcome(
      ffi::cz_ext4_extended_attribute_compress_name(name),
      format!("compress the extended attribute name {name}"),
    )
    .map(|outcome| (outcome.compressed_name_id(), outcome.compressed_name_str()))
  }

  /// `decompressName(id:suffix:)`.
  pub fn decompress_name(id: isize, suffix: &str) -> Result<String, Error> {
    platform::outcome(
      ffi::cz_ext4_extended_attribute_decompress_name(id, suffix),
      format!("decompress the extended attribute name {suffix}"),
    )
    .map(|outcome| outcome.text())
  }
}
