//! Readers for what ContainerizationOS returns.

use crate::bridge::ffi;
use crate::containerization_os;
use crate::platform;

impl ffi::CzOutcome {
  /// The `KeychainQueryResult` an outcome holds, its dates crossing as
  /// seconds since 1970.
  pub(crate) fn keychain_query_result(&self) -> containerization_os::KeychainQueryResult {
    containerization_os::KeychainQueryResult {
      username: self.keychain_query_result_username(),
      password: self.keychain_query_result_password(),
      modified_date: platform::system_time(self.keychain_query_result_modified_date()),
      created_date: platform::system_time(self.keychain_query_result_created_date()),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::bridge::ffi;
  use crate::containerization_os;

  #[test]
  fn copies_swifts_capability_names() {
    let names = containerization_os::CapabilityName::ALL_CASES;

    assert_eq!(
      ffi::cz_capability_name_descriptions(),
      names
        .iter()
        .map(|name| name.description())
        .collect::<Vec<_>>()
    );
    assert_eq!(
      ffi::cz_capability_name_cap_values(),
      names
        .iter()
        .map(|name| name.cap_value())
        .collect::<Vec<_>>()
    );
  }

  #[test]
  fn copies_swifts_capability_sets() {
    assert_eq!(
      ffi::cz_capability_set_descriptions(),
      containerization_os::CapabilitySet::ALL
        .iter()
        .map(|set| set.description())
        .collect::<Vec<_>>()
    );
  }
}
