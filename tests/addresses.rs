#![cfg(feature = "integration")]

//! The `ContainerizationExtras` address types, each asked of Swift.

use cfw::containerization_extras as extras;
use containerization_framework as cfw;
use std::cmp::Ordering;

fn ipv4(string: &str) -> extras::address::IPv4Address {
  extras::address::IPv4Address::parse(string).expect("a valid IPv4 address")
}

fn ipv6(string: &str) -> extras::address::IPv6Address {
  extras::address::IPv6Address::parse(string).expect("a valid IPv6 address")
}

fn prefix(length: u8) -> extras::address::Prefix {
  extras::address::Prefix::new(length)
    .expect("a prefix")
    .expect("a length of at most 128")
}

#[test]
fn parses_and_describes_an_ipv4_address() {
  let address = ipv4("10.0.0.2");

  assert_eq!(address, extras::address::IPv4Address::new(0x0a00_0002));
  assert_eq!(address.description().expect("a description"), "10.0.0.2");
  assert_eq!(address.bytes().expect("bytes"), vec![10, 0, 0, 2]);
  assert_eq!(
    extras::address::IPv4Address::from_bytes(&[10, 0, 0, 2]).expect("four bytes"),
    address
  );
  assert!(extras::address::IPv4Address::from_bytes(&[10, 0, 0]).is_err());
  assert!(extras::address::IPv4Address::parse("10.0.0").is_err());
}

#[test]
fn classifies_ipv4_addresses() {
  assert!(ipv4("0.0.0.0").is_unspecified().expect("an answer"));
  assert!(ipv4("127.0.0.1").is_loopback().expect("an answer"));
  assert!(ipv4("224.0.0.1").is_multicast().expect("an answer"));
  assert!(ipv4("169.254.1.1").is_link_local().expect("an answer"));
  assert!(ipv4("255.255.255.255").is_broadcast().expect("an answer"));
  assert!(!ipv4("10.0.0.2").is_loopback().expect("an answer"));
}

#[test]
fn orders_ipv4_addresses_as_swift_does() {
  assert_eq!(ipv4("10.0.0.2").partial_cmp(&ipv4("10.0.0.10")), Some(Ordering::Less));
  assert_eq!(
    ipv4("10.0.0.10").partial_cmp(&ipv4("10.0.0.2")),
    Some(Ordering::Greater)
  );
  assert_eq!(ipv4("10.0.0.2").partial_cmp(&ipv4("10.0.0.2")), Some(Ordering::Equal));
}

#[test]
fn parses_and_describes_an_ipv6_address() {
  let address = ipv6("fe80::1%en0");

  assert_eq!(address.value, 0xfe80 << 112 | 1);
  assert_eq!(address.zone.as_deref(), Some("en0"));
  assert_eq!(address.description().expect("a description"), "fe80::1%en0");
  assert!(address.is_link_local().expect("an answer"));

  let bytes = address.bytes().expect("bytes");
  assert_eq!(bytes.len(), 16);
  assert_eq!(
    extras::address::IPv6Address::from_bytes(&bytes, Some("en0")).expect("sixteen bytes"),
    address
  );
  assert!(extras::address::IPv6Address::from_bytes(&bytes[..15], None).is_err());
  assert!(extras::address::IPv6Address::parse("fe80:::1").is_err());
}

#[test]
fn reads_the_ipv6_constants_from_swift() {
  assert_eq!(
    extras::address::IPv6Address::unspecified().expect("the unspecified address"),
    extras::address::IPv6Address::new(0, None)
  );
  assert_eq!(
    extras::address::IPv6Address::loopback().expect("the loopback address"),
    extras::address::IPv6Address::new(1, None)
  );
}

#[test]
fn classifies_ipv6_addresses() {
  assert!(ipv6("::").is_unspecified().expect("an answer"));
  assert!(ipv6("::1").is_loopback().expect("an answer"));
  assert!(ipv6("ff02::1").is_multicast().expect("an answer"));
  assert!(ipv6("fd00::1").is_unique_local().expect("an answer"));
  assert!(ipv6("2606:4700::1").is_global_unicast().expect("an answer"));
  assert!(ipv6("2001:db8::1").is_documentation().expect("an answer"));
  assert!(!ipv6("fd00::1").is_global_unicast().expect("an answer"));
}

#[test]
fn orders_ipv6_addresses_as_swift_does() {
  assert_eq!(ipv6("fd00::1").partial_cmp(&ipv6("fd00::2")), Some(Ordering::Less));
  assert_eq!(
    ipv6("fe80::1%en1").partial_cmp(&ipv6("fe80::1%en0")),
    Some(Ordering::Greater)
  );

  // Swift orders no zone as an empty one, but the two aren't equal.
  let none = extras::address::IPv6Address::new(1, None);
  let empty = extras::address::IPv6Address::new(1, Some(String::new()));
  assert_eq!(none.partial_cmp(&empty), None);
}

