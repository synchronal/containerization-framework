//! Containerization's `ContainerizationOCI` module.
//!
//! Each type wraps the Swift type of the same name, and its methods are the
//! Swift methods written in snake case.

mod annotation_keys;
mod content;
mod content_writer;
mod descriptor;
mod image_config;
mod index;
mod local_content_store;
mod manifest;
mod media_types;
mod parsed_digest;
mod platform;
pub mod reference;
mod user;

pub use self::annotation_keys::AnnotationKeys;
pub use self::content::Content;
pub use self::content_writer::ContentWriter;
pub use self::descriptor::Descriptor;
pub use self::image_config::History;
pub use self::image_config::Image;
pub use self::image_config::ImageConfig;
pub use self::image_config::Rootfs;
pub use self::index::Index;
pub use self::local_content_store::LocalContentStore;
pub use self::manifest::Manifest;
pub use self::media_types::MediaTypes;
pub use self::parsed_digest::ParsedDigest;
pub use self::platform::Platform;
pub use self::reference::Reference;
pub use self::user::User;
