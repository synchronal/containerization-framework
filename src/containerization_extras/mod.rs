//! Containerization's `ContainerizationExtras` module.
//!
//! Each type wraps the Swift type of the same name. The network address types
//! are in [`address`].

pub mod address;
mod progress_event;
pub mod proxy_utils;

pub use self::progress_event::ProgressEvent;
pub use self::progress_event::ProgressHandler;
