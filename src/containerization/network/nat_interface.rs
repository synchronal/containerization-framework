use crate::containerization_extras;

/// `NATInterface`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NatInterface {
  pub ipv4_address: containerization_extras::address::CIDRv4,
  pub ipv4_gateway: Option<containerization_extras::address::IPv4Address>,
  pub ipv6_address: Option<containerization_extras::address::CIDRv6>,
  pub ipv6_gateway: Option<containerization_extras::address::IPv6Address>,
  pub mac_address: Option<containerization_extras::address::MACAddress>,
  pub mtu: u32,
}

impl NatInterface {
  /// `NATInterface(ipv4Address:ipv4Gateway:)`, its other arguments at their
  /// defaults.
  pub fn new(
    ipv4_address: containerization_extras::address::CIDRv4,
    ipv4_gateway: Option<containerization_extras::address::IPv4Address>,
  ) -> Self {
    Self {
      ipv4_address,
      ipv4_gateway,
      ipv6_address: None,
      ipv6_gateway: None,
      mac_address: None,
      mtu: 1500,
    }
  }
}
