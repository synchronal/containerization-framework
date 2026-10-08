use super::NATInterface;
use super::vmnet_network;
use crate::containerization_extras;

/// `any Interface`: one of its conforming types on macOS.
///
/// `NATNetworkInterface` is not among them. Its one initializer available on
/// macOS 26 takes a `vmnet_network_ref`, which Rust can't make.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Interface {
  Nat(NATInterface),
  Vmnet(vmnet_network::Interface),
}

impl Interface {
  /// `Interface.ipv4Address`.
  pub fn ipv4_address(&self) -> containerization_extras::address::CIDRv4 {
    match self {
      Self::Nat(interface) => interface.ipv4_address,
      Self::Vmnet(interface) => interface.ipv4_address(),
    }
  }

  /// `Interface.ipv4Gateway`.
  pub fn ipv4_gateway(&self) -> Option<containerization_extras::address::IPv4Address> {
    match self {
      Self::Nat(interface) => interface.ipv4_gateway,
      Self::Vmnet(interface) => interface.ipv4_gateway(),
    }
  }

  /// `Interface.ipv6Address`.
  pub fn ipv6_address(&self) -> Option<containerization_extras::address::CIDRv6> {
    match self {
      Self::Nat(interface) => interface.ipv6_address.clone(),
      Self::Vmnet(interface) => interface.ipv6_address(),
    }
  }

  /// `Interface.ipv6Gateway`.
  pub fn ipv6_gateway(&self) -> Option<containerization_extras::address::IPv6Address> {
    match self {
      Self::Nat(interface) => interface.ipv6_gateway.clone(),
      Self::Vmnet(interface) => interface.ipv6_gateway(),
    }
  }

  /// `Interface.macAddress`.
  pub fn mac_address(&self) -> Option<containerization_extras::address::MACAddress> {
    match self {
      Self::Nat(interface) => interface.mac_address,
      Self::Vmnet(interface) => interface.mac_address(),
    }
  }

  /// `Interface.mtu`.
  pub fn mtu(&self) -> u32 {
    match self {
      Self::Nat(interface) => interface.mtu,
      Self::Vmnet(interface) => interface.mtu(),
    }
  }
}