#[test]
fn reads_an_ip_address_of_either_family() {
  let v4 = extras::address::IpAddress::parse("10.0.0.2").expect("an IPv4 address");
  let v6 = extras::address::IpAddress::parse("::1").expect("an IPv6 address");

  assert_eq!(v4, extras::address::IpAddress::V4(ipv4("10.0.0.2")));
  assert_eq!(v6, extras::address::IpAddress::V6(ipv6("::1")));
  assert_eq!(v4.description().expect("a description"), "10.0.0.2");
  assert!(v4.is_v4().expect("an answer"));
  assert!(!v4.is_v6().expect("an answer"));
  assert!(v6.is_v6().expect("an answer"));
  assert_eq!(v4.ipv4().expect("an answer"), Some(ipv4("10.0.0.2")));
  assert_eq!(v4.ipv6().expect("an answer"), None);
  assert_eq!(v6.ipv6().expect("an answer"), Some(ipv6("::1")));
  assert!(v6.is_loopback().expect("an answer"));
  assert!(!v4.is_multicast().expect("an answer"));
  assert!(!v4.is_unspecified().expect("an answer"));
  assert!(extras::address::IpAddress::parse("nowhere").is_err());
}

#[test]
fn makes_prefixes_only_of_lengths_swift_accepts() {
  assert_eq!(prefix(24).length(), 24);
  assert_eq!(extras::address::Prefix::new(129).expect("an answer"), None);
  assert_eq!(extras::address::Prefix::ipv4(33).expect("an answer"), None);
  assert_eq!(extras::address::Prefix::ipv4(32).expect("an answer"), Some(prefix(32)));
  assert_eq!(
    extras::address::Prefix::ipv6(128).expect("an answer"),
    Some(prefix(128))
  );
  assert_eq!(prefix(24).description().expect("a description"), "24");
}

#[test]
fn reads_a_prefix_s_masks() {
  assert_eq!(prefix(24).prefix_mask32().expect("a mask"), 0xffff_ff00);
  assert_eq!(prefix(24).suffix_mask32().expect("a mask"), 0x0000_00ff);
  assert_eq!(prefix(64).prefix_mask128().expect("a mask"), u128::MAX << 64);
  assert_eq!(prefix(64).suffix_mask128().expect("a mask"), u128::from(u64::MAX));
}

#[test]
fn reads_an_ipv4_block() {
  let block = extras::address::CIDRv4::parse("10.0.0.2/24").expect("a valid block");

  assert_eq!(block.address(), ipv4("10.0.0.2"));
  assert_eq!(block.prefix(), prefix(24));
  assert_eq!(block.gateway().expect("a gateway"), ipv4("10.0.0.1"));
  assert_eq!(block.lower().expect("the lowest address"), ipv4("10.0.0.0"));
  assert_eq!(block.upper().expect("the highest address"), ipv4("10.0.0.255"));
  assert!(block.contains(ipv4("10.0.0.200")).expect("an answer"));
  assert!(!block.contains(ipv4("10.0.1.1")).expect("an answer"));
  assert_eq!(block.description().expect("a description"), "10.0.0.2/24");
}

#[test]
fn makes_ipv4_blocks_as_swift_does() {
  assert_eq!(
    extras::address::CIDRv4::new(ipv4("10.0.0.2"), prefix(24)).expect("a valid block"),
    extras::address::CIDRv4::parse("10.0.0.2/24").expect("a valid block")
  );
  assert!(extras::address::CIDRv4::new(ipv4("10.0.0.2"), prefix(33)).is_err());
  assert_eq!(
    extras::address::CIDRv4::from_range(ipv4("10.0.0.0"), ipv4("10.0.0.255"))
      .expect("a range")
      .description()
      .expect("a description"),
    "10.0.0.0/24"
  );
  assert!(extras::address::CIDRv4::from_range(ipv4("10.0.0.255"), ipv4("10.0.0.0")).is_err());
  assert!(extras::address::CIDRv4::parse("10.0.0.2").is_err());
}

