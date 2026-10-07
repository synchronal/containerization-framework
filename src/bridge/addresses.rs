//! `ContainerizationExtras`' address types, which Swift reads.

use crate::containerization_extras::address::IPv6Address as RustIPv6Address;
use crate::containerization_extras::address::IpAddress as RustIpAddress;

#[swift_bridge::bridge]
mod ffi {
  extern "Rust" {
    // Its `UInt128` value crosses as two halves.
    type RustIPv6Address;
    #[swift_bridge(swift_name = "valueHigh")]
    fn value_high(self: &RustIPv6Address) -> u64;
    #[swift_bridge(swift_name = "valueLow")]
    fn value_low(self: &RustIPv6Address) -> u64;
    fn zone(self: &RustIPv6Address) -> Option<&str>;

    type RustIpAddress;
    #[swift_bridge(swift_name = "holdsV6")]
    fn holds_v6(self: &RustIpAddress) -> bool;
    #[swift_bridge(swift_name = "v4Value")]
    fn v4_value(self: &RustIpAddress) -> u32;
    fn v6(self: &RustIpAddress) -> &RustIPv6Address;
  }
}
