//! Getters and setters for Containerization's value types.
//!
//! The helpers the container, pod and VM configurations share are re-exported
//! here for the other accessors to use.

mod configuration;
mod manager;
mod network;
mod process;
mod statistics;
mod vm;

pub(super) use configuration::custom_seccomp;
pub(super) use configuration::seccomp_mode;
pub(super) use configuration::seccomp_profile;
pub(super) use network::dns;
pub(super) use network::hosts;
pub(super) use network::interface_kind;
pub(super) use network::nat;
pub(super) use network::nat_interface;
pub(super) use network::push_hosts_entry;
pub(super) use network::set_ipv6_address;
pub(super) use network::set_ipv6_gateway;
pub(super) use network::vmnet;
pub(super) use network::vmnet_interface;
pub(super) use vm::boot_log_file;

use crate::bridge::ffi;
use std::collections::BTreeMap;

impl ffi::CzOutcome {
  /// The `Int32?` an outcome holds.
  pub(crate) fn optional_int32(&self) -> Option<i32> {
    self.optional(Self::int32)
  }

  /// A held `[String: Int32]`.
  pub(crate) fn int32_map(&self) -> BTreeMap<String, i32> {
    self.map_of(Self::int32)
  }
}
