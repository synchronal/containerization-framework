#![cfg(feature = "integration")]

//! A registry client, the authentication it takes, and the credentials the
//! keychain keeps for registries.
//!
//! The client talks to the registry the suite pulls its image from, which
//! lets nothing be written and lists no catalog.

mod support;

use containerization_framework as cfw;

/// [`support::store::IMAGE`]'s repository and tag.
const NAME: &str = "library/alpine";
const TAG: &str = "3";

#[test]
fn resolves_and_fetches_an_index() {
  let client = cfw::containerization_oci::client::RegistryClient::new(support::store::IMAGE, false, None)
    .expect("a client for the suite's registry");
  let directory = tempfile::tempdir().expect("a temporary directory");

  client.ping().expect("the registry should answer");
  let descriptor = client.resolve(NAME, TAG).expect("the tag should resolve");
  let data = client
    .fetch_data(NAME, &descriptor)
    .expect("the index should fetch");
  let (size, digest) = client
    .fetch_blob(NAME, &descriptor, &directory.path().join("index"), None)
    .expect("the index should fetch into a file");

  assert_eq!(data.len() as i64, descriptor.size);
  assert_eq!(
    cfw::containerization_oci::content::ContentWriter::new(directory.path())
      .expect("a writer")
      .write(&data)
      .expect("the data should write")
      .1,
    descriptor.digest
  );
  assert_eq!((size, digest), (descriptor.size, descriptor.digest.clone()));
  assert_eq!(
    std::fs::read(directory.path().join("index")).expect("the fetched file"),
    data
  );
  client
    .referrers(NAME, &descriptor.digest, None)
    .expect("the referrers, or none");
}

#[test]
fn makes_a_client_for_a_host() {
  let client = cfw::containerization_oci::client::RegistryClient::with_host(
    "registry-1.docker.io",
    cfw::containerization_oci::client::registry_client::RegistryClientOptions {
      retry_options: Some(cfw::containerization_oci::client::RetryOptions::new(1, 1_000_000)),
      ..Default::default()
    },
  )
  .expect("a client for the host");

  assert!(client.resolve(NAME, TAG).is_ok());
}

#[test]
fn says_which_reference_it_could_not_make_a_client_for() {
  let error = cfw::containerization_oci::client::RegistryClient::new("Not A Reference", false, None)
    .err()
    .expect("a malformed reference");

  assert!(error.to_string().contains("Not A Reference"), "{error}");
}

/// Kept apart from any domain a real tool saves credentials under.
const SECURITY_DOMAIN: &str = "dev.reflective.containerization-framework.tests";

#[test]
fn makes_a_basic_token() {
  let auth = cfw::containerization_oci::client::Authentication::basic("user", "password").expect("an authentication");

  assert_eq!(
    auth.token().expect("a token"),
    "Basic dXNlcjpwYXNzd29yZA==",
    "base64 of `user:password`"
  );
  assert_eq!(
    auth.clone().token().expect("a copy's token"),
    auth.token().expect("a token")
  );
}

/// Writes to the login keychain, under a domain of its own, and removes what
/// it wrote.
#[test]
fn saves_looks_up_lists_and_deletes_credentials() {
  let helper = cfw::containerization_oci::client::KeychainHelper::new(SECURITY_DOMAIN, None);
  let hostname = "registry.example.test";

  helper
    .save(hostname, "user", "password")
    .expect("the credentials should save");
  let looked_up = helper.lookup(hostname);
  let listed = helper.list();
  helper
    .delete(hostname)
    .expect("the credentials should delete");

  assert_eq!(
    looked_up
      .expect("the saved credentials")
      .token()
      .expect("a token"),
    "Basic dXNlcjpwYXNzd29yZA=="
  );
  let listed = listed.expect("the saved registries");
  let info = listed
    .iter()
    .find(|info| info.hostname == hostname)
    .unwrap_or_else(|| panic!("{hostname} should be listed: {listed:?}"));
  assert_eq!(info.username, "user");
  assert!(info.created_date <= info.modified_date, "{info:?}");
  assert!(helper.lookup(hostname).is_err(), "deleted credentials should be gone");
}
