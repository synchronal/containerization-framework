use super::address;

/// `DefaultRoute`, which
/// [`crate::containerization::vm::Vminitd::route_add_default`] installs.
/// [`Default`] uses Swift's `DefaultRoute()`.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct DefaultRoute {
  pub ipv4_gateway: Option<address::IPv4Address>,
  pub ipv6_gateway: Option<address::IPv6Address>,
}
