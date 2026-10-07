use super::address::CIDRv4;
use super::address::CIDRv6;
use super::address::IPv4Address;
use super::address::IPv6Address;

/// `InterfaceAddress`, the addresses
/// [`crate::containerization::vm::Vminitd::address_add`] gives an interface.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InterfaceAddress {
  pub ipv4_address: CIDRv4,
  pub ipv6_address: Option<CIDRv6>,
}

/// `LinkRoute`, which [`crate::containerization::vm::Vminitd::route_add_link`]
/// installs. Its `Default` is Swift's `LinkRoute()`.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct LinkRoute {
  pub ipv4_destination: Option<IPv4Address>,
  pub ipv4_source: Option<IPv4Address>,
  pub ipv6_destination: Option<IPv6Address>,
  pub ipv6_source: Option<IPv6Address>,
}

/// `DefaultRoute`, which
/// [`crate::containerization::vm::Vminitd::route_add_default`] installs. Its
/// `Default` is Swift's `DefaultRoute()`.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct DefaultRoute {
  pub ipv4_gateway: Option<IPv4Address>,
  pub ipv6_gateway: Option<IPv6Address>,
}
