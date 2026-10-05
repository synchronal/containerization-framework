//! `Hosts`, and its nested `Hosts.Entry`.

use super::strings;

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
}

/// `Hosts`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hosts {
  pub entries: Vec<Entry>,
  pub comment: Option<String>,
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
