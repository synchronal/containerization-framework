//! Where the boot artefacts live.
//!
//! Containerization's `ImageStore` layout, which `ContainerManager` opens as-is:
//! an image index in `state.json`, blobs under `content/blobs/sha256`, and one
//! rootfs per container under `containers/<id>`.
//!
//! Ours: `build-cache` (the builder's step snapshots) and `unpacked` (each
//! image's rootfs, unpacked once and cloned per container).
//!
//! The kernel and unpacked init image live wherever the caller says, as with
//! Containerization's `Kernel(path:)` and `ContainerManager(initfs:)`.
//!
//! Everything is re-creatable by `provision` and `build`, hence `~/.cache`.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// Map of reference to OCI descriptor.
const INDEX: &str = "state.json";
const CONTAINERS: &str = "containers";

/// The init image release this crate is pinned to: the initfs carrying
/// `vminitd`, the agent the library talks to over vsock.
///
/// Must match the `containerization` release in
/// this crate's `swift/Package.swift`: they share a protocol,
/// and a mismatch fails at runtime, not build time.
macro_rules! initfs_version {
  () => {
    "0.47.0"
  };
}

macro_rules! kernel_version {
  () => {
    "3.17.0"
  };
}

pub const INITFS_VERSION: &str = initfs_version!();
pub const INITFS_REFERENCE: &str = concat!("ghcr.io/apple/containerization/vminit:", initfs_version!());

/// The kernel `provision` fetches, and its path inside the archive.
///
/// Kata Containers' static build, the release Containerization's Makefile
/// pins. Downloaded, since no one publishes a kernel as an image.
pub const KERNEL_VERSION: &str = kernel_version!();

/// The archive [`KERNEL_VERSION`] is published in.
pub const KERNEL_URL: &str = concat!(
  "https://github.com/kata-containers/kata-containers/releases/download/",
  kernel_version!(),
  "/kata-static-",
  kernel_version!(),
  "-arm64.tar.xz"
);
/// The kernel's path inside [`KERNEL_URL`]'s archive.
pub const KERNEL_IN_ARCHIVE: &str = "opt/kata/share/kata-containers/vmlinux.container";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Store {
  root: PathBuf,
  kernel: PathBuf,
  initfs_reference: String,
  initfs: PathBuf,
}

#[derive(Debug, Eq, PartialEq)]
pub enum StoreError {
  /// Never provisioned.
  Missing(PathBuf),
  /// Lacks the kernel or init image a boot needs.
  Incomplete { root: PathBuf, missing: String },
  /// The index exists but can't be read.
  Unreadable { path: PathBuf, source: String },
}

impl fmt::Display for StoreError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Missing(root) => write!(formatter, "no image store at {}", root.display()),
      Self::Incomplete { root, missing } => {
        write!(formatter, "the image store at {} has no {missing}", root.display())
      }
      Self::Unreadable { path, source } => write!(formatter, "cannot read {}: {source}", path.display()),
    }
  }
}

impl std::error::Error for StoreError {}

impl Store {
  /// Names a store without touching it; the first `build` provisions it.
  ///
  /// - `root`: the image store.
  /// - `kernel`: the kernel VMs boot.
  /// - `initfs_reference`: the init image, e.g. [`INITFS_REFERENCE`]; its
  ///   `vminitd` must match this crate's Containerization release.
  /// - `initfs`: where that image is unpacked and booted from.
  ///
  /// Provisioning fills an empty path and leaves an existing file alone, so
  /// name paths for [`KERNEL_VERSION`] and [`INITFS_VERSION`] to pick up pin
  /// changes.
  pub fn at(
    root: impl Into<PathBuf>,
    kernel: impl Into<PathBuf>,
    initfs_reference: impl Into<String>,
    initfs: impl Into<PathBuf>,
  ) -> Self {
    Self {
      root: root.into(),
      kernel: kernel.into(),
      initfs_reference: initfs_reference.into(),
      initfs: initfs.into(),
    }
  }

