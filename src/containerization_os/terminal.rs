//! `Terminal` and its nested types.

use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::os::fd::RawFd;

/// `Terminal.Size`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Size {
  pub width: u16,
  pub height: u16,
}

/// `Terminal`. Like Swift's, it never closes its descriptor by itself:
/// [`Terminal::close`] does.
pub struct Terminal {
  pub(crate) handle: ffi::CzTerminal,
}

// Swift's `Terminal` is `Sendable`.
unsafe impl Send for Terminal {}
unsafe impl Sync for Terminal {}

impl Terminal {
  /// `Terminal(descriptor:setInitState:)`. Swift's `setInitState` defaults to
  /// `true`.
  pub fn new(descriptor: RawFd, set_init_state: bool) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_terminal_new(descriptor, set_init_state),
      format!("open descriptor {descriptor} as a terminal"),
    )
    .map(|outcome| Self {
      handle: outcome.terminal(),
    })
  }

  /// `Terminal.current`: the first of stderr, stdout and stdin that is a
  /// terminal.
  pub fn current() -> Result<Self, Error> {
    platform::outcome(ffi::cz_terminal_current(), "find the current terminal").map(|outcome| Self {
      handle: outcome.terminal(),
    })
  }

  /// `Terminal.create(initialSize:)`: a new pty, as its parent and child.
  /// Swift's `initialSize` defaults to `nil`, which Swift takes as 120 by 40.
  pub fn create(initial_size: Option<Size>) -> Result<(Self, Self), Error> {
    let size = initial_size.unwrap_or(Size { width: 0, height: 0 });

    platform::outcome(
      ffi::cz_terminal_create(initial_size.is_some(), size.width, size.height),
      "create a pty",
    )
    .map(|outcome| {
      (
        Self {
          handle: outcome.parent_terminal(),
        },
        Self {
          handle: outcome.child_terminal(),
        },
      )
    })
  }

  /// `handle`'s descriptor, which the terminal doesn't own.
  pub fn handle(&self) -> RawFd {
    self.handle.handle()
  }

  /// `write(_:)`.
  pub fn write(&self, data: &[u8]) -> Result<(), Error> {
    platform::outcome(self.handle.write(data.to_vec()), "write to a terminal").map(drop)
  }

  /// `size`.
  pub fn size(&self) -> Result<Size, Error> {
    platform::outcome(self.handle.size(), "read a terminal's size").map(|outcome| Size {
      width: outcome.terminal_size_width(),
      height: outcome.terminal_size_height(),
    })
  }

  /// `resize(from:)`.
  pub fn resize_from(&self, pty: &Terminal) -> Result<(), Error> {
    platform::outcome(
      self.handle.resize_from(pty.handle.duplicate()),
      "resize a terminal from another",
    )
    .map(drop)
  }

  /// `resize(size:)`.
  pub fn resize(&self, size: Size) -> Result<(), Error> {
    platform::outcome(self.handle.resize_size(size.width, size.height), "resize a terminal").map(drop)
  }

  /// `resize(width:height:)`.
  pub fn resize_width_height(&self, width: u16, height: u16) -> Result<(), Error> {
    platform::outcome(self.handle.resize(width, height), "resize a terminal").map(drop)
  }

  /// `setraw()`.
  pub fn setraw(&self) -> Result<(), Error> {
    platform::outcome(self.handle.setraw(), "put a terminal in raw mode").map(drop)
  }

  /// `enableEcho()`.
  pub fn enable_echo(&self) -> Result<(), Error> {
    platform::outcome(self.handle.enable_echo(), "enable a terminal's echo").map(drop)
  }

  /// `disableEcho()`.
  pub fn disable_echo(&self) -> Result<(), Error> {
    platform::outcome(self.handle.disable_echo(), "disable a terminal's echo").map(drop)
  }

  /// `close()`.
  pub fn close(&self) -> Result<(), Error> {
    platform::outcome(self.handle.close(), "close a terminal").map(drop)
  }

  /// `reset()`: the attributes the terminal had when it was made with
  /// `set_init_state`, or else nothing.
  pub fn reset(&self) -> Result<(), Error> {
    platform::outcome(self.handle.reset(), "reset a terminal").map(drop)
  }

  /// `tryReset()`: [`Terminal::reset`], ignoring its error.
  pub fn try_reset(&self) {
    self.handle.try_reset();
  }
}

/// Swift's `Terminal` is a struct, so a copy shares its descriptor.
impl Clone for Terminal {
  fn clone(&self) -> Self {
    Self {
      handle: self.handle.duplicate(),
    }
  }
}
