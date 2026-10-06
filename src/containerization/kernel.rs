//! `Kernel`, and its nested `Kernel.CommandLine`.

use super::SystemPlatform;
use super::strings;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::path::PathBuf;

/// swift-log's `Logger.Level`, which `CommandLine::set_agent_log_level`
/// takes. Like Swift's, it orders from `Trace` to `Critical`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LogLevel {
  Trace,
  Debug,
  Info,
  Notice,
  Warning,
  Error,
  Critical,
}

/// `Kernel.CommandLine`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandLine {
  pub kernel_args: Vec<String>,
  pub init_args: Vec<String>,
}

impl CommandLine {
  /// `Kernel.CommandLine.kernelDefaults`.
  pub fn kernel_defaults() -> Vec<String> {
    strings(&["console=hvc0", "tsc=reliable"])
  }

  /// `Kernel.CommandLine(debug:panic:initArgs:)`.
  pub fn new(debug: bool, panic: i64, init_args: Vec<String>) -> Self {
    let mut kernel_args = Self::kernel_defaults();

    if debug {
      kernel_args.push("debug".to_string());
    }

    kernel_args.push(format!("panic={panic}"));

    Self { kernel_args, init_args }
  }

  /// `Kernel.CommandLine.addDebug()`.
  pub fn add_debug(&mut self) -> Result<(), Error> {
    self.edit(
      ffi::cz_kernel_command_line_add_debug(self.kernel_args.clone(), self.init_args.clone()),
      "add debug to a kernel command line",
    )
  }

  /// `Kernel.CommandLine.addPanic(level:)`.
  pub fn add_panic(&mut self, level: i64) -> Result<(), Error> {
    self.edit(
      ffi::cz_kernel_command_line_add_panic(self.kernel_args.clone(), self.init_args.clone(), level),
      "add a panic level to a kernel command line",
    )
  }

  /// `Kernel.CommandLine.setAgentLogLevel(level:)`.
  pub fn set_agent_log_level(&mut self, level: LogLevel) -> Result<(), Error> {
    self.edit(
      ffi::cz_kernel_command_line_set_agent_log_level(
        self.kernel_args.clone(),
        self.init_args.clone(),
        ffi::LogLevel::from(level),
      ),
      "set the agent's log level on a kernel command line",
    )
  }

  /// Takes the command line Swift changed.
  fn edit(&mut self, outcome: ffi::CzOutcome, action: &str) -> Result<(), Error> {
    let outcome = platform::outcome(outcome, action)?;
    self.kernel_args = outcome.command_line_kernel_args();
    self.init_args = outcome.command_line_init_args();

    Ok(())
  }
}

/// `Kernel`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Kernel {
  pub path: PathBuf,
  pub platform: SystemPlatform,
  pub command_line: CommandLine,
}

impl Kernel {
  /// `Kernel(path:platform:)`, with its default command line.
  pub fn new(path: impl Into<PathBuf>, platform: SystemPlatform) -> Self {
    Self {
      path: path.into(),
      platform,
      command_line: CommandLine::new(false, 0, Vec::new()),
    }
  }

  /// `Kernel.kernelArgs`, its command line's.
  pub fn kernel_args(&self) -> &[String] {
    &self.command_line.kernel_args
  }

  /// `Kernel.initArgs`, its command line's.
  pub fn init_args(&self) -> &[String] {
    &self.command_line.init_args
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn defaults_the_command_line_as_containerization_does() {
    let kernel = Kernel::new("/vmlinux", SystemPlatform::LINUX_ARM);

    assert_eq!(
      kernel.command_line.kernel_args,
      ["console=hvc0", "tsc=reliable", "panic=0"]
    );
    assert!(kernel.command_line.init_args.is_empty());
  }
}
