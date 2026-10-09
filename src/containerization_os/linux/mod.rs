//! Linux's binfmt_misc entries and capabilities, as in Swift's `Linux`
//! directory.

pub mod binfmt;
mod capability_name;
mod capability_set;

pub use self::capability_name::CapabilityName;
pub use self::capability_set::CapabilitySet;