  /// The kernel this store's VMs boot.
  pub fn kernel(&self) -> &Path {
    &self.kernel
  }

  /// The init image this store provisions.
  pub fn initfs_reference(&self) -> &str {
    &self.initfs_reference
  }

  /// Where that init image is unpacked, and booted from.
  pub fn initfs(&self) -> &Path {
    &self.initfs
  }

  /// Whether this store holds what a boot needs, naming the first thing
  /// missing. Worth asking before a boot, which would fail late instead.
  pub fn ready(&self) -> Result<(), StoreError> {
    if !self.root.is_dir() {
      return Err(StoreError::Missing(self.root.clone()));
    }

    if !self.kernel.is_file() {
      return Err(StoreError::Incomplete {
        root: self.root.clone(),
        missing: format!("kernel at {}", self.kernel.display()),
      });
    }

    if !self.holds(&self.initfs_reference) {
      return Err(StoreError::Incomplete {
        root: self.root.clone(),
        missing: format!("{} in {INDEX}", self.initfs_reference),
      });
    }

    Ok(())
  }

  pub fn root(&self) -> &Path {
    &self.root
  }

  /// A container's rootfs, kernel copy, and boot log.
  pub fn container_dir(&self, name: &str) -> PathBuf {
    self.root.join(CONTAINERS).join(name)
  }

  /// Every image the store holds, as `name:tag`, sorted. No index yet means
  /// none, not an error: that's a first run.
  pub fn images(&self) -> Result<Vec<String>, StoreError> {
    Ok(self.references()?.into_keys().collect())
  }

  /// Whether the index names an image. An unreadable index names none.
  pub fn holds(&self, reference: &str) -> bool {
    self
      .references()
      .is_ok_and(|references| references.contains_key(reference))
  }

