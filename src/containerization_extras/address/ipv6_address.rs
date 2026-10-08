use super::boolean;
use super::description;
use super::ordering;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::cmp::Ordering;

/// `IPv6Address`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct IPv6Address {
  pub value: u128,
  pub zone: Option<String>,
}

impl IPv6Address {
  /// `IPv6Address(_ value: UInt128, zone:)`.
  pub const fn new(value: u128, zone: Option<String>) -> Self {
    Self { value, zone }
  }

  /// `IPv6Address(_ address: String)`.
  pub fn parse(address: &str) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_ipv6_address_parse(address),
      format!("parse {address:?} as an IPv6 address"),
    )
    .map(|outcome| outcome.ipv6_address())
  }

  /// `IPv6Address(_ bytes: [UInt8], zone:)`.
  pub fn from_bytes(bytes: &[u8], zone: Option<&str>) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_ipv6_address_from_bytes(bytes.to_vec(), zone.map(str::to_string)),
      format!("read {bytes:?} as an IPv6 address"),
    )
    .map(|outcome| outcome.ipv6_address())
  }

  /// `IPv6Address.unspecified`.
  pub fn unspecified() -> Result<Self, Error> {
    platform::outcome(ffi::cz_ipv6_address_unspecified(), "read the unspecified IPv6 address")
      .map(|outcome| outcome.ipv6_address())
  }

  /// `IPv6Address.loopback`.
  pub fn loopback() -> Result<Self, Error> {
    platform::outcome(ffi::cz_ipv6_address_loopback(), "read the loopback IPv6 address")
      .map(|outcome| outcome.ipv6_address())
  }

  /// `IPv6Address.bytes`.
  pub fn bytes(&self) -> Result<Vec<u8>, Error> {
    platform::outcome(ffi::cz_ipv6_address_bytes(self.clone()), "read an IPv6 address's bytes")
      .map(|outcome| outcome.bytes())
  }

  /// `IPv6Address.description`.
  pub fn description(&self) -> Result<String, Error> {
    description(
      ffi::cz_ipv6_address_description(self.clone()),
      "describe an IPv6 address",
    )
  }

  /// `IPv6Address.isUnspecified`.
  pub fn is_unspecified(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_ipv6_address_is_unspecified(self.clone()),
      "check whether an IPv6 address is unspecified",
    )
  }

  /// `IPv6Address.isLoopback`.
  pub fn is_loopback(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_ipv6_address_is_loopback(self.clone()),
      "check whether an IPv6 address is loopback",
    )
  }

  /// `IPv6Address.isMulticast`.
  pub fn is_multicast(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_ipv6_address_is_multicast(self.clone()),
      "check whether an IPv6 address is multicast",
    )
  }

  /// `IPv6Address.isLinkLocal`.
  pub fn is_link_local(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_ipv6_address_is_link_local(self.clone()),
      "check whether an IPv6 address is link-local",
    )
  }

  /// `IPv6Address.isUniqueLocal`.
  pub fn is_unique_local(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_ipv6_address_is_unique_local(self.clone()),
      "check whether an IPv6 address is unique local",
    )
  }

  /// `IPv6Address.isGlobalUnicast`.
  pub fn is_global_unicast(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_ipv6_address_is_global_unicast(self.clone()),
      "check whether an IPv6 address is global unicast",
    )
  }

  /// `IPv6Address.isDocumentation`.
  pub fn is_documentation(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_ipv6_address_is_documentation(self.clone()),
      "check whether an IPv6 address is for documentation",
    )
  }
}

impl PartialOrd for IPv6Address {
  /// `IPv6Address.<`. It panics off macOS, where Swift can't be asked. Swift
  /// orders an absent zone as an empty one, so two addresses that differ only
  /// in that way are unordered.
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    ordering(self, other, |lhs, rhs| {
      boolean(
        ffi::cz_ipv6_address_less_than(lhs.clone(), rhs.clone()),
        "compare IPv6 addresses",
      )
    })
  }
}
