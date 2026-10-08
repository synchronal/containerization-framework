//! A lock between test processes.

use std::path::PathBuf;

/// An exclusive lock on a file, released by dropping it. The system also
/// releases it when its process dies, so a crashed test leaves no stale lock.
///
/// Nextest runs each test in a process of its own, so a `Mutex` would not do.
pub struct Lock(std::fs::File);

impl Lock {
  pub fn take(path: PathBuf) -> Self {
    let file = std::fs::File::options()
      .create(true)
      .truncate(false)
      .write(true)
      .open(&path)
      .unwrap_or_else(|error| panic!("could not open {}: {error}", path.display()));

    file
      .lock()
      .unwrap_or_else(|error| panic!("could not take {}: {error}", path.display()));

    Self(file)
  }
}
