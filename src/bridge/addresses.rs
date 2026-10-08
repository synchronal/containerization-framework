//! `ContainerizationExtras`' address types, which Swift reads, and the
//! interface addresses and routes made of them.

use crate::containerization_extras::DefaultRoute as RustDefaultRoute;
use crate::containerization_extras::InterfaceAddress as RustInterfaceAddress;
use crate::containerization_extras::LinkRoute as RustLinkRoute;
use crate::containerization_extras::address::IPAddress as RustIpAddress;
use crate::containerization_extras::address::IPv6Address as RustIPv6Address;

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

    // Each CIDR block crosses as its address and its prefix's length.
    type RustInterfaceAddress;
    #[swift_bridge(swift_name = "ipv4AddressValue")]
    fn ipv4_address_value(self: &RustInterfaceAddress) -> u32;
    #[swift_bridge(swift_name = "ipv4AddressPrefix")]
    fn ipv4_address_prefix(self: &RustInterfaceAddress) -> u8;
    #[swift_bridge(swift_name = "hasIpv6Address")]
    fn has_ipv6_address(self: &RustInterfaceAddress) -> bool;
    #[swift_bridge(swift_name = "ipv6AddressAddress")]
    fn ipv6_address_address(self: &RustInterfaceAddress) -> &RustIPv6Address;
    #[swift_bridge(swift_name = "ipv6AddressPrefix")]
    fn ipv6_address_prefix(self: &RustInterfaceAddress) -> u8;

    type RustLinkRoute;
    #[swift_bridge(swift_name = "ipv4Destination")]
    fn ipv4_destination(self: &RustLinkRoute) -> Option<u32>;
    #[swift_bridge(swift_name = "ipv4Source")]
    fn ipv4_source(self: &RustLinkRoute) -> Option<u32>;
    #[swift_bridge(swift_name = "hasIpv6Destination")]
    fn has_ipv6_destination(self: &RustLinkRoute) -> bool;
    #[swift_bridge(swift_name = "ipv6Destination")]
    fn ipv6_destination(self: &RustLinkRoute) -> &RustIPv6Address;
    #[swift_bridge(swift_name = "hasIpv6Source")]
    fn has_ipv6_source(self: &RustLinkRoute) -> bool;
    #[swift_bridge(swift_name = "ipv6Source")]
    fn ipv6_source(self: &RustLinkRoute) -> &RustIPv6Address;

    type RustDefaultRoute;
    #[swift_bridge(swift_name = "ipv4Gateway")]
    fn ipv4_gateway(self: &RustDefaultRoute) -> Option<u32>;
    #[swift_bridge(swift_name = "hasIpv6Gateway")]
    fn has_ipv6_gateway(self: &RustDefaultRoute) -> bool;
    #[swift_bridge(swift_name = "ipv6Gateway")]
    fn ipv6_gateway(self: &RustDefaultRoute) -> &RustIPv6Address;
  }
}
