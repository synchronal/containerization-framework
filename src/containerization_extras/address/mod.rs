//! Network addresses: IP and MAC addresses, prefixes, and CIDR blocks.
//!
//! What Swift computes, from parsing to an address's `isLoopback`, is asked of
//! Swift, so each of those returns [`Error::Unavailable`] off macOS.

mod cidr;
mod cidr_v4;
mod cidr_v6;
mod ip_address;
mod ipv4_address;
mod ipv6_address;
mod mac_address;
mod prefix;

pub use self::cidr::CIDR;
pub use self::cidr_v4::CIDRv4;
pub use self::cidr_v6::CIDRv6;
pub use self::ip_address::IPAddress;
pub use self::ipv4_address::IPv4Address;
pub use self::ipv6_address::IPv6Address;
pub use self::mac_address::MACAddress;
pub use self::prefix::Prefix;

use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::cmp::Ordering;

/// A Swift `Bool`.
fn boolean(outcome: ffi::CzOutcome, action: &str) -> Result<bool, Error> {
  platform::outcome(outcome, action).map(|outcome| outcome.boolean())
}

/// A Swift `description`.
fn description(outcome: ffi::CzOutcome, action: &str) -> Result<String, Error> {
  platform::outcome(outcome, action).map(|outcome| outcome.text())
}

/// Swift's `<`, asked both ways, as Rust's partial order. `None` when neither
/// is less but the two differ. Off macOS, where Swift can't be asked, it
/// panics.
fn ordering<T: PartialEq>(lhs: &T, rhs: &T, less: impl Fn(&T, &T) -> Result<bool, Error>) -> Option<Ordering> {
  let less = |lhs, rhs| less(lhs, rhs).unwrap_or_else(|error| panic!("{error}"));

  if less(lhs, rhs) {
    Some(Ordering::Less)
  } else if less(rhs, lhs) {
    Some(Ordering::Greater)
  } else if lhs == rhs {
    Some(Ordering::Equal)
  } else {
    None
  }
}
