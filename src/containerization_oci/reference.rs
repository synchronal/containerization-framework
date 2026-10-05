//! `Reference`, and the options its initializer takes.

use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `Reference`, a handle on Swift's class. [`Reference::normalize`] changes
/// it in place.
pub struct Reference {
  handle: ffi::CzReference,
}

/// The defaulted arguments of `Reference(path:domain:tag:digest:)`. As in
/// Swift, they default to `None`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NewOptions {
  pub domain: Option<String>,
  pub tag: Option<String>,
  pub digest: Option<String>,
}

impl Reference {
  /// `Reference(path:domain:tag:digest:)`.
  pub fn new(path: &str, options: NewOptions) -> Result<Self, Error> {
    Self::made(
      ffi::cz_reference_new(path, options.domain, options.tag, options.digest),
      format!("make a reference to {path}"),
    )
  }

  /// `Reference.parse(_:)`.
  pub fn parse(string: &str) -> Result<Self, Error> {
    Self::made(
      ffi::cz_reference_parse(string),
      format!("parse {string:?} as a reference"),
    )
  }

  /// `Reference.withName(_:)`.
  pub fn with_name(name: &str) -> Result<Self, Error> {
    Self::made(
      ffi::cz_reference_with_name(name),
      format!("make a reference named {name:?}"),
    )
  }

  /// `Reference.resolveDomain(domain:)`.
  pub fn resolve_domain(domain: &str) -> Result<String, Error> {
    platform::outcome(
      ffi::cz_reference_resolve_domain(domain),
      format!("resolve the domain {domain:?}"),
    )
    .map(|outcome| outcome.text())
  }

  fn made(outcome: ffi::CzOutcome, action: String) -> Result<Self, Error> {
    platform::outcome(outcome, action).map(|outcome| Self {
      handle: outcome.reference(),
    })
  }

  /// `Reference.domain`.
  pub fn domain(&self) -> Option<String> {
    self.handle.domain()
  }

  /// `Reference.resolvedDomain`.
  pub fn resolved_domain(&self) -> Option<String> {
    self.handle.resolved_domain()
  }

  /// `Reference.path`.
  pub fn path(&self) -> String {
    self.handle.path()
  }

  /// `Reference.tag`.
  pub fn tag(&self) -> Option<String> {
    self.handle.tag()
  }

  /// `Reference.digest`.
  pub fn digest(&self) -> Option<String> {
    self.handle.digest()
  }

  /// `Reference.name`.
  pub fn name(&self) -> String {
    self.handle.name()
  }

  /// `Reference.description`.
  pub fn description(&self) -> String {
    self.handle.description()
  }

  /// `Reference.withTag(_:)`.
  pub fn with_tag(&self, tag: &str) -> Result<Self, Error> {
    Self::made(
      self.handle.with_tag(tag),
      format!("tag {} with {tag:?}", self.description()),
    )
  }

  /// `Reference.withDigest(_:)`.
  pub fn with_digest(&self, digest: &str) -> Result<Self, Error> {
    Self::made(
      self.handle.with_digest(digest),
      format!("give {} the digest {digest:?}", self.description()),
    )
  }

  /// `Reference.normalize()`.
  pub fn normalize(&mut self) {
    self.handle.normalize();
  }
}
