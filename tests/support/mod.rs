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

/// Enough to run a shell; `.config/nextest.toml` caps how many run at once.
const TEST_CPUS: u32 = 1;
const TEST_MEMORY_IN_BYTES: u64 = 512 * 1024 * 1024;

/// The rootfs ceiling, against Containerization's 8 GiB default: nothing here
/// writes more than a marker file.
const TEST_ROOTFS_SIZE_IN_BYTES: u64 = 1024 * 1024 * 1024;

/// A VM for the test limits plus guest overhead.
pub fn vm() -> cfw::containerization::VmResources {
  cfw::containerization::VmResources {
    cpus: TEST_CPUS,
    memory_in_bytes: TEST_MEMORY_IN_BYTES + cfw::containerization::VmResources::GUEST_MEMORY_OVERHEAD,
  }
}
