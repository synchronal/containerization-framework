//! The network values a container or VM is configured with, which Swift
//! reads.

use crate::containerization::network::Dns as RustDns;
use crate::containerization::network::Hosts as RustHosts;
use crate::containerization::network::NatInterface as RustNatInterface;
use crate::containerization::network::hosts::Entry as RustHostsEntry;
use crate::containerization_extras::address::IPv6Address as RustIPv6Address;

#[swift_bridge::bridge]
mod ffi {
  extern "Rust" {
    #[swift_bridge(already_declared)]
    type RustIPv6Address;

    type RustNatInterface;
    #[swift_bridge(swift_name = "ipv4AddressValue")]
    fn ipv4_address_value(self: &RustNatInterface) -> u32;
    #[swift_bridge(swift_name = "ipv4Prefix")]
    fn ipv4_prefix(self: &RustNatInterface) -> u8;
    #[swift_bridge(swift_name = "ipv4Gateway")]
    fn ipv4_gateway(self: &RustNatInterface) -> Option<u32>;
    #[swift_bridge(swift_name = "hasIpv6Address")]
    fn has_ipv6_address(self: &RustNatInterface) -> bool;
    #[swift_bridge(swift_name = "ipv6Address")]
    fn ipv6_address(self: &RustNatInterface) -> &RustIPv6Address;
    #[swift_bridge(swift_name = "ipv6Prefix")]
    fn ipv6_prefix(self: &RustNatInterface) -> u8;
    #[swift_bridge(swift_name = "hasIpv6Gateway")]
    fn has_ipv6_gateway(self: &RustNatInterface) -> bool;
    #[swift_bridge(swift_name = "ipv6Gateway")]
    fn ipv6_gateway(self: &RustNatInterface) -> &RustIPv6Address;
    #[swift_bridge(swift_name = "macAddress")]
    fn mac_address(self: &RustNatInterface) -> Option<u64>;
    fn mtu(self: &RustNatInterface) -> u32;

    type RustDns;
    #[swift_bridge(swift_name = "nameserversLen")]
    fn nameservers_len(self: &RustDns) -> usize;
    #[swift_bridge(swift_name = "nameserversAt")]
    fn nameservers_at(self: &RustDns, index: usize) -> &str;
    fn domain(self: &RustDns) -> Option<&str>;
    #[swift_bridge(swift_name = "searchDomainsLen")]
    fn search_domains_len(self: &RustDns) -> usize;
    #[swift_bridge(swift_name = "searchDomainsAt")]
    fn search_domains_at(self: &RustDns, index: usize) -> &str;
    #[swift_bridge(swift_name = "optionsLen")]
    fn options_len(self: &RustDns) -> usize;
    #[swift_bridge(swift_name = "optionsAt")]
    fn options_at(self: &RustDns, index: usize) -> &str;

    type RustHostsEntry;
    #[swift_bridge(swift_name = "ipAddress")]
    fn ip_address(self: &RustHostsEntry) -> &str;
    #[swift_bridge(swift_name = "hostnamesLen")]
    fn hostnames_len(self: &RustHostsEntry) -> usize;
    #[swift_bridge(swift_name = "hostnamesAt")]
    fn hostnames_at(self: &RustHostsEntry, index: usize) -> &str;
    fn comment(self: &RustHostsEntry) -> Option<&str>;

    type RustHosts;
    #[swift_bridge(swift_name = "entriesLen")]
    fn entries_len(self: &RustHosts) -> usize;
    #[swift_bridge(swift_name = "entriesAt")]
    fn entries_at(self: &RustHosts, index: usize) -> &RustHostsEntry;
    fn comment(self: &RustHosts) -> Option<&str>;
  }
}
