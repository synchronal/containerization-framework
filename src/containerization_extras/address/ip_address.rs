use super::IPv4Address;
use super::IPv6Address;
use super::boolean;
use super::description;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `IPAddress`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum IPAddress {
  V4(IPv4Address),
  V6(IPv6Address),
}

impl IPAddress {
  /// `IPAddress(_ string: String)`.
  pub fn parse(string: &str) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_ip_address_parse(string),
      format!("parse {string:?} as an IP address"),
    )
    .map(|outcome| outcome.ip_address())
  }

  /// `IPAddress.description`.
  pub fn description(&self) -> Result<String, Error> {
    description(ffi::cz_ip_address_description(self.clone()), "describe an IP address")
  }

  /// `IPAddress.isV4`.
  pub fn is_v4(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_ip_address_is_v4(self.clone()),
      "check whether an IP address is IPv4",
    )
  }

  /// `IPAddress.isV6`.
  pub fn is_v6(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_ip_address_is_v6(self.clone()),
      "check whether an IP address is IPv6",
    )
  }

  /// `IPAddress.ipv4`.
  pub fn ipv4(&self) -> Result<Option<IPv4Address>, Error> {
    platform::outcome(ffi::cz_ip_address_ipv4(self.clone()), "read an IP address as IPv4")
      .map(|outcome| outcome.is_some().then(|| outcome.ipv4_address()))
  }

  /// `IPAddress.ipv6`.
  pub fn ipv6(&self) -> Result<Option<IPv6Address>, Error> {
    platform::outcome(ffi::cz_ip_address_ipv6(self.clone()), "read an IP address as IPv6")
      .map(|outcome| outcome.is_some().then(|| outcome.ipv6_address()))
  }

  /// `IPAddress.isLoopback`.
  pub fn is_loopback(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_ip_address_is_loopback(self.clone()),
      "check whether an IP address is loopback",
    )
  }

  /// `IPAddress.isMulticast`.
  pub fn is_multicast(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_ip_address_is_multicast(self.clone()),
      "check whether an IP address is multicast",
    )
  }

  /// `IPAddress.isUnspecified`.
  pub fn is_unspecified(&self) -> Result<bool, Error> {
    boolean(
      ffi::cz_ip_address_is_unspecified(self.clone()),
      "check whether an IP address is unspecified",
    )
  }
}
