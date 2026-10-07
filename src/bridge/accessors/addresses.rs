//! Getters for the addresses, interface addresses and routes Swift reads, and
//! the addresses, prefixes and CIDR blocks Rust builds from an outcome.

use crate::bridge::ffi;
use crate::containerization_extras;
use crate::containerization_extras::address;

impl address::IPv6Address {
  /// An address whose `UInt128` value crossed as two halves.
  pub(crate) fn from_halves(high: u64, low: u64, zone: Option<String>) -> Self {
    Self::new(u128::from(high) << 64 | u128::from(low), zone)
  }

  pub(crate) fn value_high(&self) -> u64 {
    (self.value >> 64) as u64
  }

  pub(crate) fn value_low(&self) -> u64 {
    self.value as u64
  }

  pub(crate) fn zone(&self) -> Option<&str> {
    self.zone.as_deref()
  }
}

impl address::IpAddress {
  pub(crate) fn holds_v6(&self) -> bool {
    matches!(self, Self::V6(_))
  }

  pub(crate) fn v4_value(&self) -> u32 {
    match self {
      Self::V4(address) => address.value,
      Self::V6(_) => unreachable!("Swift asks for an IPv4 value only after `holds_v6`"),
    }
  }

  pub(crate) fn v6(&self) -> &address::IPv6Address {
    match self {
      Self::V6(address) => address,
      Self::V4(_) => unreachable!("Swift asks for an IPv6 address only after `holds_v6`"),
    }
  }
}

impl containerization_extras::InterfaceAddress {
  pub(crate) fn ipv4_address_value(&self) -> u32 {
    self.ipv4_address.address.value
  }

  pub(crate) fn ipv4_address_prefix(&self) -> u8 {
    self.ipv4_address.prefix.length
  }

  pub(crate) fn has_ipv6_address(&self) -> bool {
    self.ipv6_address.is_some()
  }

  pub(crate) fn ipv6_address_address(&self) -> &address::IPv6Address {
    &self.ipv6().address
  }

  pub(crate) fn ipv6_address_prefix(&self) -> u8 {
    self.ipv6().prefix.length
  }

  fn ipv6(&self) -> &address::CIDRv6 {
    self
      .ipv6_address
      .as_ref()
      .expect("Swift asks for the IPv6 address only after `has_ipv6_address`")
  }
}

impl containerization_extras::LinkRoute {
  pub(crate) fn ipv4_destination(&self) -> Option<u32> {
    self.ipv4_destination.map(|address| address.value)
  }

  pub(crate) fn ipv4_source(&self) -> Option<u32> {
    self.ipv4_source.map(|address| address.value)
  }

  pub(crate) fn has_ipv6_destination(&self) -> bool {
    self.ipv6_destination.is_some()
  }

  pub(crate) fn ipv6_destination(&self) -> &address::IPv6Address {
    self
      .ipv6_destination
      .as_ref()
      .expect("Swift asks for the IPv6 destination only after `has_ipv6_destination`")
  }

  pub(crate) fn has_ipv6_source(&self) -> bool {
    self.ipv6_source.is_some()
  }

  pub(crate) fn ipv6_source(&self) -> &address::IPv6Address {
    self
      .ipv6_source
      .as_ref()
      .expect("Swift asks for the IPv6 source only after `has_ipv6_source`")
  }
}

impl containerization_extras::DefaultRoute {
  pub(crate) fn ipv4_gateway(&self) -> Option<u32> {
    self.ipv4_gateway.map(|address| address.value)
  }

  pub(crate) fn has_ipv6_gateway(&self) -> bool {
    self.ipv6_gateway.is_some()
  }

  pub(crate) fn ipv6_gateway(&self) -> &address::IPv6Address {
    self
      .ipv6_gateway
      .as_ref()
      .expect("Swift asks for the IPv6 gateway only after `has_ipv6_gateway`")
  }
}

impl ffi::CzOutcome {
  pub(crate) fn ipv4_address(&self) -> address::IPv4Address {
    address::IPv4Address::new(self.ipv4_value())
  }

  pub(crate) fn ipv6_address(&self) -> address::IPv6Address {
    address::IPv6Address::from_halves(self.ipv6_high(), self.ipv6_low(), self.ipv6_zone())
  }

  pub(crate) fn ip_address(&self) -> address::IpAddress {
    if self.is_ipv6() {
      address::IpAddress::V6(self.ipv6_address())
    } else {
      address::IpAddress::V4(self.ipv4_address())
    }
  }

  pub(crate) fn prefix(&self) -> address::Prefix {
    address::Prefix {
      length: self.prefix_length(),
    }
  }

  pub(crate) fn cidr_v4(&self) -> address::CIDRv4 {
    address::CIDRv4 {
      address: self.ipv4_address(),
      prefix: self.prefix(),
    }
  }

  pub(crate) fn cidr_v6(&self) -> address::CIDRv6 {
    address::CIDRv6 {
      address: self.ipv6_address(),
      prefix: self.prefix(),
    }
  }

  pub(crate) fn cidr(&self) -> address::Cidr {
    if self.is_ipv6() {
      address::Cidr::V6(self.ipv6_address(), self.prefix())
    } else {
      address::Cidr::V4(self.ipv4_address(), self.prefix())
    }
  }

  pub(crate) fn mac_address(&self) -> address::MACAddress {
    address::MACAddress {
      value: self.mac_value(),
    }
  }

  /// The `IPv4Address?` an outcome holds.
  pub(crate) fn optional_ipv4_address(&self) -> Option<address::IPv4Address> {
    self.optional(Self::ipv4_address)
  }

  /// The `IPv6Address?` an outcome holds.
  pub(crate) fn optional_ipv6_address(&self) -> Option<address::IPv6Address> {
    self.optional(Self::ipv6_address)
  }

  /// The `CIDRv6?` an outcome holds.
  pub(crate) fn optional_cidr_v6(&self) -> Option<address::CIDRv6> {
    self.optional(Self::cidr_v6)
  }

  /// The `MACAddress?` an outcome holds.
  pub(crate) fn optional_mac_address(&self) -> Option<address::MACAddress> {
    self.optional(Self::mac_address)
  }

  pub(crate) fn uint128(&self) -> u128 {
    u128::from(self.wide_high()) << 64 | u128::from(self.wide_low())
  }
}
