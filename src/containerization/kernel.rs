//! `Kernel`, and its nested `Kernel.CommandLine`.

use super::SystemPlatform;
use super::strings;
use std::path::PathBuf;

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
