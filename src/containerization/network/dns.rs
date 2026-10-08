use crate::containerization::strings;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `DNS`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DNS {
  pub nameservers: Vec<String>,
  pub domain: Option<String>,
  pub search_domains: Vec<String>,
  pub options: Vec<String>,
}

impl DNS {
  /// `DNS.defaultNameservers`.
  pub fn default_nameservers() -> Vec<String> {
    strings(&["1.1.1.1"])
  }

  /// `DNS.validate()`, which fails on a nameserver that isn't an IPv4 or IPv6
  /// address.
  pub fn validate(&self) -> Result<(), Error> {
    platform::outcome(ffi::cz_dns_validate(self.clone()), "validate DNS").map(drop)
  }

  /// `DNS.resolvConf`: the configuration as `/etc/resolv.conf` text.
  pub fn resolv_conf(&self) -> Result<String, Error> {
    platform::outcome(ffi::cz_dns_resolv_conf(self.clone()), "render resolv.conf").map(|outcome| outcome.text())
  }
}

/// `DNS()`.
impl Default for DNS {
  fn default() -> Self {
    Self {
      nameservers: Self::default_nameservers(),
      domain: None,
      search_domains: Vec::new(),
      options: Vec::new(),
    }
  }
}
