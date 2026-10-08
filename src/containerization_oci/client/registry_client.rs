//! `RegistryClient`, the options for its host init:
//! [`RegistryClientOptions`], and the [`RetryOptions`] they take.
//!
//! Neither init takes Swift's `tlsConfiguration` or `logger`, which are
//! swift-nio and swift-log types.

use super::Authentication;
use super::authentication;
use crate::containerization_extras;
use crate::containerization_oci::image;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::fmt;
use std::path::Path;

/// `RetryOptions`, without its `shouldRetry` closure, which takes an
/// AsyncHTTPClient response.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RetryOptions {
  pub max_retries: isize,
  /// In nanoseconds.
  pub retry_interval: u64,
}

impl RetryOptions {
  /// `RetryOptions(maxRetries:retryInterval:)`.
  pub fn new(max_retries: isize, retry_interval: u64) -> Self {
    Self {
      max_retries,
      retry_interval,
    }
  }
}

/// `RegistryClient(host:...)`'s defaulted arguments. [`Default`] is Swift's
/// defaults.
#[derive(Clone, Debug)]
pub struct RegistryClientOptions {
  pub scheme: Option<String>,
  pub port: Option<u16>,
  pub authentication: Option<Authentication>,
  pub client_id: Option<String>,
  pub retry_options: Option<RetryOptions>,
  pub buffer_size: usize,
}

impl Default for RegistryClientOptions {
  fn default() -> Self {
    Self {
      scheme: Some("https".to_string()),
      port: None,
      authentication: None,
      client_id: None,
      retry_options: None,
      buffer_size: 4 * 1024 * 1024,
    }
  }
}

/// `RegistryClient`. Every method talks to the registry.
pub struct RegistryClient {
  handle: ffi::CzRegistryClient,
}

impl fmt::Debug for RegistryClient {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("RegistryClient")
      .finish_non_exhaustive()
  }
}

// Swift's `RegistryClient` is `Sendable`.
unsafe impl Send for RegistryClient {}
unsafe impl Sync for RegistryClient {}

impl RegistryClient {
  /// `RegistryClient(reference:insecure:auth:)`, which retries a request that
  /// a server error answers.
  pub fn new(reference: &str, insecure: bool, auth: Option<&Authentication>) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_registry_client_new(reference, insecure, authentication::crossing(auth)?),
      format!("make a registry client for {reference}"),
    )
    .map(|outcome| Self {
      handle: outcome.registry_client(),
    })
  }

  /// `RegistryClient(host:scheme:port:authentication:clientID:retryOptions:bufferSize:)`.
  /// Rust has no overloading, and the suffix names the argument label that
  /// tells it apart from [`Self::new`].
  pub fn with_host(host: &str, options: RegistryClientOptions) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_registry_client_with_host(
        host,
        options.scheme,
        options.port,
        authentication::crossing(options.authentication.as_ref())?,
        options.client_id,
        options.retry_options.is_some(),
        options
          .retry_options
          .map_or(0, |retry_options| retry_options.max_retries),
        options
          .retry_options
          .map_or(0, |retry_options| retry_options.retry_interval),
        options.buffer_size,
      ),
      format!("make a registry client for {host}"),
    )
    .map(|outcome| Self {
      handle: outcome.registry_client(),
    })
  }

  /// `RegistryClient.ping()`.
  pub fn ping(&self) -> Result<(), Error> {
    platform::outcome(self.handle.ping(), "ping the registry").map(drop)
  }

  /// `RegistryClient.resolve(name:tag:)`.
  pub fn resolve(&self, name: &str, tag: &str) -> Result<image::Descriptor, Error> {
    platform::outcome(self.handle.resolve(name, tag), format!("resolve {name}:{tag}"))
      .map(|outcome| outcome.descriptor())
  }

  /// `RegistryClient.fetchData(name:descriptor:)`.
  pub fn fetch_data(&self, name: &str, descriptor: &image::Descriptor) -> Result<Vec<u8>, Error> {
    platform::outcome(
      self.handle.fetch_data(name, descriptor.clone()),
      format!("fetch {} from {name}", descriptor.digest),
    )
    .map(|outcome| outcome.bytes())
  }

  /// `RegistryClient.fetchBlob(name:descriptor:into:progress:)`: the size,
  /// and the digest as its `digestString`.
  pub fn fetch_blob(
    &self,
    name: &str,
    descriptor: &image::Descriptor,
    into: &Path,
    progress: Option<containerization_extras::ProgressHandler>,
  ) -> Result<(i64, String), Error> {
    let outcome = platform::outcome(
      self.handle.fetch_blob(
        name,
        descriptor.clone(),
        platform::path(into)?,
        platform::Progress(progress),
      ),
      format!("fetch {} from {name} into {}", descriptor.digest, into.display()),
    )?;

    Ok((outcome.written_size(), outcome.written_digest()))
  }

  /// `RegistryClient.catalog(prefix:)`.
  pub fn catalog(&self, prefix: Option<&str>) -> Result<Vec<String>, Error> {
    platform::outcome(
      self.handle.catalog(prefix.map(str::to_string)),
      "list the registry's repositories",
    )
    .map(|outcome| outcome.strings())
  }

  /// `RegistryClient.referrers(name:digest:artifactType:)`.
  pub fn referrers(&self, name: &str, digest: &str, artifact_type: Option<&str>) -> Result<image::Index, Error> {
    platform::outcome(
      self
        .handle
        .referrers(name, digest, artifact_type.map(str::to_string)),
      format!("list what refers to {digest} in {name}"),
    )
    .map(|outcome| outcome.index())
  }
}
