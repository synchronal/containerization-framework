use super::address;

/// `LinkRoute`, which [`crate::containerization::vm::Vminitd::route_add_link`]
/// installs. [`Default`] uses Swift's `LinkRoute()`.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct LinkRoute {
  pub ipv4_destination: Option<address::IPv4Address>,
  pub ipv4_source: Option<address::IPv4Address>,
  pub ipv6_destination: Option<address::IPv6Address>,
  pub ipv6_source: Option<address::IPv6Address>,
}
