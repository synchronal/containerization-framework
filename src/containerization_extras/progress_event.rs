//! `ProgressEvent`, and the `ProgressHandler` that receives them.

/// `ProgressEvent`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgressEvent {
  AddItems(isize),
  AddTotalItems(isize),
  AddSize(i64),
  AddTotalSize(i64),
}

/// `ProgressHandler`. Swift calls it with a batch of events, from any thread,
/// and possibly from several at once.
pub type ProgressHandler = Box<dyn Fn(&[ProgressEvent]) + Send + Sync>;
