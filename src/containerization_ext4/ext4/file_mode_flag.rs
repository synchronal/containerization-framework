use std::ops::BitOr;

/// `EXT4.FileModeFlag`. Like Swift's, its raw value is private: pass it to
/// [`Inode::mode`](super::Inode::mode).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FileModeFlag(pub(crate) u16);

impl FileModeFlag {
  pub const S_IXOTH: Self = Self(0x1);
  pub const S_IWOTH: Self = Self(0x2);
  pub const S_IROTH: Self = Self(0x4);
  pub const S_IXGRP: Self = Self(0x8);
  pub const S_IWGRP: Self = Self(0x10);
  pub const S_IRGRP: Self = Self(0x20);
  pub const S_IXUSR: Self = Self(0x40);
  pub const S_IWUSR: Self = Self(0x80);
  pub const S_IRUSR: Self = Self(0x100);
  pub const S_ISVTX: Self = Self(0x200);
  pub const S_ISGID: Self = Self(0x400);
  pub const S_ISUID: Self = Self(0x800);
  pub const S_IFIFO: Self = Self(0x1000);
  pub const S_IFCHR: Self = Self(0x2000);
  pub const S_IFDIR: Self = Self(0x4000);
  pub const S_IFBLK: Self = Self(0x6000);
  pub const S_IFREG: Self = Self(0x8000);
  pub const S_IFLNK: Self = Self(0xA000);
  pub const S_IFSOCK: Self = Self(0xC000);

  /// `TypeMask`.
  pub const TYPE_MASK: Self = Self(0xF000);

  /// Every flag, in the order the bridge lists Swift's.
  #[cfg(all(test, target_os = "macos"))]
  pub(crate) const ALL: &[Self] = &[
    Self::S_IXOTH,
    Self::S_IWOTH,
    Self::S_IROTH,
    Self::S_IXGRP,
    Self::S_IWGRP,
    Self::S_IRGRP,
    Self::S_IXUSR,
    Self::S_IWUSR,
    Self::S_IRUSR,
    Self::S_ISVTX,
    Self::S_ISGID,
    Self::S_ISUID,
    Self::S_IFIFO,
    Self::S_IFCHR,
    Self::S_IFDIR,
    Self::S_IFBLK,
    Self::S_IFREG,
    Self::S_IFLNK,
    Self::S_IFSOCK,
    Self::TYPE_MASK,
  ];
}

/// `|`. Swift's other `|`, returning a `UInt16`, is `Inode::mode(lhs | rhs, 0)`.
impl BitOr for FileModeFlag {
  type Output = Self;

  fn bitor(self, rhs: Self) -> Self {
    Self(self.0 | rhs.0)
  }
}
