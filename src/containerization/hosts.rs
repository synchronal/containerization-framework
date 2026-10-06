//! `Hosts`, and its nested `Hosts.Entry`.

use super::strings;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `Hosts.Entry`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
  pub ip_address: String,
  pub hostnames: Vec<String>,
  pub comment: Option<String>,
}

impl Entry {
  /// `Hosts.Entry(ipAddress:hostnames:)`.
  pub fn new(ip_address: impl Into<String>, hostnames: &[&str]) -> Self {
    Self {
      ip_address: ip_address.into(),
      hostnames: strings(hostnames),
      comment: None,
    }
  }

  /// `Hosts.Entry.rendered`: the entry as a line of `/etc/hosts`.
  pub fn rendered(&self) -> Result<String, Error> {
    platform::outcome(ffi::cz_hosts_entry_rendered(self.clone()), "render a hosts entry").map(|outcome| outcome.text())
  }

  /// `Hosts.Entry.localHostIPV4(comment:)`.
  pub fn local_host_ipv4(comment: Option<&str>) -> Result<Self, Error> {
    Self::named(ffi::HostsEntryName::LocalHostIpv4, comment)
  }

  /// `Hosts.Entry.localHostIPV6(comment:)`.
  pub fn local_host_ipv6(comment: Option<&str>) -> Result<Self, Error> {
    Self::named(ffi::HostsEntryName::LocalHostIpv6, comment)
  }

  /// `Hosts.Entry.ipv6LocalNet(comment:)`.
  pub fn ipv6_local_net(comment: Option<&str>) -> Result<Self, Error> {
    Self::named(ffi::HostsEntryName::Ipv6LocalNet, comment)
  }

  /// `Hosts.Entry.ipv6MulticastPrefix(comment:)`.
  pub fn ipv6_multicast_prefix(comment: Option<&str>) -> Result<Self, Error> {
    Self::named(ffi::HostsEntryName::Ipv6MulticastPrefix, comment)
  }

  /// `Hosts.Entry.ipv6AllNodes(comment:)`.
  pub fn ipv6_all_nodes(comment: Option<&str>) -> Result<Self, Error> {
    Self::named(ffi::HostsEntryName::Ipv6AllNodes, comment)
  }

  /// `Hosts.Entry.ipv6AllRouters(comment:)`.
  pub fn ipv6_all_routers(comment: Option<&str>) -> Result<Self, Error> {
    Self::named(ffi::HostsEntryName::Ipv6AllRouters, comment)
  }

  /// The entry one of Swift's static constructors makes.
  fn named(name: ffi::HostsEntryName, comment: Option<&str>) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_hosts_entry_named(name, comment.map(str::to_string)),
      "make a hosts entry",
    )
    .map(|outcome| outcome.hosts_entry())
  }
}

/// `Hosts`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hosts {
  pub entries: Vec<Entry>,
  pub comment: Option<String>,
}

impl Hosts {
  /// `Hosts.hostsFile`: the entries as `/etc/hosts` text.
  pub fn hosts_file(&self) -> Result<String, Error> {
    platform::outcome(ffi::cz_hosts_file(self.clone()), "render a hosts file").map(|outcome| outcome.text())
  }
}

/// `Hosts.default`.
impl Default for Hosts {
  fn default() -> Self {
    Self {
      entries: vec![
        Entry::new("127.0.0.1", &["localhost"]),
        Entry::new("::1", &["localhost", "ip6-localhost", "ip6-loopback"]),
        Entry::new("fe00::", &["ip6-localnet"]),
        Entry::new("ff00::", &["ip6-mcastprefix"]),
        Entry::new("ff02::1", &["ip6-allnodes"]),
        Entry::new("ff02::2", &["ip6-allrouters"]),
      ],
      comment: None,
    }
  }
}
