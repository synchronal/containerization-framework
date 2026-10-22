//! Getters and setters for network interfaces, DNS and hosts.

use crate::bridge::accessors;
use crate::bridge::ffi;
use crate::containerization::network;
use crate::containerization::network::hosts;
use crate::containerization::network::vmnet_network;
use crate::containerization_extras::address;

impl ffi::CzOutcome {
  /// The `VmnetNetwork.Interface?` an outcome holds.
  pub(crate) fn optional_vmnet_interface(&self) -> Option<vmnet_network::Interface> {
    self.optional(|interface| vmnet_network::Interface {
      handle: interface.vmnet_interface(),
    })
  }

  /// The `Hosts.Entry` an outcome holds, read field by field.
  pub(crate) fn hosts_entry(&self) -> hosts::Entry {
    hosts::Entry {
      ip_address: self.hosts_entry_ip_address(),
      hostnames: self.hosts_entry_hostnames(),
      comment: self.hosts_entry_comment(),
    }
  }
}

impl network::NATInterface {
  pub(crate) fn ipv4_address_value(&self) -> u32 {
    self.ipv4_address.address.value
  }

  pub(crate) fn ipv4_prefix(&self) -> u8 {
    self.ipv4_address.prefix.length
  }

  pub(crate) fn ipv4_gateway(&self) -> Option<u32> {
    self.ipv4_gateway.map(|gateway| gateway.value)
  }

  pub(crate) fn has_ipv6_address(&self) -> bool {
    self.ipv6_address.is_some()
  }

  /// Swift calls this only when [`Self::has_ipv6_address`] is true.
  pub(crate) fn ipv6_address(&self) -> &address::IPv6Address {
    &self.ipv6_cidr().address
  }

  /// Swift calls this only when [`Self::has_ipv6_address`] is true.
  pub(crate) fn ipv6_prefix(&self) -> u8 {
    self.ipv6_cidr().prefix.length
  }

  fn ipv6_cidr(&self) -> &address::CIDRv6 {
    accessors::present(&self.ipv6_address)
  }

  getters! {
    present ipv6_gateway: address::IPv6Address => has_ipv6_gateway;
  }

  pub(crate) fn mac_address(&self) -> Option<u64> {
    self.mac_address.map(|address| address.value)
  }

  getters! {
    copy mtu: u32;
  }
}

impl network::DNS {
  getters! {
    list nameservers: str => nameservers_len, nameservers_at;
    optional_str domain;
    list search_domains: str => search_domains_len, search_domains_at;
    list options: str => options_len, options_at;
  }
}

impl hosts::Entry {
  getters! {
    str ip_address;
    list hostnames: str => hostnames_len, hostnames_at;
    optional_str comment;
  }
}

impl network::Hosts {
  getters! {
    list entries: hosts::Entry => entries_len, entries_at;
    optional_str comment;
  }
}

// What the container, pod and VM configurations share, read and filled the
// same way. The macros in `container/mod.rs` call these.

pub(super) fn interface_kind(interface: &network::Interface) -> ffi::InterfaceKind {
  match interface {
    network::Interface::Nat(_) => ffi::InterfaceKind::Nat,
    network::Interface::Vmnet(_) => ffi::InterfaceKind::Vmnet,
  }
}

pub(super) fn nat_interface(interface: &network::Interface) -> &network::NATInterface {
  match interface {
    network::Interface::Nat(interface) => interface,
    network::Interface::Vmnet(_) => unreachable!("Swift asks for a NAT interface only of its kind"),
  }
}

/// A handle on a vmnet interface for Swift to keep.
pub(super) fn vmnet_interface(interface: &network::Interface) -> ffi::CzVmnetInterface {
  match interface {
    network::Interface::Vmnet(interface) => interface.handle.duplicate(),
    network::Interface::Nat(_) => unreachable!("Swift asks for a vmnet interface only of its kind"),
  }
}

/// A `NATInterface` built from its IPv4 fields. Swift sets its IPv6 ones
/// afterwards.
pub(super) fn nat(
  ipv4_address: u32,
  ipv4_prefix: u8,
  ipv4_gateway: Option<u32>,
  mac_address: Option<u64>,
  mtu: u32,
) -> network::Interface {
  network::Interface::Nat(network::NATInterface {
    ipv4_address: address::CIDRv4 {
      address: address::IPv4Address::new(ipv4_address),
      prefix: address::Prefix { length: ipv4_prefix },
    },
    ipv4_gateway: ipv4_gateway.map(address::IPv4Address::new),
    ipv6_address: None,
    ipv6_gateway: None,
    mac_address: mac_address.map(|value| address::MACAddress { value }),
    mtu,
  })
}

pub(super) fn vmnet(interface: ffi::CzVmnetInterface) -> network::Interface {
  network::Interface::Vmnet(vmnet_network::Interface { handle: interface })
}

pub(super) fn set_ipv6_address(
  interfaces: &mut [network::Interface],
  high: u64,
  low: u64,
  zone: Option<String>,
  prefix: u8,
) {
  last_nat(interfaces).ipv6_address = Some(address::CIDRv6 {
    address: address::IPv6Address::from_halves(high, low, zone),
    prefix: address::Prefix { length: prefix },
  });
}

pub(super) fn set_ipv6_gateway(interfaces: &mut [network::Interface], high: u64, low: u64, zone: Option<String>) {
  last_nat(interfaces).ipv6_gateway = Some(address::IPv6Address::from_halves(high, low, zone));
}

fn last_nat(interfaces: &mut [network::Interface]) -> &mut network::NATInterface {
  match interfaces.last_mut() {
    Some(network::Interface::Nat(interface)) => interface,
    _ => unreachable!("Swift sets an interface's IPv6 addresses only after push_interface"),
  }
}

pub(super) fn dns(
  nameservers: Vec<String>,
  domain: Option<String>,
  search_domains: Vec<String>,
  options: Vec<String>,
) -> network::DNS {
  network::DNS {
    nameservers,
    domain,
    search_domains,
    options,
  }
}

pub(super) fn hosts(comment: Option<String>) -> network::Hosts {
  network::Hosts {
    entries: Vec::new(),
    comment,
  }
}

/// Swift calls this only after `set_hosts`.
pub(super) fn push_hosts_entry(
  hosts: &mut Option<network::Hosts>,
  ip_address: String,
  hostnames: Vec<String>,
  comment: Option<String>,
) {
  hosts
    .as_mut()
    .unwrap_or_else(|| unreachable!("Swift adds a hosts entry only after set_hosts"))
    .entries
    .push(hosts::Entry {
      ip_address,
      hostnames,
      comment,
    });
}
