//! What a caller describes: a container to boot, a process to run in one, and
//! an image to build.
//!
//! Container and process types mirror Containerization's
//! (`LinuxContainer.Configuration`, `LinuxProcessConfiguration`, `Mount`, ...)
//! in names, shapes and defaults, so its documentation applies. Fields the
//! image seeds are `Option`s here; `None` keeps the image's.
//!
//! Plain, platform-free data. Swift reads it in place through the bridge.

mod build;
mod container;

pub use self::build::*;
pub use self::container::*;

const MIB: u64 = 1024 * 1024;
const GIB: u64 = 1024 * MIB;

fn strings(values: &[&str]) -> Vec<String> {
  values.iter().map(|value| value.to_string()).collect()
}
