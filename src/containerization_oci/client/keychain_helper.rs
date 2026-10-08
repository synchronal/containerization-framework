use super::Authentication;
use crate::containerization_os::keychain;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `KeychainHelper`. Its prompts, which read from the terminal, aren't bound.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct KeychainHelper {
  security_domain: String,
  access_group: Option<String>,
}

impl KeychainHelper {
  /// `KeychainHelper(securityDomain:accessGroup:)`.
  pub fn new(security_domain: impl Into<String>, access_group: Option<String>) -> Self {
    Self {
      security_domain: security_domain.into(),
      access_group,
    }
  }

  /// `KeychainHelper.lookup(hostname:)`.
  pub fn lookup(&self, hostname: &str) -> Result<Authentication, Error> {
    platform::outcome(
      ffi::cz_keychain_helper_lookup(&self.security_domain, self.access_group.clone(), hostname),
      format!("look up {hostname} in the keychain"),
    )
    .map(|outcome| Authentication {
      handle: outcome.authentication(),
    })
  }

  /// `KeychainHelper.list()`.
  pub fn list(&self) -> Result<Vec<keychain::RegistryInfo>, Error> {
    platform::outcome(
      ffi::cz_keychain_helper_list(&self.security_domain, self.access_group.clone()),
      "list the keychain's registries",
    )
    .map(|outcome| outcome.registry_infos())
  }

  /// `KeychainHelper.delete(hostname:)`.
  pub fn delete(&self, hostname: &str) -> Result<(), Error> {
    platform::outcome(
      ffi::cz_keychain_helper_delete(&self.security_domain, self.access_group.clone(), hostname),
      format!("delete {hostname} from the keychain"),
    )
    .map(drop)
  }

  /// `KeychainHelper.save(hostname:username:password:)`.
  pub fn save(&self, hostname: &str, username: &str, password: &str) -> Result<(), Error> {
    platform::outcome(
      ffi::cz_keychain_helper_save(
        &self.security_domain,
        self.access_group.clone(),
        hostname,
        username,
        password,
      ),
      format!("save {hostname} to the keychain"),
    )
    .map(drop)
  }
}
