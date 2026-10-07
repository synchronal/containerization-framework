//! Getters and setters for network interfaces, DNS and hosts.

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

impl From<vmnet_network::Mode> for ffi::VmnetMode {
  fn from(mode: vmnet_network::Mode) -> Self {
    match mode {
      vmnet_network::Mode::Shared => Self::Shared,
      vmnet_network::Mode::Host => Self::Host,
      vmnet_network::Mode::Bridged => Self::Bridged,
    }
  }
}

impl network::NatInterface {
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

  /// Only when [`Self::has_ipv6_address`].
  pub(crate) fn ipv6_address(&self) -> &address::IPv6Address {
    &self.ipv6_cidr().address
  }

  /// Only when [`Self::has_ipv6_address`].
  pub(crate) fn ipv6_prefix(&self) -> u8 {
    self.ipv6_cidr().prefix.length
  }

  fn ipv6_cidr(&self) -> &address::CIDRv6 {
    self
      .ipv6_address
      .as_ref()
      .expect("Swift asks for an IPv6 address only after has_ipv6_address")
  }

  pub(crate) fn has_ipv6_gateway(&self) -> bool {
    self.ipv6_gateway.is_some()
  }

  /// Only when [`Self::has_ipv6_gateway`].
  pub(crate) fn ipv6_gateway(&self) -> &address::IPv6Address {
    self
      .ipv6_gateway
      .as_ref()
      .expect("Swift asks for an IPv6 gateway only after has_ipv6_gateway")
  }

  pub(crate) fn mac_address(&self) -> Option<u64> {
    self.mac_address.map(|address| address.value)
  }

  pub(crate) fn mtu(&self) -> u32 {
    self.mtu
  }
}

impl network::Dns {
  pub(crate) fn nameservers_len(&self) -> usize {
    self.nameservers.len()
  }

  pub(crate) fn nameservers_at(&self, index: usize) -> &str {
    &self.nameservers[index]
  }

  pub(crate) fn domain(&self) -> Option<&str> {
    self.domain.as_deref()
  }

  pub(crate) fn search_domains_len(&self) -> usize {
    self.search_domains.len()
  }

  pub(crate) fn search_domains_at(&self, index: usize) -> &str {
    &self.search_domains[index]
  }

  pub(crate) fn options_len(&self) -> usize {
    self.options.len()
  }

  pub(crate) fn options_at(&self, index: usize) -> &str {
    &self.options[index]
  }
}

impl hosts::Entry {
  pub(crate) fn ip_address(&self) -> &str {
    &self.ip_address
  }

  pub(crate) fn hostnames_len(&self) -> usize {
    self.hostnames.len()
  }

  pub(crate) fn hostnames_at(&self, index: usize) -> &str {
    &self.hostnames[index]
  }

  pub(crate) fn comment(&self) -> Option<&str> {
    self.comment.as_deref()
  }
}

impl network::Hosts {
  pub(crate) fn entries_len(&self) -> usize {
    self.entries.len()
  }

  pub(crate) fn entries_at(&self, index: usize) -> &hosts::Entry {
    &self.entries[index]
  }

  pub(crate) fn comment(&self) -> Option<&str> {
    self.comment.as_deref()
  }
}

// What the container, pod and VM configurations share, read and filled the
// same way.

pub(in super::super) fn interface_kind(interface: &network::Interface) -> ffi::InterfaceKind {
  match interface {
    network::Interface::Nat(_) => ffi::InterfaceKind::Nat,
    network::Interface::Vmnet(_) => ffi::InterfaceKind::Vmnet,
  }
}

pub(in super::super) fn nat_interface(interface: &network::Interface) -> &network::NatInterface {
  match interface {
    network::Interface::Nat(interface) => interface,
    network::Interface::Vmnet(_) => unreachable!("Swift asks for a NAT interface only of its kind"),
  }
}

/// A handle on a vmnet interface for Swift to keep.
pub(in super::super) fn vmnet_interface(interface: &network::Interface) -> ffi::CzVmnetInterface {
  match interface {
    network::Interface::Vmnet(interface) => interface.handle.duplicate(),
    network::Interface::Nat(_) => unreachable!("Swift asks for a vmnet interface only of its kind"),
  }
}

/// A `NATInterface` of its IPv4 fields: Swift sets its IPv6 ones after.
pub(in super::super) fn nat(
  ipv4_address: u32,
  ipv4_prefix: u8,
  ipv4_gateway: Option<u32>,
  mac_address: Option<u64>,
  mtu: u32,
) -> network::Interface {
  network::Interface::Nat(network::NatInterface {
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

pub(in super::super) fn vmnet(interface: ffi::CzVmnetInterface) -> network::Interface {
  network::Interface::Vmnet(vmnet_network::Interface { handle: interface })
}

pub(in super::super) fn set_ipv6_address(
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

pub(in super::super) fn set_ipv6_gateway(
  interfaces: &mut [network::Interface],
  high: u64,
  low: u64,
  zone: Option<String>,
) {
  last_nat(interfaces).ipv6_gateway = Some(address::IPv6Address::from_halves(high, low, zone));
}

fn last_nat(interfaces: &mut [network::Interface]) -> &mut network::NatInterface {
  match interfaces.last_mut() {
    Some(network::Interface::Nat(interface)) => interface,
    _ => unreachable!("Swift sets an interface's IPv6 addresses only after push_interface"),
  }
}

pub(in super::super) fn dns(
  nameservers: Vec<String>,
  domain: Option<String>,
  search_domains: Vec<String>,
  options: Vec<String>,
) -> network::Dns {
  network::Dns {
    nameservers,
    domain,
    search_domains,
    options,
  }
}

pub(in super::super) fn hosts(comment: Option<String>) -> network::Hosts {
  network::Hosts {
    entries: Vec::new(),
    comment,
  }
}

/// Only after `set_hosts`.
pub(in super::super) fn push_hosts_entry(
  hosts: &mut Option<network::Hosts>,
  ip_address: String,
  hostnames: Vec<String>,
  comment: Option<String>,
) {
  hosts
    .as_mut()
    .expect("Swift adds a hosts entry only after set_hosts")
    .entries
    .push(hosts::Entry {
      ip_address,
      hostnames,
      comment,
    });
}
