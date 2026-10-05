//! Addresses on Virtualization.framework's built-in NAT, which hands out no
//! leases.

use containerization_framework as cfw;

/// `.1` is the gateway, `.255` the broadcast.
pub const GATEWAY: &str = "192.168.64.1";
const PREFIX: u32 = 24;
const FIRST_HOST: u32 = 2;
const LAST_HOST: u32 = 250;

/// Resolve through the NAT's gateway.
pub fn gateway_dns() -> cfw::containerization::Dns {
  cfw::containerization::Dns {
    nameservers: vec![GATEWAY.to_string()],
    ..Default::default()
  }
}

/// Where a container by this name sits on the NAT network.
///
/// Nothing hands out leases, so the caller allocates. Hashed from the name:
/// stable per container, distinct between concurrent ones.
pub fn interface(name: &str) -> cfw::containerization::NatInterface {
  // FNV-1a: short and well spread.
  let mut hash: u64 = 0xcbf2_9ce4_8422_2325;

  for byte in name.as_bytes() {
    hash ^= u64::from(*byte);
    hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
  }

  let host = FIRST_HOST + (hash % u64::from(LAST_HOST - FIRST_HOST + 1)) as u32;
  let (subnet, _) = GATEWAY
    .rsplit_once('.')
    .expect("the gateway is a dotted quad");

  cfw::containerization::NatInterface::new(format!("{subnet}.{host}/{PREFIX}"), GATEWAY)
}
