use super::IPv4Address;
use super::IPv6Address;
use super::IpAddress;
use super::Prefix;
use super::boolean;
use super::description;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `CIDR`. As in Swift, a case can be made directly with any prefix, and only
/// the initializers check it against the address family.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Cidr {
  V4(IPv4Address, Prefix),
  V6(IPv6Address, Prefix),
}

impl Cidr {
  /// `CIDR(_ cidr: String)`.
  pub fn parse(cidr: &str) -> Result<Self, Error> {
    platform::outcome(ffi::cz_cidr_parse(cidr), format!("parse {cidr:?} as a CIDR block")).map(|outcome| outcome.cidr())
  }

  /// `CIDR(_ address:prefix:)`.
  pub fn new(address: &IpAddress, prefix: Prefix) -> Result<Self, Error> {
    platform::outcome(ffi::cz_cidr_new(address.clone(), prefix.length), "make a CIDR block")
      .map(|outcome| outcome.cidr())
  }

  /// `CIDR(lower:upper:)`.
  pub fn from_range(lower: &IpAddress, upper: &IpAddress) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_cidr_from_range(lower.clone(), upper.clone()),
      "make a CIDR block from a range",
    )
    .map(|outcome| outcome.cidr())
  }

  /// `CIDR.address`.
  pub fn address(&self) -> Result<IpAddress, Error> {
    let (address, length) = self.parts();
    self.address_of(
      ffi::cz_cidr_address(address, length),
      "read the address of a CIDR block",
    )
  }

  /// `CIDR.prefix`.
  pub fn prefix(&self) -> Result<Prefix, Error> {
    let (address, length) = self.parts();
    platform::outcome(ffi::cz_cidr_prefix(address, length), "read the prefix of a CIDR block")
      .map(|outcome| outcome.prefix())
  }

  /// `CIDR.lower`.
  pub fn lower(&self) -> Result<IpAddress, Error> {
    let (address, length) = self.parts();
    self.address_of(
      ffi::cz_cidr_lower(address, length),
      "read the lowest address of a CIDR block",
    )
  }

  /// `CIDR.upper`.
  pub fn upper(&self) -> Result<IpAddress, Error> {
    let (address, length) = self.parts();
    self.address_of(
      ffi::cz_cidr_upper(address, length),
      "read the highest address of a CIDR block",
    )
  }

  fn address_of(&self, outcome: ffi::CzOutcome, action: &str) -> Result<IpAddress, Error> {
    platform::outcome(outcome, action).map(|outcome| outcome.ip_address())
  }

  /// `CIDR.contains(_:)`.
  pub fn contains(&self, ip: &IpAddress) -> Result<bool, Error> {
    let (address, length) = self.parts();
    boolean(
      ffi::cz_cidr_contains(address, length, ip.clone()),
      "check whether a CIDR block contains an address",
    )
  }

  /// `CIDR.description`.
  pub fn description(&self) -> Result<String, Error> {
    let (address, length) = self.parts();
    description(ffi::cz_cidr_description(address, length), "describe a CIDR block")
  }

  /// The case's address and prefix length, as they cross to Swift.
  fn parts(&self) -> (IpAddress, u8) {
    match self {
      Self::V4(address, prefix) => (IpAddress::V4(*address), prefix.length),
      Self::V6(address, prefix) => (IpAddress::V6(address.clone()), prefix.length),
    }
  }
}
