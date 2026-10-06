use super::FileModeFlag;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `EXT4.InodeNumber`.
pub type InodeNumber = u32;

/// `EXT4.Inode`. Swift's tuples are arrays.
///
/// Laid out as Swift's is, which is ext4's layout on disk, so it crosses from
/// Swift as its bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct Inode {
  pub mode: u16,
  pub uid: u16,
  pub size_low: u32,
  pub atime: u32,
  pub ctime: u32,
  pub mtime: u32,
  pub dtime: u32,
  pub gid: u16,
  pub links_count: u16,
  pub blocks_low: u32,
  pub flags: u32,
  pub version: u32,
  pub block: [u8; 60],
  pub generation: u32,
  pub xattr_block_low: u32,
  pub size_high: u32,
  pub obsolete_fragment_addr: u32,
  pub blocks_high: u16,
  pub xattr_block_high: u16,
  pub uid_high: u16,
  pub gid_high: u16,
  pub checksum_low: u16,
  pub reserved: u16,
  pub extra_isize: u16,
  pub checksum_high: u16,
  pub ctime_extra: u32,
  pub mtime_extra: u32,
  pub atime_extra: u32,
  pub crtime: u32,
  pub crtime_extra: u32,
  pub version_high: u32,
  pub projid: u32,
  pub inline_xattrs: [u8; 96],
}

// SAFETY: `repr(C)`, and only integers and arrays of them, so any bytes make
// one.
unsafe impl platform::Plain for Inode {}

impl Inode {
  /// `EXT4.Inode.Mode(_:_:)`.
  pub fn mode(mode: FileModeFlag, perm: u16) -> u16 {
    mode.0 | perm
  }

  /// `EXT4.Inode.Root()`: the root directory's inode, with its times set to
  /// now.
  pub fn root() -> Result<Self, Error> {
    platform::outcome(ffi::cz_ext4_inode_root(), "make a root inode").map(|outcome| outcome.inode())
  }
}
