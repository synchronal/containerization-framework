use super::LinuxCapabilities;
use super::LinuxRLimit;
use super::strings;
use crate::containerization_oci;
use crate::containerization_os::terminal::Terminal;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::os::fd::RawFd;

/// `LinuxProcessConfiguration`.
///
/// `stdin`, `stdout` and `stderr` are descriptors where Swift takes a
/// `ReaderStream` or `Writer`. Swift streams a duplicate of each, so the caller
/// keeps its own.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinuxProcessConfiguration {
  pub arguments: Vec<String>,
  pub environment_variables: Vec<String>,
  pub working_directory: String,
  pub user: containerization_oci::User,
  pub rlimits: Vec<LinuxRLimit>,
  pub no_new_privileges: bool,
  pub capabilities: LinuxCapabilities,
  pub terminal: bool,
  pub stdin: Option<RawFd>,
  pub stdout: Option<RawFd>,
  pub stderr: Option<RawFd>,
}

impl LinuxProcessConfiguration {
  /// `LinuxProcessConfiguration.defaultPath`.
  pub const DEFAULT_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";

  /// `LinuxProcessConfiguration(arguments:)`, its other arguments at their
  /// defaults.
  pub fn new(arguments: &[&str]) -> Self {
    Self {
      arguments: strings(arguments),
      ..Self::default()
    }
  }

  /// `LinuxProcessConfiguration(from:)`: the process an image's
  /// configuration describes.
  pub fn from_image_config(config: &containerization_oci::ImageConfig) -> Result<Self, Error> {
    platform::filled_process(
      |receive| ffi::cz_linux_process_configuration_from_image_config(config.clone(), Self::default(), receive),
      "make a process configuration from an image's",
    )
  }

  /// `LinuxProcessConfiguration.setTerminalIO(terminal:)`: sets what a pty
  /// needs, and `terminal` as stdin and stdout. Where Swift keeps the
  /// terminal, `stdin` and `stdout` hold its descriptor.
  pub fn set_terminal_io(&mut self, terminal: &Terminal) -> Result<(), Error> {
    let process = platform::filled_process(
      |receive| ffi::cz_linux_process_configuration_set_terminal_io(self.clone(), terminal.handle.duplicate(), receive),
      "set a process configuration's terminal",
    )?;

    *self = Self {
      stdin: Some(terminal.handle()),
      stdout: Some(terminal.handle()),
      ..process
    };

    Ok(())
  }
}

/// `LinuxProcessConfiguration()`.
impl Default for LinuxProcessConfiguration {
  fn default() -> Self {
    Self {
      arguments: Vec::new(),
      environment_variables: vec![format!("PATH={}", Self::DEFAULT_PATH)],
      working_directory: "/".to_string(),
      user: containerization_oci::User::default(),
      rlimits: Vec::new(),
      no_new_privileges: false,
      capabilities: LinuxCapabilities::default_oci_capabilities(),
      terminal: false,
      stdin: None,
      stdout: None,
      stderr: None,
    }
  }
}
