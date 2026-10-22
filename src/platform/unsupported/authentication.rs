//! Stand-ins for `Authentication.swift`: `Authentication` and
//! `KeychainHelper`.

use super::CzOutcome;
use crate::containerization_os;
use std::convert::Infallible;

taken!(
  authentication -> CzAuthentication,
  registry_infos -> Result<Vec<containerization_os::keychain::RegistryInfo>, crate::error::Error>,
);

handles!(CzAuthentication);

failing!(
  cz_basic_authentication(&str, &str),
  cz_no_authentication(),
  cz_keychain_helper_lookup(&str, Option<String>, &str),
  cz_keychain_helper_list(&str, Option<String>),
  cz_keychain_helper_delete(&str, Option<String>, &str),
  cz_keychain_helper_save(&str, Option<String>, &str, &str, &str),
);

uninhabited!(CzAuthentication {
  fn duplicate() -> CzAuthentication;
  fn token() -> CzOutcome;
});
