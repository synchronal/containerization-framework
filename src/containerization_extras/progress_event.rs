//! `ProgressEvent`, and the `ProgressHandler` that receives them.

use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `ProgressEvent`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgressEvent {
  AddItems(isize),
  AddTotalItems(isize),
  AddSize(i64),
  AddTotalSize(i64),
}

impl ProgressEvent {
  /// `ProgressEvent.event`, such as `add-items`.
  pub fn event(&self) -> Result<String, Error> {
    let (kind, value) = self.kind();

    platform::outcome(ffi::cz_progress_event_event(kind, value), "name a progress event").map(|outcome| outcome.text())
  }

  /// `ProgressEvent.value`. Swift's is an `Int` or an `Int64`, as the case
  /// holds; both fit in an `i64`.
  pub fn value(&self) -> i64 {
    self.kind().1
  }

  /// The case as it crosses to Swift, and its value.
  pub(crate) fn kind(&self) -> (ffi::ProgressKind, i64) {
    match *self {
      ProgressEvent::AddItems(value) => (ffi::ProgressKind::Items, value as i64),
      ProgressEvent::AddTotalItems(value) => (ffi::ProgressKind::TotalItems, value as i64),
      ProgressEvent::AddSize(value) => (ffi::ProgressKind::Size, value),
      ProgressEvent::AddTotalSize(value) => (ffi::ProgressKind::TotalSize, value),
    }
  }
}

/// `ProgressHandler`. Swift calls it with a batch of events, from any thread,
/// and possibly from several at once.
pub type ProgressHandler = Box<dyn Fn(&[ProgressEvent]) + Send + Sync>;
