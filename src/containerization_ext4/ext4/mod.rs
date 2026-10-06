//! `EXT4`, the namespace `ContainerizationEXT4`'s types are nested in.
//! `EXT4Reader.readFile` and `Formatter`'s methods take their defaulted
//! arguments in options structs, in [`ext4_reader`] and [`formatter`].

pub mod ext4_reader;
mod extended_attribute;
mod file_mode_flag;
pub mod formatter;
mod inode;
pub mod journal_config;
mod super_block;

pub use self::ext4_reader::Ext4Reader;
pub use self::extended_attribute::ExtendedAttribute;
pub use self::file_mode_flag::FileModeFlag;
pub use self::formatter::Formatter;
pub use self::inode::Inode;
pub use self::inode::InodeNumber;
pub use self::journal_config::JournalConfig;
pub use self::super_block::SuperBlock;

/// `EXT4.SuperBlockMagic`.
pub const SUPER_BLOCK_MAGIC: u16 = 0xef53;
