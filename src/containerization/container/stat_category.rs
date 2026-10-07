use std::ops::BitOr;

/// `StatCategory`, the statistics [`super::LinuxContainer::statistics`]
/// reports.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StatCategory {
  pub raw_value: i64,
}

impl StatCategory {
  /// `StatCategory.process`: `pids.current` and `pids.max`.
  pub const PROCESS: Self = Self::new(1 << 0);
  /// `StatCategory.memory`.
  pub const MEMORY: Self = Self::new(1 << 1);
  /// `StatCategory.cpu`.
  pub const CPU: Self = Self::new(1 << 2);
  /// `StatCategory.blockIO`.
  pub const BLOCK_IO: Self = Self::new(1 << 3);
  /// `StatCategory.network`.
  pub const NETWORK: Self = Self::new(1 << 4);
  /// `StatCategory.memoryEvents`.
  pub const MEMORY_EVENTS: Self = Self::new(1 << 5);
  /// `StatCategory.filesystem`.
  pub const FILESYSTEM: Self = Self::new(1 << 6);
  /// `StatCategory.all`.
  pub const ALL: Self = Self::new(0x7f);

  /// `StatCategory(rawValue:)`.
  pub const fn new(raw_value: i64) -> Self {
    Self { raw_value }
  }

  /// `contains(_:)`.
  pub const fn contains(self, member: Self) -> bool {
    self.raw_value & member.raw_value == member.raw_value
  }
}

/// `union(_:)`.
impl BitOr for StatCategory {
  type Output = Self;

  fn bitor(self, rhs: Self) -> Self {
    Self::new(self.raw_value | rhs.raw_value)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn all_is_every_category() {
    let every = StatCategory::PROCESS
      | StatCategory::MEMORY
      | StatCategory::CPU
      | StatCategory::BLOCK_IO
      | StatCategory::NETWORK
      | StatCategory::MEMORY_EVENTS
      | StatCategory::FILESYSTEM;

    assert_eq!(StatCategory::ALL, every);
    assert!(StatCategory::ALL.contains(StatCategory::MEMORY_EVENTS));
    assert!(!StatCategory::MEMORY.contains(StatCategory::ALL));
  }
}