#[test]
fn reads_an_ipv6_block() {
  let block = extras::address::CIDRv6::parse("fd00::5/64").expect("a valid block");

  assert_eq!(block.address(), &ipv6("fd00::5"));
  assert_eq!(block.prefix(), prefix(64));
  assert_eq!(block.gateway().expect("a gateway"), ipv6("fd00::1"));
  assert_eq!(block.lower().expect("the lowest address"), ipv6("fd00::"));
  assert_eq!(
    block.upper().expect("the highest address"),
    ipv6("fd00::ffff:ffff:ffff:ffff")
  );
  assert!(block.contains(&ipv6("fd00::abcd")).expect("an answer"));
  assert!(!block.contains(&ipv6("fd01::1")).expect("an answer"));
  assert_eq!(block.description().expect("a description"), "fd00::5/64");
  assert_eq!(
    extras::address::CIDRv6::new(&ipv6("fd00::5"), prefix(64)).expect("a valid block"),
    block
  );
  assert_eq!(
    extras::address::CIDRv6::from_range(&ipv6("fd00::"), &ipv6("fd00::ff"))
      .expect("a range")
      .description()
      .expect("a description"),
    "fd00::/120"
  );
}

#[test]
fn reads_a_block_of_either_family() {
  let v4 = extras::address::Cidr::parse("10.0.0.2/24").expect("a valid block");
  let v6 = extras::address::Cidr::parse("fd00::5/64").expect("a valid block");

  assert_eq!(v4, extras::address::Cidr::V4(ipv4("10.0.0.2"), prefix(24)));
  assert_eq!(v6, extras::address::Cidr::V6(ipv6("fd00::5"), prefix(64)));
  assert_eq!(
    v4.address().expect("an address"),
    extras::address::IpAddress::V4(ipv4("10.0.0.2"))
  );
  assert_eq!(v6.prefix().expect("a prefix"), prefix(64));
  assert_eq!(
    v4.lower().expect("the lowest address"),
    extras::address::IpAddress::V4(ipv4("10.0.0.0"))
  );
  assert_eq!(
    v4.upper().expect("the highest address"),
    extras::address::IpAddress::V4(ipv4("10.0.0.255"))
  );
  assert!(
    v6.contains(&extras::address::IpAddress::V6(ipv6("fd00::abcd")))
      .expect("an answer")
  );
  assert!(
    !v4
      .contains(&extras::address::IpAddress::V6(ipv6("fd00::abcd")))
      .expect("an answer")
  );
  assert_eq!(v6.description().expect("a description"), "fd00::5/64");
}

#[test]
fn makes_blocks_of_either_family_as_swift_does() {
  let v4 = extras::address::IpAddress::V4(ipv4("10.0.0.2"));
  let v6 = extras::address::IpAddress::V6(ipv6("fd00::5"));

  assert_eq!(
    extras::address::Cidr::new(&v4, prefix(24)).expect("a valid block"),
    extras::address::Cidr::V4(ipv4("10.0.0.2"), prefix(24))
  );
  assert!(extras::address::Cidr::new(&v4, prefix(64)).is_err());
  assert!(extras::address::Cidr::from_range(&v4, &v6).is_err());
  assert_eq!(
    extras::address::Cidr::from_range(
      &extras::address::IpAddress::V4(ipv4("10.0.0.0")),
      &extras::address::IpAddress::V4(ipv4("10.0.0.255"))
    )
    .expect("a range"),
    extras::address::Cidr::V4(ipv4("10.0.0.0"), prefix(24))
  );
  assert!(extras::address::Cidr::parse("nowhere/24").is_err());
}

#[test]
fn reads_a_mac_address() {
  let address = extras::address::MACAddress::parse("02:42:AC:11:00:02").expect("a valid MAC address");

  assert_eq!(address.value(), 0x0242_ac11_0002);
  assert_eq!(address.description().expect("a description"), "02:42:ac:11:00:02");
  assert_eq!(
    address.bytes().expect("bytes"),
    vec![0x02, 0x42, 0xac, 0x11, 0x00, 0x02]
  );
  assert_eq!(
    extras::address::MACAddress::from_bytes(&[0x02, 0x42, 0xac, 0x11, 0x00, 0x02]).expect("six bytes"),
    address
  );
  assert_eq!(
    extras::address::MACAddress::new(0xffff_0242_ac11_0002).expect("a MAC address"),
    address,
    "Swift drops the top 16 bits"
  );
  assert!(address.is_locally_administered().expect("an answer"));
  assert!(!address.is_multicast().expect("an answer"));
  assert_eq!(
    address
      .ipv6_address(&ipv6("fe80::"))
      .expect("a link-local address"),
    ipv6("fe80::42:acff:fe11:2")
  );
  assert_eq!(
    address.partial_cmp(&extras::address::MACAddress::parse("02:42:ac:11:00:03").expect("a valid MAC address")),
    Some(Ordering::Less)
  );
  assert!(extras::address::MACAddress::parse("02:42:ac:11:00").is_err());
}
