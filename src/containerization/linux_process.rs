use super::ExitStatus;
use super::Signal;
use crate::containerization_os::terminal;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `LinuxProcess`. Made by [`super::LinuxContainer::exec`].
pub struct LinuxProcess {
  pub(crate) handle: ffi::CzLinuxProcess,
}

// Swift's `LinuxProcess` is `Sendable`.
unsafe impl Send for LinuxProcess {}
unsafe impl Sync for LinuxProcess {}

impl LinuxProcess {
  /// `LinuxProcess.id`.
  pub fn id(&self) -> String {
    self.handle.id()
  }

  /// `LinuxProcess.pid`.
  pub fn pid(&self) -> i32 {
    self.handle.pid()
  }

  /// `LinuxProcess.start()`.
  pub fn start(&self) -> Result<(), Error> {
    platform::outcome(self.handle.start(), format!("start {}", self.id())).map(|_| ())
  }

  /// `LinuxProcess.kill(_:)`.
  pub fn kill(&self, signal: Signal) -> Result<(), Error> {
    platform::outcome(self.handle.kill(signal.raw_value), format!("signal {}", self.id())).map(|_| ())
  }

  /// `LinuxProcess.resize(to:)`.
  pub fn resize(&self, to: terminal::Size) -> Result<(), Error> {
    platform::outcome(self.handle.resize(to.width, to.height), format!("resize {}", self.id())).map(|_| ())
  }

  /// `LinuxProcess.closeStdin()`.
  pub fn close_stdin(&self) -> Result<(), Error> {
    platform::outcome(self.handle.close_stdin(), format!("close {}'s stdin", self.id())).map(|_| ())
  }

  /// `LinuxProcess.wait(timeoutInSeconds:)`.
  pub fn wait(&self, timeout_in_seconds: Option<i64>) -> Result<ExitStatus, Error> {
    platform::outcome(self.handle.wait(timeout_in_seconds), format!("wait for {}", self.id()))
      .map(|outcome| platform::exit_status(&outcome))
  }

  /// `LinuxProcess.delete()`.
  pub fn delete(&self) -> Result<(), Error> {
    platform::outcome(self.handle.delete(), format!("delete {}", self.id())).map(|_| ())
  }
}
