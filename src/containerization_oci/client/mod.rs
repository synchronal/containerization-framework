//! Registry clients, and the credentials they authenticate with.

pub(crate) mod authentication;
mod keychain_helper;
pub mod registry_client;

pub use self::authentication::Authentication;
pub use self::keychain_helper::KeychainHelper;
pub use self::registry_client::RegistryClient;
pub use self::registry_client::RetryOptions;
