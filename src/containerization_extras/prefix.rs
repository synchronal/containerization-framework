use super::description;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `Prefix`. Only Swift makes one, so its length is one Swift accepted.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Prefix {
  pub(crate) length: u8,
}

impl Prefix {
  /// `Prefix(length:)`: `None` above 128.
  pub fn new(length: u8) -> Result<Option<Self>, Error> {
    Self::made(ffi::cz_prefix_new(length), length)
  }

  /// `Prefix.ipv4(_:)`: `None` above 32.
  pub fn ipv4(length: u8) -> Result<Option<Self>, Error> {
    Self::made(ffi::cz_prefix_ipv4(length), length)
  }

  /// `Prefix.ipv6(_:)`: `None` above 128.
  pub fn ipv6(length: u8) -> Result<Option<Self>, Error> {
    Self::made(ffi::cz_prefix_ipv6(length), length)
  }

  fn made(outcome: ffi::CzOutcome, length: u8) -> Result<Option<Self>, Error> {
    platform::outcome(outcome, format!("make a prefix of length {length}"))
      .map(|outcome| outcome.is_some().then(|| outcome.prefix()))
  }

  /// `Prefix.length`.
  pub fn length(&self) -> u8 {
    self.length
  }

  /// `Prefix.description`.
  pub fn description(&self) -> Result<String, Error> {
    description(ffi::cz_prefix_description(self.length), "describe a prefix")
  }

  /// `Prefix.suffixMask32`.
  pub fn suffix_mask32(&self) -> Result<u32, Error> {
    self.mask32(ffi::cz_prefix_suffix_mask32(self.length))
  }

  /// `Prefix.prefixMask32`.
  pub fn prefix_mask32(&self) -> Result<u32, Error> {
    self.mask32(ffi::cz_prefix_prefix_mask32(self.length))
  }

  /// `Prefix.suffixMask128`.
  pub fn suffix_mask128(&self) -> Result<u128, Error> {
    self.mask128(ffi::cz_prefix_suffix_mask128(self.length))
  }

  /// `Prefix.prefixMask128`.
  pub fn prefix_mask128(&self) -> Result<u128, Error> {
    self.mask128(ffi::cz_prefix_prefix_mask128(self.length))
  }

  fn mask32(&self, outcome: ffi::CzOutcome) -> Result<u32, Error> {
    platform::outcome(outcome, format!("read the mask of /{}", self.length)).map(|outcome| outcome.number() as u32)
  }

  fn mask128(&self, outcome: ffi::CzOutcome) -> Result<u128, Error> {
    platform::outcome(outcome, format!("read the mask of /{}", self.length)).map(|outcome| outcome.uint128())
  }
}
