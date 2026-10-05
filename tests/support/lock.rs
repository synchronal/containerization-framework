//! A lock between test processes.

use std::path::{Path, PathBuf};
use std::time::Duration;

/// Long enough for an image pull and a build, short enough that a lock left
/// by a killed test does not stop the next run.
const LOCK_TIMEOUT: Duration = Duration::from_secs(20 * 60);
const LOCK_POLL: Duration = Duration::from_millis(250);

/// A lock held by a file's existence, released by dropping it. One left by a
/// test that died is taken over after [`LOCK_TIMEOUT`], so a crash costs one
/// slow run rather than every run after it.
///
/// Nextest runs each test in a process of its own, so a `Mutex` would not do.
pub struct Lock(PathBuf);

impl Lock {
  pub fn take(path: PathBuf) -> Self {
    loop {
      match std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
      {
        Ok(_) => return Self(path),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
          if held_too_long(&path) {
            let _ = std::fs::remove_file(&path);
          }
          std::thread::sleep(LOCK_POLL);
        }
        Err(error) => panic!("could not take {}: {error}", path.display()),
      }
    }
  }
}

impl Drop for Lock {
  fn drop(&mut self) {
    let _ = std::fs::remove_file(&self.0);
  }
}

/// Whether whoever made a lock file is gone. A missing one has just been
/// released, which is not a timeout.
fn held_too_long(path: &Path) -> bool {
  std::fs::metadata(path)
    .and_then(|metadata| metadata.modified())
    .is_ok_and(|taken| taken.elapsed().is_ok_and(|held| held > LOCK_TIMEOUT))
}
