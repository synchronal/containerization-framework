//! `EXT4.JournalConfig`, and its nested `EXT4.JournalConfig.JournalMode`.

/// `EXT4.JournalConfig.JournalMode`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JournalMode {
  Writeback,
  Ordered,
  Journal,
}

/// `EXT4.JournalConfig`. [`Default`] is Swift's `JournalConfig.default`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct JournalConfig {
  pub size: Option<u64>,
  pub default_mode: Option<JournalMode>,
}
