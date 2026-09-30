//! Booting a container, and running processes in it.
//!
//! A container lives *in* this process, not a daemon, so it exists exactly as
//! long as the process that booted it. Nothing lists containers: ask
//! [`Session::is_running`], which answers for this process alone.
//!
//! Reaching a container from a second process is the caller's to arrange — this
//! crate hands a guest process whichever descriptors it is given, wherever they
//! came from.

use crate::error::Error;
use crate::model;
use crate::stdio::Stdio;
use crate::store::{Store, StoreError};
use crate::{checked, ffi};
use std::os::fd::RawFd;

pub struct Session {
  store: Store,
}

impl Session {
  pub fn new(store: Store) -> Self {
    Self { store }
  }

  pub fn store(&self) -> &Store {
    &self.store
  }

  /// Creates and starts `spec`'s container, owned by this process
  /// (`ContainerManager.create`, `create`, `start`).
  ///
  /// The container's directory is cleared first: the VM dies with its process,
  /// so nothing cleaned up after the previous run, and a container always
  /// starts from a fresh clone of its image's unpacked rootfs.
  pub fn boot(&self, spec: &model::BootSpec) -> Result<(), Error> {
    let _ = std::fs::remove_dir_all(self.store.container_dir(&spec.id));

    let code = ffi::czbridge_boot(
      spec.clone(),
      &self.store.root().display().to_string(),
      &self.store.kernel().display().to_string(),
      self.store.initfs_reference(),
      &self.store.initfs().display().to_string(),
    );

    checked(code)
      .map(|_| ())
      .map_err(|error| Error::failed(format!("boot {}", spec.id), error))
  }

  /// Runs a process in container `name` until it exits, returning its exit
  /// code. `LinuxContainer.exec`, seeded from the image like the first process.
  ///
  /// `id` names the process for [`Session::resize`]. `stdio` is closed when
  /// the process ends; pass [`Stdio::try_clone`] to keep your own streams. A
  /// terminal gets `TERM=xterm` (as `setTerminalIO`) unless the environment
  /// sets it.
  pub fn exec(
    &self,
    name: &str,
    id: &str,
    configuration: &model::LinuxProcessConfiguration,
    stdio: Stdio,
  ) -> Result<i32, Error> {
    let code = ffi::czbridge_exec(
      name,
      id,
      configuration.clone(),
      stdio.terminal,
      stdio.stdin,
      stdio.stdout,
      stdio.stderr,
    );

    checked(code).map_err(|error| Error::failed("exec", error))
  }

  /// Tells the guest that the terminal `id`'s process reads changed size.
  ///
  /// A no-op once the process has gone, since the window may change size as it
  /// exits. `terminal` is re-read rather than passed as a size, so a stale one
  /// cannot race a second resize.
  pub fn resize(&self, id: &str, terminal: RawFd) -> Result<(), Error> {
    checked(ffi::czbridge_resize(id, terminal))
      .map(|_| ())
      .map_err(|error| Error::failed(format!("resize {id}"), error))
  }

  /// Whether *this process* owns a running container by that name.
  ///
  /// Only this process can say: a container is not registered anywhere outside
  /// the process that booted it.
  pub fn is_running(&self, name: &str) -> bool {
    ffi::czbridge_is_running(name)
  }

  /// Whether [`Session::boot`] can skip unpacking this image. False until its
  /// first (slow) boot, which a caller may want to announce.
  pub fn is_unpacked(&self, image: &str) -> Result<bool, Error> {
    let code = ffi::czbridge_is_unpacked(&self.store.root().display().to_string(), image);

    Ok(checked(code).map_err(|error| Error::failed("find the image", error))? == 1)
  }

  /// Every image the store holds, as `name:tag`. Read from the store's index;
  /// there is no daemon to ask.
  pub fn images(&self) -> Result<Vec<String>, StoreError> {
    self.store.images()
  }

  /// The pinned Containerization release and kernel, which are pinned
  /// separately and whose mismatch would fail only at runtime.
  pub fn version() -> String {
    format!(
      "Containerization {}, kernel {}",
      crate::INITFS_VERSION,
      crate::KERNEL_VERSION
    )
  }
}
