use super::ExtendedAttribute;
use crate::error::Error;
use crate::platform::ffi;

/// `EXT4.FileXattrsState`. Swift's initializer is internal and nothing public
/// returns one, so only its static `read` is bound.
#[derive(Debug)]
pub struct FileXattrsState {
  _private: (),
}

impl FileXattrsState {
  /// `EXT4.FileXattrsState.read(buffer:start:offset:)`: the attributes whose
  /// entries begin at `start`, with value offsets counted from `offset`.
  pub fn read(buffer: &[u8], start: usize, offset: usize) -> Result<Vec<ExtendedAttribute>, Error> {
    super::ext4_reader::attributes(
      ffi::cz_ext4_file_xattrs_state_read(buffer.to_vec(), start, offset),
      "read extended attributes",
    )
  }
}
