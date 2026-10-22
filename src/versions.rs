//! The releases this crate is built to run with.
//!
//! Containerization doesn't download a kernel or an init image itself, and
//! its Swift API names neither. A caller downloads both, and these constants
//! say which releases to download.

/// The init image Apple publishes for the Containerization release this
/// crate builds against, which `swift/Package.swift` pins.
///
/// Its `vminitd` shares a protocol with the library, so an init image from
/// another release fails at runtime rather than at build time.
pub const INIT_IMAGE: &str = concat!(
  "ghcr.io/apple/containerization/vminit:",
  env!("CFW_CONTAINERIZATION_VERSION")
);

/// The archive of Kata Containers' static build for arm64, whose kernel
/// Containerization's Makefile uses.
///
/// No one publishes a kernel as an image, so Containerization borrows Kata's.
pub const KERNEL_ARCHIVE_URL: &str =
  "https://github.com/kata-containers/kata-containers/releases/download/3.17.0/kata-static-3.17.0-arm64.tar.xz";

/// The kernel's path in [`KERNEL_ARCHIVE_URL`]. It is a symlink to the
/// versioned kernel in the same directory.
pub const KERNEL_IN_ARCHIVE: &str = "opt/kata/share/kata-containers/vmlinux.container";

#[cfg(test)]
mod tests {
  #[test]
  fn names_the_init_image_for_the_pinned_release() {
    assert_eq!(super::INIT_IMAGE, "ghcr.io/apple/containerization/vminit:0.53.0");
  }

  /// Containerization's Makefile is the kernel's only pin, so a Containerization
  /// update that moves it fails here.
  #[cfg(target_os = "macos")]
  #[test]
  fn downloads_the_kernel_containerizations_makefile_pins() {
    let makefile = std::path::Path::new(env!("CFW_CONTAINERIZATION_CHECKOUT")).join("Makefile");
    let makefile = std::fs::read_to_string(&makefile)
      .unwrap_or_else(|error| panic!("{} should be readable: {error}", makefile.display()));
    let pinned = makefile
      .lines()
      .find_map(|line| line.strip_prefix("KATA_BINARY_PACKAGE := "))
      .expect("Containerization's Makefile should pin KATA_BINARY_PACKAGE");

    assert_eq!(pinned.trim(), super::KERNEL_ARCHIVE_URL);
  }

  /// The integration suites' setup script downloads the kernel itself, so it
  /// keeps its own copy of the pin.
  #[test]
  fn integration_setup_downloads_the_same_kernel() {
    let script = include_str!("../bin/dev/prepare-integration");

    assert!(
      script.contains(&format!("url=\"{}\"", super::KERNEL_ARCHIVE_URL)),
      "bin/dev/prepare-integration should download KERNEL_ARCHIVE_URL"
    );
    assert!(
      script.contains(&format!("kernel_in_archive=\"{}\"", super::KERNEL_IN_ARCHIVE)),
      "bin/dev/prepare-integration should take KERNEL_IN_ARCHIVE from the archive"
    );
  }
}
