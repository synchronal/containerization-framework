//! Stand-ins for `VmnetNetwork.swift`: a `Network?`, `VmnetNetwork` and its
//! interfaces.

use super::CzOutcome;
use super::VmnetMode;
use crate::containerization::network;
use crate::containerization_extras;
use std::convert::Infallible;

/// A `Network?`, which is only ever `nil` here.
pub(crate) struct CzNetwork;

pub(crate) fn cz_no_network() -> CzNetwork {
  CzNetwork
}

taken!(
  vmnet_network -> CzVmnetNetwork,
  optional_vmnet_interface -> Option<network::vmnet_network::Interface>,
);

handles!(CzVmnetNetwork, CzVmnetInterface);

failing!(cz_vmnet_network_new(VmnetMode, Option<u32>, u8, bool, containerization_extras::address::IPv6Address, u8));

uninhabited!(CzVmnetNetwork {
  fn as_network() -> CzNetwork;
  fn subnet() -> CzOutcome;
  fn prefix_v6() -> CzOutcome;
  fn ipv4_gateway() -> CzOutcome;
  fn ipv6_gateway() -> CzOutcome;
  fn create_interface(&str) -> CzOutcome;
  fn create_interface_with_mtu(&str, u32) -> CzOutcome;
  fn create_interface_without_gateway(&str) -> CzOutcome;
  fn release_interface(&str) -> CzOutcome;
});

uninhabited!(CzVmnetInterface {
  fn duplicate() -> CzVmnetInterface;
  fn ipv4_address() -> CzOutcome;
  fn ipv4_gateway() -> CzOutcome;
  fn ipv6_address() -> CzOutcome;
  fn ipv6_gateway() -> CzOutcome;
  fn mac_address() -> CzOutcome;
  fn mtu() -> u32;
});
