//! Containerization's `ContainerizationOS` module: the parts that run on a
//! macOS host. The keychain types are in [`keychain`], as they are in Swift's
//! `Keychain` directory.

mod capability_name;
mod capability_set;
pub mod file;
pub mod keychain;
mod stat;
pub mod sysctl;
pub mod terminal;

pub use self::capability_name::CapabilityName;
pub use self::capability_set::CapabilitySet;
pub use self::file::FileInfo;
pub use self::stat::Stat;
pub use self::stat::TimeSpec;
pub use self::terminal::Terminal;
