//! Getters for the addresses Swift reads, and the addresses, prefixes and CIDR
//! blocks Rust builds from an outcome.

use crate::bridge::ffi;
use crate::containerization_extras;

impl containerization_extras::IPv6Address {
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

impl containerization_extras::IpAddress {
  pub(crate) fn holds_v6(&self) -> bool {
    matches!(self, Self::V6(_))
  }

  pub(crate) fn v4_value(&self) -> u32 {
    match self {
      Self::V4(address) => address.value,
      Self::V6(_) => unreachable!("Swift asks for an IPv4 value only after `holds_v6`"),
    }
  }

  pub(crate) fn v6(&self) -> &containerization_extras::IPv6Address {
    match self {
      Self::V6(address) => address,
      Self::V4(_) => unreachable!("Swift asks for an IPv6 address only after `holds_v6`"),
    }
  }
}

impl ffi::CzOutcome {
  pub(crate) fn ipv4_address(&self) -> containerization_extras::IPv4Address {
    containerization_extras::IPv4Address::new(self.ipv4_value())
  }

  pub(crate) fn ipv6_address(&self) -> containerization_extras::IPv6Address {
    containerization_extras::IPv6Address::from_halves(self.ipv6_high(), self.ipv6_low(), self.ipv6_zone())
  }

  pub(crate) fn ip_address(&self) -> containerization_extras::IpAddress {
    if self.is_ipv6() {
      containerization_extras::IpAddress::V6(self.ipv6_address())
    } else {
      containerization_extras::IpAddress::V4(self.ipv4_address())
    }
  }

  pub(crate) fn prefix(&self) -> containerization_extras::Prefix {
    containerization_extras::Prefix {
      length: self.prefix_length(),
    }
  }

  pub(crate) fn cidr_v4(&self) -> containerization_extras::CIDRv4 {
    containerization_extras::CIDRv4 {
      address: self.ipv4_address(),
      prefix: self.prefix(),
    }
  }

  pub(crate) fn cidr_v6(&self) -> containerization_extras::CIDRv6 {
    containerization_extras::CIDRv6 {
      address: self.ipv6_address(),
      prefix: self.prefix(),
    }
  }

  pub(crate) fn cidr(&self) -> containerization_extras::Cidr {
    if self.is_ipv6() {
      containerization_extras::Cidr::V6(self.ipv6_address(), self.prefix())
    } else {
      containerization_extras::Cidr::V4(self.ipv4_address(), self.prefix())
    }
  }

  pub(crate) fn mac_address(&self) -> containerization_extras::MACAddress {
    containerization_extras::MACAddress {
      value: self.mac_value(),
    }
  }

  /// The `IPv4Address?` an outcome holds.
  pub(crate) fn optional_ipv4_address(&self) -> Option<containerization_extras::IPv4Address> {
    self.optional(Self::ipv4_address)
  }

  /// The `IPv6Address?` an outcome holds.
  pub(crate) fn optional_ipv6_address(&self) -> Option<containerization_extras::IPv6Address> {
    self.optional(Self::ipv6_address)
  }

  /// The `CIDRv6?` an outcome holds.
  pub(crate) fn optional_cidr_v6(&self) -> Option<containerization_extras::CIDRv6> {
    self.optional(Self::cidr_v6)
  }

  /// The `MACAddress?` an outcome holds.
  pub(crate) fn optional_mac_address(&self) -> Option<containerization_extras::MACAddress> {
    self.optional(Self::mac_address)
  }

  pub(crate) fn uint128(&self) -> u128 {
    u128::from(self.wide_high()) << 64 | u128::from(self.wide_low())
  }
}
