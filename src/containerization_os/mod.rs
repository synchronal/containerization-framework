//! Containerization's `ContainerizationOS` module: the parts that run on a
//! macOS host. The keychain types are in [`keychain`], and binfmt_misc and
//! capabilities are in [`linux`], as they are in Swift's `Keychain` and `Linux`
//! directories.

pub mod file;
pub mod keychain;
pub mod linux;
mod stat;
pub mod sysctl;
pub mod terminal;

pub use self::file::FileInfo;
pub use self::stat::Stat;
pub use self::stat::TimeSpec;
pub use self::terminal::Terminal;
