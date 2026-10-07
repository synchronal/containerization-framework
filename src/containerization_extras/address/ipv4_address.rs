use super::boolean;
use super::description;
use super::ordering;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::cmp::Ordering;

/// `IPv4Address`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IPv4Address {
  pub value: u32,
}

impl IPv4Address {
  /// `IPv4Address(_ value: UInt32)`.
  pub const fn new(value: u32) -> Self {
    Self { value }
  }

  /// `IPv4Address(_ bytes: [UInt8])`.
  pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_ipv4_address_from_bytes(bytes.to_vec()),
      format!("read {bytes:?} as an IPv4 address"),
    )
    .map(|outcome| outcome.ipv4_address())
  }

  /// `IPv4Address(_ string: String)`.
  pub fn parse(string: &str) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_ipv4_address_parse(string),
      format!("parse {string:?} as an IPv4 address"),
    )
    .map(|outcome| outcome.ipv4_address())
  }

  /// `IPv4Address.bytes`.
  pub fn bytes(&self) -> Result<Vec<u8>, Error> {
    platform::outcome(ffi::cz_ipv4_address_bytes(self.value), "read an IPv4 address's bytes")
      .map(|outcome| outcome.bytes())
  }

  /// `IPv4Address.description`.
  pub fn description(&self) -> Result<String, Error> {
    description(ffi::cz_ipv4_address_description(self.value), "describe an IPv4 address")
  }

  /// `IPv4Address.isUnspecified`.
  pub fn is_unspecified(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_ipv4_address_is_unspecified(self.value),
      "check whether an IPv4 address is unspecified",
    )
  }

  /// `IPv4Address.isLoopback`.
  pub fn is_loopback(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_ipv4_address_is_loopback(self.value),
      "check whether an IPv4 address is loopback",
    )
  }

  /// `IPv4Address.isMulticast`.
  pub fn is_multicast(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_ipv4_address_is_multicast(self.value),
      "check whether an IPv4 address is multicast",
    )
  }

  /// `IPv4Address.isLinkLocal`.
  pub fn is_link_local(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_ipv4_address_is_link_local(self.value),
      "check whether an IPv4 address is link-local",
    )
  }

  /// `IPv4Address.isBroadcast`.
  pub fn is_broadcast(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_ipv4_address_is_broadcast(self.value),
      "check whether an IPv4 address is broadcast",
    )
  }
}

impl PartialOrd for IPv4Address {
  /// `IPv4Address.<`. `None` off macOS, where Swift can't be asked.
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    ordering(self, other, |lhs, rhs| {
      boolean(
        ffi::cz_ipv4_address_less_than(lhs.value, rhs.value),
        "compare IPv4 addresses",
      )
    })
  }
}
