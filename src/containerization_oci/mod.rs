//! Containerization's `ContainerizationOCI` module.
//!
//! Each type wraps the Swift type of the same name, and its methods are the
//! Swift methods written in snake case. The types are grouped as the OCI
//! specifications are: the image format, content and the stores it is kept
//! in, registry clients, and the runtime spec.

pub mod client;
pub mod content;
pub mod image;
pub mod runtime;
