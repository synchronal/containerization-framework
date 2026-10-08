use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::fmt;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;

/// `VsockListener`, made by
/// [`super::VZVirtualMachineInstance::listen`]. Swift's is an
/// `AsyncSequence` of connections; this is an [`Iterator`] of their
/// descriptors, whose `next` blocks until one arrives, and ends once
/// [`Self::finish`] is called. `&VsockListener` iterates too, so another
/// thread can finish a listener that one is waiting on, as in Swift.
pub struct VsockListener {
  pub(crate) handle: ffi::CzVsockListener,
}

// Swift's `VsockListener` is `Sendable`, and the bridge lets one `next` wait
// at a time.
unsafe impl Send for VsockListener {}
unsafe impl Sync for VsockListener {}

impl VsockListener {
  /// `VsockListener.port`.
  pub fn port(&self) -> u32 {
    self.handle.port()
  }

  /// `VsockListener.finish()`. Calling it again does nothing, as in Swift.
  pub fn finish(&self) -> Result<(), Error> {
    platform::outcome(
      self.handle.finish(),
      format!("stop listening on vsock port {}", self.port()),
    )
    .map(drop)
  }
}

impl Iterator for &VsockListener {
  type Item = OwnedFd;

  fn next(&mut self) -> Option<OwnedFd> {
    self
      .handle
      .next()
      .optional_int32()
      // SAFETY: Swift's handle for a connection doesn't close its descriptor,
      // and forgets it once it crosses.
      .map(|descriptor| unsafe { OwnedFd::from_raw_fd(descriptor) })
  }
}

impl Iterator for VsockListener {
  type Item = OwnedFd;

  fn next(&mut self) -> Option<OwnedFd> {
    (&*self).next()
  }
}

impl fmt::Debug for VsockListener {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("VsockListener")
      .field("port", &self.port())
      .finish_non_exhaustive()
  }
}
