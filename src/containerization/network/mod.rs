//! Networks, the interfaces a VM is given on them, and the DNS and hosts
//! configuration a container is given.

mod dns;
pub mod hosts;
mod interface;
mod nat_interface;
pub mod vmnet_network;

pub use self::dns::Dns;
pub use self::hosts::Hosts;
pub use self::interface::Interface;
pub use self::nat_interface::NatInterface;
pub use self::vmnet_network::VmnetNetwork;
