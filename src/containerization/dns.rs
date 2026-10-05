use super::strings;

/// `DNS`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dns {
  pub nameservers: Vec<String>,
  pub domain: Option<String>,
  pub search_domains: Vec<String>,
  pub options: Vec<String>,
}

impl Dns {
  /// `DNS.defaultNameservers`.
  pub fn default_nameservers() -> Vec<String> {
    strings(&["1.1.1.1"])
  }
}

/// `DNS()`.
impl Default for Dns {
  fn default() -> Self {
    Self {
      nameservers: Self::default_nameservers(),
      domain: None,
      search_domains: Vec::new(),
      options: Vec::new(),
    }
  }
}
