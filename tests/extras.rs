#![cfg(feature = "integration")]

//! `ContainerizationExtras`' `ProgressEvent` and `ProxyUtils`, each asked of
//! Swift.

use cfw::containerization_extras as extras;
use containerization_framework as cfw;
use std::collections::BTreeMap;

#[test]
fn names_progress_events_as_swift_does() {
  let events = [
    (extras::ProgressEvent::AddItems(1), "add-items", 1),
    (extras::ProgressEvent::AddTotalItems(2), "add-total-items", 2),
    (extras::ProgressEvent::AddSize(3), "add-size", 3),
    (extras::ProgressEvent::AddTotalSize(4), "add-total-size", 4),
  ];

  for (event, name, value) in events {
    assert_eq!(event.event().expect("a name"), name);
    assert_eq!(event.value(), value);
  }
}

fn env(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
  entries
    .iter()
    .map(|(key, value)| (key.to_string(), value.to_string()))
    .collect()
}

#[test]
fn finds_the_proxy_for_a_scheme_in_the_environment_given() {
  let env = env(&[
    ("HTTP_PROXY", "http://proxy.test:3128"),
    ("HTTPS_PROXY", "http://secure.test:3129"),
    ("NO_PROXY", ".internal.test,localhost"),
  ]);

  assert_eq!(
    extras::proxy_utils::proxy_from_environment(Some("http"), "example.com", Some(&env)).expect("an answer"),
    Some("http://proxy.test:3128".to_string())
  );
  assert_eq!(
    extras::proxy_utils::proxy_from_environment(Some("https"), "example.com", Some(&env)).expect("an answer"),
    Some("http://secure.test:3129".to_string())
  );
  assert_eq!(
    extras::proxy_utils::proxy_from_environment(Some("https"), "registry.internal.test", Some(&env))
      .expect("an answer"),
    None
  );
  assert_eq!(
    extras::proxy_utils::proxy_from_environment(None, "example.com", Some(&env)).expect("an answer"),
    None
  );
}

#[test]
fn finds_no_proxy_in_an_empty_environment() {
  assert_eq!(
    extras::proxy_utils::proxy_from_environment(Some("https"), "example.com", Some(&BTreeMap::new()))
      .expect("an answer"),
    None
  );
}
