//! `VmnetNetwork`, the operating [`Mode`] it takes, and the [`Interface`]s it
//! makes.

use crate::containerization_extras;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::fmt;

/// vmnet's `operating_modes_t`, as `VmnetNetwork(mode:)` takes it.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Mode {
  /// `VMNET_SHARED_MODE`, Swift's default.
  #[default]
  Shared,
  /// `VMNET_HOST_MODE`.
  Host,
  /// `VMNET_BRIDGED_MODE`.
  Bridged,
}

/// `VmnetNetwork.Interface`, attached to the network that made it.
pub struct Interface {
  pub(crate) handle: ffi::CzVmnetInterface,
}

// Swift's `VmnetNetwork.Interface` is `Sendable`.
unsafe impl Send for Interface {}
unsafe impl Sync for Interface {}

impl Interface {
  /// `VmnetNetwork.Interface.ipv4Address`.
  pub fn ipv4_address(&self) -> containerization_extras::address::CIDRv4 {
    self.handle.ipv4_address().cidr_v4()
  }

  /// `VmnetNetwork.Interface.ipv4Gateway`.
  pub fn ipv4_gateway(&self) -> Option<containerization_extras::address::IPv4Address> {
    self.handle.ipv4_gateway().optional_ipv4_address()
  }

  /// `VmnetNetwork.Interface.ipv6Address`.
  pub fn ipv6_address(&self) -> Option<containerization_extras::address::CIDRv6> {
    self.handle.ipv6_address().optional_cidr_v6()
  }

  /// `VmnetNetwork.Interface.ipv6Gateway`.
  pub fn ipv6_gateway(&self) -> Option<containerization_extras::address::IPv6Address> {
    self.handle.ipv6_gateway().optional_ipv6_address()
  }

  /// `VmnetNetwork.Interface.macAddress`.
  pub fn mac_address(&self) -> Option<containerization_extras::address::MACAddress> {
    self.handle.mac_address().optional_mac_address()
  }

  /// `VmnetNetwork.Interface.mtu`.
  pub fn mtu(&self) -> u32 {
    self.handle.mtu()
  }
}

/// Swift's `VmnetNetwork.Interface` is a struct, so a copy attaches to the
/// same network.
impl Clone for Interface {
  fn clone(&self) -> Self {
    Self {
      handle: self.handle.duplicate(),
    }
  }
}

/// Without the network, which Swift keeps private.
impl fmt::Debug for Interface {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("Interface")
      .field("ipv4_address", &self.ipv4_address())
      .field("ipv4_gateway", &self.ipv4_gateway())
      .field("ipv6_address", &self.ipv6_address())
      .field("ipv6_gateway", &self.ipv6_gateway())
      .field("mac_address", &self.mac_address())
      .field("mtu", &self.mtu())
      .finish_non_exhaustive()
  }
}

/// Equal when the six fields Swift shows are. Swift's isn't `Equatable`, and
/// the network it attaches to is private.
impl PartialEq for Interface {
  fn eq(&self, other: &Self) -> bool {
    self.ipv4_address() == other.ipv4_address()
      && self.ipv4_gateway() == other.ipv4_gateway()
      && self.ipv6_address() == other.ipv6_address()
      && self.ipv6_gateway() == other.ipv6_gateway()
      && self.mac_address() == other.mac_address()
      && self.mtu() == other.mtu()
  }
}

impl Eq for Interface {}

/// `VmnetNetwork`, which hands out addresses from its subnet.
pub struct VmnetNetwork {
  pub(crate) handle: ffi::CzVmnetNetwork,
}

// Swift's `VmnetNetwork` is `Sendable`; the interface methods take
// `&mut self` as Swift's are `mutating`.
unsafe impl Send for VmnetNetwork {}
unsafe impl Sync for VmnetNetwork {}

/// Without its allocations, which Swift keeps private.
impl fmt::Debug for VmnetNetwork {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("VmnetNetwork")
      .field("subnet", &self.subnet())
      .field("prefix_v6", &self.prefix_v6())
      .finish_non_exhaustive()
  }
}

impl VmnetNetwork {
  /// `VmnetNetwork(mode:subnet:prefixV6:)`. A `None` subnet or prefix is
  /// left to vmnet to choose.
  pub fn new(
    mode: Mode,
    subnet: Option<containerization_extras::address::CIDRv4>,
    prefix_v6: Option<containerization_extras::address::CIDRv6>,
  ) -> Result<Self, Error> {
    let (has_prefix_v6, prefix_v6) = match prefix_v6 {
      Some(prefix_v6) => (true, prefix_v6),
      None => (
        false,
        containerization_extras::address::CIDRv6 {
          address: containerization_extras::address::IPv6Address::new(0, None),
          prefix: containerization_extras::address::Prefix { length: 0 },
        },
      ),
    };

    let outcome = platform::outcome(
      ffi::cz_vmnet_network_new(
        mode.into(),
        subnet.map(|subnet| subnet.address.value),
        subnet.map_or(0, |subnet| subnet.prefix.length),
        has_prefix_v6,
        prefix_v6.address,
        prefix_v6.prefix.length,
      ),
      "make a vmnet network",
    )?;

    Ok(Self {
      handle: outcome.vmnet_network(),
    })
  }

  /// `VmnetNetwork.subnet`.
  pub fn subnet(&self) -> containerization_extras::address::CIDRv4 {
    self.handle.subnet().cidr_v4()
  }

  /// `VmnetNetwork.prefixV6`.
  pub fn prefix_v6(&self) -> Option<containerization_extras::address::CIDRv6> {
    self.handle.prefix_v6().optional_cidr_v6()
  }

  /// `VmnetNetwork.ipv4Gateway`.
  pub fn ipv4_gateway(&self) -> containerization_extras::address::IPv4Address {
    self.handle.ipv4_gateway().ipv4_address()
  }

  /// `VmnetNetwork.ipv6Gateway`.
  pub fn ipv6_gateway(&self) -> Option<containerization_extras::address::IPv6Address> {
    self.handle.ipv6_gateway().optional_ipv6_address()
  }

  /// `VmnetNetwork.createInterface(_:)`.
  pub fn create_interface(&mut self, id: &str) -> Result<Option<Interface>, Error> {
    platform::outcome(
      self.handle.create_interface(id),
      format!("create an interface for {id}"),
    )
    .map(|outcome| outcome.optional_vmnet_interface())
  }

  /// `VmnetNetwork.createInterface(_:mtu:)`. Rust has no overloading, and the
  /// suffix tells it apart from [`Self::create_interface`].
  pub fn create_interface_with_mtu(&mut self, id: &str, mtu: u32) -> Result<Option<Interface>, Error> {
    platform::outcome(
      self.handle.create_interface_with_mtu(id, mtu),
      format!("create an interface for {id}"),
    )
    .map(|outcome| outcome.optional_vmnet_interface())
  }

  /// `VmnetNetwork.createInterfaceWithoutGateway(_:)`.
  pub fn create_interface_without_gateway(&mut self, id: &str) -> Result<Option<Interface>, Error> {
    platform::outcome(
      self.handle.create_interface_without_gateway(id),
      format!("create an interface for {id}"),
    )
    .map(|outcome| outcome.optional_vmnet_interface())
  }

  /// `VmnetNetwork.releaseInterface(_:)`.
  pub fn release_interface(&mut self, id: &str) -> Result<(), Error> {
    platform::outcome(
      self.handle.release_interface(id),
      format!("release the interface for {id}"),
    )
    .map(drop)
  }
}
