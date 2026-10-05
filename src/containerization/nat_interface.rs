/// `NATInterface`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NatInterface {
  /// As CIDR.
  pub ipv4_address: String,
  pub ipv4_gateway: Option<String>,
  /// As CIDR.
  pub ipv6_address: Option<String>,
  pub ipv6_gateway: Option<String>,
  pub mac_address: Option<String>,
  pub mtu: u32,
}

impl NatInterface {
  /// `NATInterface(ipv4Address:ipv4Gateway:)`, its other arguments at their
  /// defaults.
  pub fn new(ipv4_address: impl Into<String>, ipv4_gateway: impl Into<String>) -> Self {
    Self {
      ipv4_address: ipv4_address.into(),
      ipv4_gateway: Some(ipv4_gateway.into()),
      ipv6_address: None,
      ipv6_gateway: None,
      mac_address: None,
      mtu: 1500,
    }
  }
}
