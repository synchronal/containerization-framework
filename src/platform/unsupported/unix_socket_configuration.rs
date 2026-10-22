//! Stand-ins for `UnixSocketConfiguration.swift`.

use super::CzOutcome;
use super::SocketDirection;
use std::convert::Infallible;

taken!(unix_socket_configuration -> CzUnixSocketConfiguration);

handles!(CzUnixSocketConfiguration);

failing!(cz_unix_socket_configuration_new(&str, &str));

uninhabited!(CzUnixSocketConfiguration {
  fn duplicate() -> CzUnixSocketConfiguration;
  fn id() -> String;
  fn source() -> String;
  fn destination() -> String;
  fn permissions() -> Option<u16>;
  fn direction() -> SocketDirection;
  fn set_source(&str);
  fn set_destination(&str);
  fn set_permissions(Option<u16>);
  fn set_direction(SocketDirection);
});
