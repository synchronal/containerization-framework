//! Containerization's `ContainerizationOS` module: files, terminals, sysctls,
//! volume subpaths and the keychain, and a few Linux interfaces. The keychain
//! types are in [`keychain`], and binfmt_misc, capabilities and network
//! protocol counters are in [`linux`], as they are in Swift's `Keychain` and
//! `Linux` directories. Reading the Linux counters fails on macOS.

pub mod file;
pub mod file_descriptor_ops;
mod file_info;
pub mod keychain;
pub mod linux;
mod stat;
pub mod sysctl;
pub mod terminal;
pub mod volume_subpath;

pub use self::file_info::FileInfo;
pub use self::stat::Stat;
pub use self::stat::TimeSpec;
pub use self::terminal::Terminal;
