//! `UnixSocketConfiguration`, and its nested `UnixSocketConfiguration.Direction`.

use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;

/// `UnixSocketConfiguration.Direction`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Direction {
  #[default]
  Into,
  OutOf,
}

same_variants!(Direction, ffi::SocketDirection { Into, OutOf });

/// `UnixSocketConfiguration`.
///
/// It holds the Swift value, because Swift gives each configuration a
/// read-only `id` when it is made, and a relay is stopped by that `id`.
pub struct UnixSocketConfiguration {
  pub(crate) handle: ffi::CzUnixSocketConfiguration,
}

// Swift's `UnixSocketConfiguration` is `Sendable`, and the setters take
// `&mut self` because Swift's properties are `var`s of a struct.
unsafe impl Send for UnixSocketConfiguration {}
unsafe impl Sync for UnixSocketConfiguration {}

impl UnixSocketConfiguration {
  /// `UnixSocketConfiguration(source:destination:)`, with the other arguments
  /// left at their defaults.
  pub fn new(source: &Path, destination: &Path) -> Result<Self, Error> {
    let source = platform::path(source)?;
    let destination = platform::path(destination)?;
    let outcome = platform::outcome(
      ffi::cz_unix_socket_configuration_new(source, destination),
      format!("configure a socket relay from {source} to {destination}"),
    )?;

    Ok(Self {
      handle: outcome.unix_socket_configuration(),
    })
  }

  /// `UnixSocketConfiguration.id`.
  pub fn id(&self) -> String {
    self.handle.id()
  }

  /// `UnixSocketConfiguration.source`.
  pub fn source(&self) -> PathBuf {
    PathBuf::from(self.handle.source())
  }

  /// Sets `UnixSocketConfiguration.source`. It fails if the path isn't UTF-8,
  /// which Swift's `String` requires.
  pub fn set_source(&mut self, source: &Path) -> Result<(), Error> {
    self.handle.set_source(platform::path(source)?);
    Ok(())
  }

  /// `UnixSocketConfiguration.destination`.
  pub fn destination(&self) -> PathBuf {
    PathBuf::from(self.handle.destination())
  }

  /// Sets `UnixSocketConfiguration.destination`. It fails if the path isn't
  /// UTF-8, which Swift's `String` requires.
  pub fn set_destination(&mut self, destination: &Path) -> Result<(), Error> {
    self.handle.set_destination(platform::path(destination)?);
    Ok(())
  }

  /// `UnixSocketConfiguration.permissions`, as the mode's raw value.
  pub fn permissions(&self) -> Option<u16> {
    self.handle.permissions()
  }

  /// Sets `UnixSocketConfiguration.permissions` to a mode's raw value.
  pub fn set_permissions(&mut self, permissions: Option<u16>) {
    self.handle.set_permissions(permissions);
  }

  /// `UnixSocketConfiguration.direction`.
  pub fn direction(&self) -> Direction {
    self.handle.direction().into()
  }

  /// Sets `UnixSocketConfiguration.direction`.
  pub fn set_direction(&mut self, direction: Direction) {
    self.handle.set_direction(direction.into());
  }
}

/// Swift's `UnixSocketConfiguration` is a struct, so a copy keeps its `id`.
impl Clone for UnixSocketConfiguration {
  fn clone(&self) -> Self {
    Self {
      handle: self.handle.duplicate(),
    }
  }
}

impl fmt::Debug for UnixSocketConfiguration {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("UnixSocketConfiguration")
      .field("id", &self.id())
      .field("source", &self.source())
      .field("destination", &self.destination())
      .field("permissions", &self.permissions())
      .field("direction", &self.direction())
      .finish()
  }
}

/// Two configurations are equal when their five public fields are equal,
/// including `id`, so only a configuration and its copies are equal. Swift's
/// type isn't `Equatable`. Rust has this so that
/// [`super::linux_container::Configuration`] and
/// [`super::linux_pod::ContainerConfiguration`], which hold sockets, can
/// derive `PartialEq` and `Eq`.
impl PartialEq for UnixSocketConfiguration {
  fn eq(&self, other: &Self) -> bool {
    self.id() == other.id()
      && self.source() == other.source()
      && self.destination() == other.destination()
      && self.permissions() == other.permissions()
      && self.direction() == other.direction()
  }
}

impl Eq for UnixSocketConfiguration {}