  /// The index's references, their descriptors skipped rather than parsed.
  fn references(&self) -> Result<BTreeMap<String, serde::de::IgnoredAny>, StoreError> {
    let index = self.root.join(INDEX);

    if !index.exists() {
      return Ok(BTreeMap::new());
    }

    let unreadable = |source: String| StoreError::Unreadable {
      path: index.clone(),
      source,
    };
    let text = std::fs::read_to_string(&index).map_err(|source| unreadable(source.to_string()))?;

    serde_json::from_str(&text).map_err(|source| unreadable(source.to_string()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const CUSTOM_INIT_IMAGE: &str = "docker.io/example/vminit:0.47.0-runc";

  /// A store at `root`, its kernel and init image beside the image store.
  fn store_at(root: &Path) -> Store {
    Store::at(root, root.join("vmlinux"), INITFS_REFERENCE, root.join("vminit.ext4"))
  }

  /// Puts an (empty) kernel where the store boots one from.
  fn place_kernel(store: &Store) {
    std::fs::write(store.kernel(), "").expect("kernel");
  }

  /// As Foundation writes it, slashes escaped. An unescaped fixture hid a bug.
  fn index_holding(reference: &str) -> String {
    format!("{{\"{}\":{{}}}}", reference.replace('/', "\\/"))
  }

  #[test]
  fn boots_the_kernel_and_init_image_it_is_given() {
    let store = Store::at("/store", "/cache/vmlinux", CUSTOM_INIT_IMAGE, "/cache/vminit-runc.ext4");

    assert_eq!(store.root(), Path::new("/store"));
    assert_eq!(store.kernel(), Path::new("/cache/vmlinux"));
    assert_eq!(store.initfs_reference(), CUSTOM_INIT_IMAGE);
    assert_eq!(store.initfs(), Path::new("/cache/vminit-runc.ext4"));
  }

  #[test]
  fn is_not_ready_before_anything_has_provisioned_it() {
    assert_eq!(
      store_at(Path::new("/nonexistent/images")).ready(),
      Err(StoreError::Missing(PathBuf::from("/nonexistent/images")))
    );
  }

  #[test]
  fn names_the_kernel_it_could_not_find() {
    let root = tempfile::tempdir().expect("a temp dir");
    let store = store_at(root.path());

    let error = store.ready().expect_err("a store with no kernel");

    assert!(
      error
        .to_string()
        .contains(&store.kernel().display().to_string()),
      "error should name the kernel's path: {error}"
    );
  }

  #[test]
  fn names_the_reference_it_could_not_find() {
    let root = tempfile::tempdir().expect("a temp dir");
    let store = store_at(root.path());
    place_kernel(&store);
    std::fs::write(root.path().join(INDEX), "{}").expect("index");

    let error = store.ready().expect_err("an incomplete store");

    assert!(
      error.to_string().contains(INITFS_REFERENCE),
      "error should name the missing image: {error}"
    );
  }

  #[test]
  fn is_ready_only_with_the_init_image_it_boots() {
    let root = tempfile::tempdir().expect("a temp dir");
    let store = Store::at(
      root.path(),
      root.path().join("vmlinux"),
      CUSTOM_INIT_IMAGE,
      root.path().join("runc.ext4"),
    );
    place_kernel(&store);
    std::fs::write(root.path().join(INDEX), index_holding(INITFS_REFERENCE)).expect("index");

    let error = store
      .ready()
      .expect_err("a store holding another init image");

    assert!(
      error.to_string().contains(CUSTOM_INIT_IMAGE),
      "error should name the missing image: {error}"
    );
  }

  #[test]
  fn is_ready_when_it_holds_the_kernel_and_the_initfs() {
    let root = tempfile::tempdir().expect("a temp dir");
    let store = store_at(root.path());
    place_kernel(&store);
    std::fs::write(root.path().join(INDEX), index_holding(INITFS_REFERENCE)).expect("index");

    store.ready().expect("a complete store");
    assert_eq!(
      store.container_dir("session-cb"),
      root.path().join("containers/session-cb")
    );
  }

  /// A first run: an empty directory.
  #[test]
  fn holds_no_images_before_the_first_build() {
    let root = tempfile::tempdir().expect("a temp dir");

    assert_eq!(
      store_at(root.path())
        .images()
        .expect("an empty store is readable"),
      Vec::<String>::new()
    );
  }

  /// As Containerization writes it: nested objects, escaped slashes, and an
  /// `annotations` key that is not an image.
  #[test]
  fn lists_the_references_the_index_names() {
    let root = tempfile::tempdir().expect("a temp dir");
    std::fs::write(
      root.path().join(INDEX),
      r#"{"example\/base:latest":{"digest":"sha256:aa","annotations":{"org.opencontainers.image.ref.name":"latest"}},"docker.io\/library\/debian:stable-slim":{"digest":"sha256:bb"}}"#,
    )
    .expect("index");

    assert_eq!(
      store_at(root.path())
        .images()
        .expect("the index should be readable"),
      ["docker.io/library/debian:stable-slim", "example/base:latest"]
    );
  }

  #[test]
  fn names_the_index_it_could_not_parse() {
    let root = tempfile::tempdir().expect("a temp dir");
    std::fs::write(root.path().join(INDEX), "{\"truncated").expect("index");

    let store = store_at(root.path());

    assert!(matches!(
      store.images(),
      Err(StoreError::Unreadable { path, .. }) if path == root.path().join(INDEX)
    ));
    assert!(!store.holds(INITFS_REFERENCE));
  }

  #[test]
  fn finds_a_reference_whose_slashes_are_escaped() {
    let root = tempfile::tempdir().expect("a temp dir");
    std::fs::write(root.path().join(INDEX), index_holding(INITFS_REFERENCE)).expect("index");

    let store = store_at(root.path());

    assert!(store.holds(INITFS_REFERENCE));
    assert!(!store.holds("ghcr.io/apple/containerization/vminit:0.0.0"));
  }
}
