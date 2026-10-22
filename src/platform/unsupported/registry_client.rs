//! Stand-ins for `RegistryClient.swift`.

use super::CzAuthentication;
use super::CzOutcome;
use crate::containerization_oci;
use crate::platform;
use std::convert::Infallible;

taken!(registry_client -> CzRegistryClient);

handles!(CzRegistryClient);

failing!(
  cz_registry_client_new(&str, bool, CzAuthentication),
  cz_registry_client_with_host(
    &str,
    Option<String>,
    Option<u16>,
    CzAuthentication,
    Option<String>,
    bool,
    isize,
    u64,
    usize,
  ),
);

uninhabited!(CzRegistryClient {
  fn ping() -> CzOutcome;
  fn resolve(&str, &str) -> CzOutcome;
  fn fetch_data(&str, containerization_oci::image::Descriptor) -> CzOutcome;
  fn fetch_blob(&str, containerization_oci::image::Descriptor, &str, platform::Progress) -> CzOutcome;
  fn catalog(Option<String>) -> CzOutcome;
  fn referrers(&str, &str, Option<String>) -> CzOutcome;
});
