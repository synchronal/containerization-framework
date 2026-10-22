//! What the integration tests share: a store to boot from, and the guards that
//! keep concurrent tests apart.
//!
//! The store is filled once and kept between runs. What a test owns is its
//! container, which goes when the test does.
//!
//! The entitlement these need is `bin/dev/test-integration`'s business.

#![allow(dead_code)]

pub mod container;
pub mod lock;
pub mod network;
pub mod store;

use containerization_framework as cfw;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;

/// Enough to run a shell; `.config/nextest.toml` caps how many run at once.
pub const TEST_CPUS: u32 = 1;
pub const TEST_MEMORY_IN_BYTES: u64 = 512 * 1024 * 1024;

/// The rootfs ceiling, against Containerization's 8 GiB default: nothing here
/// writes more than a marker file.
pub const TEST_ROOTFS_SIZE_IN_BYTES: u64 = 1024 * 1024 * 1024;

/// A VM for the test limits plus guest overhead.
pub fn vm() -> cfw::containerization::vm::VMResources {
  cfw::containerization::vm::VMResources {
    cpus: TEST_CPUS,
    memory_in_bytes: TEST_MEMORY_IN_BYTES + cfw::containerization::vm::VMResources::GUEST_MEMORY_OVERHEAD,
  }
}

/// A VM configuration with [`vm`]'s resources and Swift's other defaults.
pub fn vm_configuration() -> cfw::containerization::vm::VMConfiguration {
  cfw::containerization::vm::VMConfiguration {
    cpus: TEST_CPUS,
    memory_in_bytes: vm().memory_in_bytes,
    ..Default::default()
  }
}

/// Each value as an owned string.
pub fn strings(values: &[&str]) -> Vec<String> {
  values.iter().map(|value| value.to_string()).collect()
}

/// Each entry as an owned key and value.
pub fn map(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
  entries
    .iter()
    .map(|(key, value)| (key.to_string(), value.to_string()))
    .collect()
}

/// The platform that `string` names, which must be valid.
pub fn platform(string: &str) -> cfw::containerization_oci::image::Platform {
  cfw::containerization_oci::image::Platform::parse(string).expect("a valid platform")
}

/// The items that `events` report done, and the total items they announce.
pub fn item_totals(events: &[cfw::containerization_extras::ProgressEvent]) -> (isize, isize) {
  events
    .iter()
    .fold((0, 0), |(items, total), event| match event {
      cfw::containerization_extras::ProgressEvent::AddItems(added) => (items + added, total),
      cfw::containerization_extras::ProgressEvent::AddTotalItems(added) => (items, total + added),
      _ => (items, total),
    })
}

/// A progress handler, and every event it is called with, in order.
pub fn recorded() -> (
  Arc<Mutex<Vec<cfw::containerization_extras::ProgressEvent>>>,
  cfw::containerization_extras::ProgressHandler,
) {
  let events = Arc::new(Mutex::new(Vec::new()));
  let recording = Arc::clone(&events);

  (
    events,
    Box::new(move |batch| {
      recording
        .lock()
        .expect("a recording")
        .extend_from_slice(batch)
    }),
  )
}
