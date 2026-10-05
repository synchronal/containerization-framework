use std::time::SystemTime;

/// `RegistryInfo`, which [`crate::containerization_oci::KeychainHelper::list`]
/// returns.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RegistryInfo {
  pub hostname: String,
  pub username: String,
  pub modified_date: SystemTime,
  pub created_date: SystemTime,
}
