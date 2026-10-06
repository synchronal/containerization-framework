//! Getters Swift reads ContainerizationEXT4's options through, and readers for
//! what it returns.

use crate::bridge::ffi;
use crate::containerization_ext4::ext4;
use crate::containerization_ext4::ext4::formatter::FormatterOptions;
use crate::containerization_ext4::ext4::journal_config::JournalMode;
use crate::platform;

impl FormatterOptions {
  pub(crate) fn block_size(&self) -> u32 {
    self.block_size
  }

  pub(crate) fn min_disk_size(&self) -> u64 {
    self.min_disk_size
  }

  pub(crate) fn has_journal(&self) -> bool {
    self.journal.is_some()
  }

  pub(crate) fn journal_size(&self) -> Option<u64> {
    self.journal.and_then(|journal| journal.size)
  }

  pub(crate) fn has_journal_mode(&self) -> bool {
    self
      .journal
      .and_then(|journal| journal.default_mode)
      .is_some()
  }

  pub(crate) fn journal_mode(&self) -> ffi::JournalModeKind {
    match self
      .journal
      .and_then(|journal| journal.default_mode)
      .expect("Swift asks for the mode only after `has_journal_mode`")
    {
      JournalMode::Writeback => ffi::JournalModeKind::Writeback,
      JournalMode::Ordered => ffi::JournalModeKind::Ordered,
      JournalMode::Journal => ffi::JournalModeKind::Journal,
    }
  }
}

impl ffi::CzOutcome {
  /// The `Inode` an outcome holds, alone or with its number.
  pub(crate) fn inode(&self) -> ext4::Inode {
    platform::from_bytes(&self.inode_bytes())
  }
}

#[cfg(test)]
mod tests {
  use crate::bridge::ffi;
  use crate::containerization_ext4::ext4;
  use crate::containerization_ext4::file_timestamps;
  use crate::containerization_io::ReadStream;
  use std::mem::offset_of;
  use std::mem::size_of;
  use std::time::Duration;
  use std::time::SystemTime;

  #[test]
  fn copies_swifts_file_mode_flags() {
    assert_eq!(
      ffi::cz_ext4_file_mode_flags(),
      ext4::FileModeFlag::ALL
        .iter()
        .map(|flag| ext4::Inode::mode(*flag, 0))
        .collect::<Vec<_>>()
    );
  }

  #[test]
  fn copies_swifts_prefix_map() {
    let map = ext4::ExtendedAttribute::PREFIX_MAP;

    assert_eq!(
      ffi::cz_ext4_prefix_map_keys(),
      map.iter().map(|(key, _)| *key).collect::<Vec<_>>()
    );
    assert_eq!(
      ffi::cz_ext4_prefix_map_values(),
      map
        .iter()
        .map(|(_, value)| value.to_string())
        .collect::<Vec<_>>()
    );
  }

  #[test]
  fn copies_swifts_super_block_magic() {
    assert_eq!(ffi::cz_ext4_super_block_magic(), ext4::SUPER_BLOCK_MAGIC);
  }

  #[test]
  fn lays_out_super_blocks_and_inodes_as_swift_does() {
    assert_eq!(
      ffi::cz_ext4_layout(),
      [
        size_of::<ext4::SuperBlock>(),
        offset_of!(ext4::SuperBlock, magic),
        offset_of!(ext4::SuperBlock, mmp_block),
        offset_of!(ext4::SuperBlock, last_error_block),
        offset_of!(ext4::SuperBlock, reserved),
        offset_of!(ext4::SuperBlock, checksum),
        size_of::<ext4::Inode>(),
        offset_of!(ext4::Inode, block),
        offset_of!(ext4::Inode, projid),
        offset_of!(ext4::Inode, inline_xattrs),
      ]
      .map(|bytes| bytes as u64)
    );
  }

  #[test]
  fn reads_timestamps_as_swift_does() {
    let dates = [
      SystemTime::UNIX_EPOCH,
      SystemTime::UNIX_EPOCH + Duration::from_millis(1_700_000_000_250),
      SystemTime::UNIX_EPOCH + Duration::from_secs(0x1_2345_6789),
      SystemTime::UNIX_EPOCH - Duration::from_millis(86_400_500),
      SystemTime::UNIX_EPOCH - Duration::from_secs(0x1_0000_0000),
      SystemTime::UNIX_EPOCH + Duration::from_secs(0x4_0000_0000),
      SystemTime::UNIX_EPOCH - Duration::from_secs(62_135_769_600),
      SystemTime::UNIX_EPOCH - Duration::from_secs(62_135_769_599),
    ];

    for date in dates {
      let fs = file_timestamps::fs(date);

      assert_eq!(
        ffi::cz_file_timestamps_access(crate::platform::seconds(date)),
        [fs as u32, (fs >> 32) as u32],
        "{date:?}"
      );
    }
  }

  #[test]
  fn copies_swifts_read_stream_buffer_size() {
    assert_eq!(ffi::cz_read_stream_buffer_size() as usize, ReadStream::BUFFER_SIZE);
  }
}
