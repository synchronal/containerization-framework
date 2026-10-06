/// `TimeSpec`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TimeSpec {
  pub seconds: i64,
  pub nanoseconds: i32,
}

/// `Stat`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Stat {
  pub dev: u64,
  pub ino: u64,
  pub mode: u32,
  pub nlink: u64,
  pub uid: u32,
  pub gid: u32,
  pub rdev: u64,
  pub size: i64,
  pub blksize: i64,
  pub blocks: i64,
  pub atime: TimeSpec,
  pub mtime: TimeSpec,
  pub ctime: TimeSpec,
}
