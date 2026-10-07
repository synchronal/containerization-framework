//! The macOS keychain, where registry credentials are kept.

mod keychain_query;
mod registry_info;

pub use self::keychain_query::KeychainQuery;
pub use self::keychain_query::KeychainQueryResult;
pub use self::registry_info::RegistryInfo;
