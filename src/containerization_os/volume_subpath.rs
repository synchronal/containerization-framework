//! `VolumeSubpath`, which carries a subpath mount's subpath to the guest as a
//! mount option. A subpath mount binds a path within a volume rather than the
//! whole volume, as Kubernetes' `subPath` does: mounting volume `A`, which
//! holds `foo/bar`, with the subpath `foo` at `/mnt` shows the container only
//! `bar` in `/mnt`.
//!
//! Swift's `VolumeSubpath.open(beneath:subpath:)` exists only on Linux, so it
//! isn't bound.

use crate::containerization::strings;
use crate::platform;
use crate::platform::ffi;
use std::fmt;
use std::io;

/// The prefix of the mount option that carries a subpath. The `x-` prefix
/// follows `mount(8)`, where such options describe a mount without changing
/// how it mounts.
const OPTION_PREFIX: &str = "x-containerization.subpath=";

/// `VolumeSubpath.Error`. Its [`fmt::Display`] is Swift's `description`,
/// except that [`Error::Resolve`] describes its `errno` as [`io::Error`]
/// does.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
  /// The subpath is not a relative path of plain names.
  Invalid { subpath: String, reason: String },
  /// A mount has no subpath option.
  Missing { options: Vec<String> },
  /// A mount with a subpath option is not a non-recursive bind mount.
  NotBind,
  /// The subpath cannot be opened or created beneath the volume.
  Resolve { subpath: String, errno: i32 },
}

impl fmt::Display for Error {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Invalid { subpath, reason } => write!(formatter, "invalid subpath '{subpath}': {reason}"),
      Self::Missing { options } => write!(formatter, "no subpath option in {options:?}"),
      Self::NotBind => formatter.write_str("a subpath needs a non-recursive bind mount"),
      Self::Resolve { subpath, errno } => write!(
        formatter,
        "subpath '{subpath}': cannot be opened: {}",
        io::Error::from_raw_os_error(*errno)
      ),
    }
  }
}

impl std::error::Error for Error {}

/// `VolumeSubpath.toMountOption(_:)`, the mount option that carries
/// `subpath` to the guest.
pub fn to_mount_option(subpath: &str) -> String {
  format!("{OPTION_PREFIX}{subpath}")
}

/// `VolumeSubpath.isSubpathMount(options:)`: whether `options` carry a
/// subpath. It fails with [`Error::NotBind`] when they do but don't make a
/// non-recursive bind mount, since only a bind has a source to resolve the
/// subpath beneath.
pub fn is_subpath_mount(options: &[&str]) -> Result<bool, crate::error::Error> {
  platform::outcome(
    ffi::cz_volume_subpath_is_subpath_mount(strings(options)),
    "check the mount options for a subpath",
  )
  .map(|outcome| outcome.boolean())
}

/// `VolumeSubpath.extract(from:)`: the subpath `options` carry, and the
/// other options in order. It fails with [`Error::Missing`] when there is no
/// subpath, and with [`Error::Invalid`] when there is more than one.
pub fn extract(options: &[&str]) -> Result<(String, Vec<String>), crate::error::Error> {
  platform::outcome(
    ffi::cz_volume_subpath_extract(strings(options)),
    "take the subpath from the mount options",
  )
  .map(|outcome| outcome.volume_subpath_extracted())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn says_what_swift_says() {
    assert_eq!(
      Error::Invalid {
        subpath: "/abs".into(),
        reason: "it is absolute".into()
      }
      .to_string(),
      "invalid subpath '/abs': it is absolute"
    );
    assert_eq!(
      Error::Missing {
        options: vec!["bind".into(), "ro".into()]
      }
      .to_string(),
      r#"no subpath option in ["bind", "ro"]"#
    );
    assert_eq!(Error::NotBind.to_string(), "a subpath needs a non-recursive bind mount");
  }

  #[test]
  fn carries_a_subpath_as_a_mount_option() {
    assert_eq!(to_mount_option("data/cache"), "x-containerization.subpath=data/cache");
  }
}
