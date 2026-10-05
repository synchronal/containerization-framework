//! Containerization's `ContainerizationOCI` module.
//!
//! Each type wraps the Swift type of the same name, and its methods are the
//! Swift methods written in snake case.

mod content;
mod content_writer;
mod descriptor;
mod local_content_store;
mod platform;
mod user;

pub use self::content::Content;
pub use self::content_writer::ContentWriter;
pub use self::descriptor::Descriptor;
pub use self::local_content_store::LocalContentStore;
pub use self::platform::Platform;
pub use self::user::User;
