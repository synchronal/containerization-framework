use super::IPv6Address;
use super::boolean;
use super::description;
use super::ordering;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::cmp::Ordering;

/// `MACAddress`. Only Swift makes one, so its value is one Swift masked to 48
/// bits.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MACAddress {
  pub(crate) value: u64,
}

impl MACAddress {
  /// `MACAddress(_ value: UInt64)`, which drops the top 16 bits.
  pub fn new(value: u64) -> Result<Self, Error> {
    platform::outcome(ffi::cz_mac_address_new(value), "make a MAC address").map(|outcome| outcome.mac_address())
  }

  /// `MACAddress(_ bytes: [UInt8])`.
  pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_mac_address_from_bytes(bytes.to_vec()),
      format!("read {bytes:?} as a MAC address"),
    )
    .map(|outcome| outcome.mac_address())
  }

  /// `MACAddress(_ string: String)`.
  pub fn parse(string: &str) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_mac_address_parse(string),
      format!("parse {string:?} as a MAC address"),
    )
    .map(|outcome| outcome.mac_address())
  }

  /// `MACAddress.value`.
  pub fn value(&self) -> u64 {
    self.value
  }

  /// `MACAddress.bytes`.
  pub fn bytes(&self) -> Result<Vec<u8>, Error> {
    platform::outcome(ffi::cz_mac_address_bytes(self.value), "read a MAC address's bytes")
      .map(|outcome| outcome.bytes())
  }

  /// `MACAddress.description`.
  pub fn description(&self) -> Result<String, Error> {
    description(ffi::cz_mac_address_description(self.value), "describe a MAC address")
  }

  /// `MACAddress.isLocallyAdministered`.
  pub fn is_locally_administered(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_mac_address_is_locally_administered(self.value),
      "check whether a MAC address is locally administered",
    )
  }

  /// `MACAddress.isMulticast`.
  pub fn is_multicast(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_mac_address_is_multicast(self.value),
      "check whether a MAC address is multicast",
    )
  }

  /// `MACAddress.ipv6Address(network:)`.
  pub fn ipv6_address(&self, network: &IPv6Address) -> Result<IPv6Address, Error> {
    platform::outcome(
      ffi::cz_mac_address_ipv6_address(self.value, network.clone()),
      "make a MAC address's link-local IPv6 address",
    )
    .map(|outcome| outcome.ipv6_address())
  }
}

impl PartialOrd for MACAddress {
  /// `MACAddress.<`. `None` off macOS, where Swift can't be asked.
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    ordering(self, other, |lhs, rhs| {
      boolean(
        ffi::cz_mac_address_less_than(lhs.value, rhs.value),
        "compare MAC addresses",
      )
    })
  }
}
