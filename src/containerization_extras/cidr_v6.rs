use super::IPv6Address;
use super::Prefix;
use super::boolean;
use super::description;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `CIDRv6`. Only Swift makes one, so its prefix is one Swift accepted.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CIDRv6 {
  pub(crate) address: IPv6Address,
  pub(crate) prefix: Prefix,
}

impl CIDRv6 {
  /// `CIDRv6(_ cidr: String)`.
  pub fn parse(cidr: &str) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_cidr_v6_parse(cidr),
      format!("parse {cidr:?} as an IPv6 CIDR block"),
    )
    .map(|outcome| outcome.cidr_v6())
  }

  /// `CIDRv6(_ address:prefix:)`.
  pub fn new(address: &IPv6Address, prefix: Prefix) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_cidr_v6_new(address.clone(), prefix.length),
      "make an IPv6 CIDR block",
    )
    .map(|outcome| outcome.cidr_v6())
  }

  /// `CIDRv6(lower:upper:)`.
  pub fn from_range(lower: &IPv6Address, upper: &IPv6Address) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_cidr_v6_from_range(lower.clone(), upper.clone()),
      "make an IPv6 CIDR block from a range",
    )
    .map(|outcome| outcome.cidr_v6())
  }

  /// `CIDRv6.address`.
  pub fn address(&self) -> &IPv6Address {
    &self.address
  }

  /// `CIDRv6.prefix`.
  pub fn prefix(&self) -> Prefix {
    self.prefix
  }

  /// `CIDRv6.lower`.
  pub fn lower(&self) -> Result<IPv6Address, Error> {
    self.address_of(
      ffi::cz_cidr_v6_lower(self.address.clone(), self.prefix.length),
      "read the lowest address of an IPv6 CIDR block",
    )
  }

  /// `CIDRv6.upper`.
  pub fn upper(&self) -> Result<IPv6Address, Error> {
    self.address_of(
      ffi::cz_cidr_v6_upper(self.address.clone(), self.prefix.length),
      "read the highest address of an IPv6 CIDR block",
    )
  }

  /// `CIDRv6.gateway`, which Containerization adds.
  pub fn gateway(&self) -> Result<IPv6Address, Error> {
    self.address_of(
      ffi::cz_cidr_v6_gateway(self.address.clone(), self.prefix.length),
      "read the gateway of an IPv6 CIDR block",
    )
  }

  fn address_of(&self, outcome: ffi::CzOutcome, action: &str) -> Result<IPv6Address, Error> {
    platform::outcome(outcome, action).map(|outcome| outcome.ipv6_address())
  }

  /// `CIDRv6.contains(_:)`.
  pub fn contains(&self, ip: &IPv6Address) -> Result<bool, Error> {
    boolean(
      ffi::cz_cidr_v6_contains(self.address.clone(), self.prefix.length, ip.clone()),
      "check whether an IPv6 CIDR block contains an address",
    )
  }

  /// `CIDRv6.description`.
  pub fn description(&self) -> Result<String, Error> {
    description(
      ffi::cz_cidr_v6_description(self.address.clone(), self.prefix.length),
      "describe an IPv6 CIDR block",
    )
  }
}
