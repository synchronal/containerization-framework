//! The OCI image format: indexes, manifests, image configs, the descriptors
//! that point at them, and the references and platforms that pick them.
//! [`Image`] is an image's config, not `containerization::image::Image`.

mod annotation_keys;
mod descriptor;
mod image_config;
mod index;
mod manifest;
mod media_types;
pub(crate) mod platform;
pub mod reference;

pub use self::annotation_keys::AnnotationKeys;
pub use self::descriptor::Descriptor;
pub use self::image_config::History;
pub use self::image_config::Image;
pub use self::image_config::ImageConfig;
pub use self::image_config::Rootfs;
pub use self::index::Index;
pub use self::manifest::Manifest;
pub use self::media_types::MediaTypes;
pub use self::platform::Platform;
pub use self::reference::Reference;
