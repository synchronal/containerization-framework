use super::RegistryInfo;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::fmt;
use std::time::SystemTime;

/// `KeychainQueryResult`.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct KeychainQueryResult {
  pub username: String,
  pub password: String,
  pub modified_date: SystemTime,
  pub created_date: SystemTime,
}

/// Leaves out the password.
impl fmt::Debug for KeychainQueryResult {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("KeychainQueryResult")
      .field("username", &self.username)
      .field("modified_date", &self.modified_date)
      .field("created_date", &self.created_date)
      .finish_non_exhaustive()
  }
}

/// `KeychainQuery`, which reads and writes internet passwords in the macOS
/// keychain. Swift's `accessGroup` arguments default to `nil`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct KeychainQuery;

impl KeychainQuery {
  /// `KeychainQuery()`.
  pub fn new() -> Self {
    Self
  }

  /// `save(securityDomain:accessGroup:hostname:username:password:)`, which
  /// replaces any entry for `hostname`.
  pub fn save(
    &self,
    security_domain: &str,
    access_group: Option<&str>,
    hostname: &str,
    username: &str,
    password: &str,
  ) -> Result<(), Error> {
    platform::outcome(
      ffi::cz_keychain_query_save(
        security_domain,
        access_group.map(str::to_string),
        hostname,
        username,
        password,
      ),
      format!("save {hostname} to the keychain"),
    )
    .map(drop)
  }

  /// `delete(securityDomain:accessGroup:hostname:)`, which succeeds when there
  /// is no entry.
  pub fn delete(&self, security_domain: &str, access_group: Option<&str>, hostname: &str) -> Result<(), Error> {
    platform::outcome(
      ffi::cz_keychain_query_delete(security_domain, access_group.map(str::to_string), hostname),
      format!("delete {hostname} from the keychain"),
    )
    .map(drop)
  }

  /// `get(securityDomain:accessGroup:hostname:)`.
  pub fn get(
    &self,
    security_domain: &str,
    access_group: Option<&str>,
    hostname: &str,
  ) -> Result<Option<KeychainQueryResult>, Error> {
    platform::outcome(
      ffi::cz_keychain_query_get(security_domain, access_group.map(str::to_string), hostname),
      format!("read {hostname} from the keychain"),
    )
    .map(|outcome| outcome.is_some().then(|| outcome.keychain_query_result()))
  }

  /// `list(securityDomain:accessGroup:)`.
  pub fn list(&self, security_domain: &str, access_group: Option<&str>) -> Result<Vec<RegistryInfo>, Error> {
    platform::outcome(
      ffi::cz_keychain_query_list(security_domain, access_group.map(str::to_string)),
      format!("list the keychain's entries for {security_domain}"),
    )
    .map(|outcome| outcome.registry_infos())
  }

  /// `exists(securityDomain:accessGroup:hostname:)`.
  pub fn exists(&self, security_domain: &str, access_group: Option<&str>, hostname: &str) -> Result<bool, Error> {
    platform::outcome(
      ffi::cz_keychain_query_exists(security_domain, access_group.map(str::to_string), hostname),
      format!("look for {hostname} in the keychain"),
    )
    .map(|outcome| outcome.boolean())
  }
}
