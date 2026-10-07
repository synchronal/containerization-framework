use super::IPv4Address;
use super::Prefix;
use super::boolean;
use super::description;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `CIDRv4`. Only Swift makes one, so its prefix is one Swift accepted.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CIDRv4 {
  pub(crate) address: IPv4Address,
  pub(crate) prefix: Prefix,
}

impl CIDRv4 {
  /// `CIDRv4(_ cidr: String)`.
  pub fn parse(cidr: &str) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_cidr_v4_parse(cidr),
      format!("parse {cidr:?} as an IPv4 CIDR block"),
    )
    .map(|outcome| outcome.cidr_v4())
  }

  /// `CIDRv4(_ address:prefix:)`.
  pub fn new(address: IPv4Address, prefix: Prefix) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_cidr_v4_new(address.value, prefix.length),
      "make an IPv4 CIDR block",
    )
    .map(|outcome| outcome.cidr_v4())
  }

  /// `CIDRv4(lower:upper:)`.
  pub fn from_range(lower: IPv4Address, upper: IPv4Address) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_cidr_v4_from_range(lower.value, upper.value),
      "make an IPv4 CIDR block from a range",
    )
    .map(|outcome| outcome.cidr_v4())
  }

  /// `CIDRv4.address`.
  pub fn address(&self) -> IPv4Address {
    self.address
  }

  /// `CIDRv4.prefix`.
  pub fn prefix(&self) -> Prefix {
    self.prefix
  }

  /// `CIDRv4.lower`.
  pub fn lower(&self) -> Result<IPv4Address, Error> {
    self.address_of(
      ffi::cz_cidr_v4_lower(self.address.value, self.prefix.length),
      "read the lowest address of an IPv4 CIDR block",
    )
  }

  /// `CIDRv4.upper`.
  pub fn upper(&self) -> Result<IPv4Address, Error> {
    self.address_of(
      ffi::cz_cidr_v4_upper(self.address.value, self.prefix.length),
      "read the highest address of an IPv4 CIDR block",
    )
  }

  /// `CIDRv4.gateway`.
  pub fn gateway(&self) -> Result<IPv4Address, Error> {
    self.address_of(
      ffi::cz_cidr_v4_gateway(self.address.value, self.prefix.length),
      "read the gateway of an IPv4 CIDR block",
    )
  }

  fn address_of(&self, outcome: ffi::CzOutcome, action: &str) -> Result<IPv4Address, Error> {
    platform::outcome(outcome, action).map(|outcome| outcome.ipv4_address())
  }

  /// `CIDRv4.contains(_:)`.
  pub fn contains(&self, ip: IPv4Address) -> Result<bool, Error> {
    boolean(
      ffi::cz_cidr_v4_contains(self.address.value, self.prefix.length, ip.value),
      "check whether an IPv4 CIDR block contains an address",
    )
  }

  /// `CIDRv4.description`.
  pub fn description(&self) -> Result<String, Error> {
    description(
      ffi::cz_cidr_v4_description(self.address.value, self.prefix.length),
      "describe an IPv4 CIDR block",
    )
  }
}
