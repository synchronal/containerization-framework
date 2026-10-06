use super::AttachedFilesystem;
use super::Mount;
use super::VirtiofsLayout;
use super::VirtualMachineInstanceState;
use super::VsockListener;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::collections::BTreeMap;
use std::fmt;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;

/// `VZVirtualMachineInstance`. Made by
/// [`super::VzVirtualMachineManager::create`], or reached through
/// [`super::LinuxContainer::with_virtual_machine_instance`].
///
/// `dialAgent()` returns the guest agent's gRPC client, which isn't bound.
/// Neither are `vzVirtualMachine`, `vmQueue`, `withMountRegistry`,
/// `withInstanceLock`, `hotplugProvider`, nor the initializer taking a
/// configuration closure.
pub struct VzVirtualMachineInstance {
  pub(crate) handle: ffi::CzVirtualMachineInstance,
}

// Swift's `VZVirtualMachineInstance` is `Sendable`.
unsafe impl Send for VzVirtualMachineInstance {}
unsafe impl Sync for VzVirtualMachineInstance {}

impl VzVirtualMachineInstance {
  /// `VZVirtualMachineInstance.state`.
  pub fn state(&self) -> VirtualMachineInstanceState {
    self.handle.state().into()
  }

  /// `VZVirtualMachineInstance.mounts`, by the ID of what they're for.
  pub fn mounts(&self) -> BTreeMap<String, Vec<AttachedFilesystem>> {
    self.handle.mounts().attached_filesystems_by_id()
  }

  /// `VZVirtualMachineInstance.virtiofsLayout`.
  pub fn virtiofs_layout(&self) -> VirtiofsLayout {
    self.handle.virtiofs_layout().into()
  }

  /// `VZVirtualMachineInstance.start()`.
  pub fn start(&self) -> Result<(), Error> {
    platform::outcome(self.handle.start(), "start the virtual machine").map(drop)
  }

  /// `VZVirtualMachineInstance.stop()`.
  pub fn stop(&self) -> Result<(), Error> {
    platform::outcome(self.handle.stop(), "stop the virtual machine").map(drop)
  }

  /// `VZVirtualMachineInstance.pause()`.
  pub fn pause(&self) -> Result<(), Error> {
    platform::outcome(self.handle.pause(), "pause the virtual machine").map(drop)
  }

  /// `VZVirtualMachineInstance.resume()`.
  pub fn resume(&self) -> Result<(), Error> {
    platform::outcome(self.handle.resume(), "resume the virtual machine").map(drop)
  }

  /// `VZVirtualMachineInstance.dial(_:)`. Swift hands over the connection's
  /// descriptor.
  pub fn dial(&self, port: u32) -> Result<OwnedFd, Error> {
    platform::outcome(self.handle.dial(port), format!("dial vsock port {port}"))
      // SAFETY: Swift's `FileHandle` doesn't close its descriptor, and
      // forgets it once it crosses.
      .map(|outcome| unsafe { OwnedFd::from_raw_fd(outcome.int32()) })
  }

  /// `VZVirtualMachineInstance.listen(_:)`.
  pub fn listen(&self, port: u32) -> Result<VsockListener, Error> {
    platform::outcome(self.handle.listen(port), format!("listen on vsock port {port}")).map(|outcome| VsockListener {
      handle: outcome.vsock_listener(),
    })
  }

  /// `VZVirtualMachineInstance.hotplug(_:id:)`. It fails unless the instance
  /// has a hotplug provider, which isn't bound.
  pub fn hotplug(&self, block: Mount, id: &str) -> Result<AttachedFilesystem, Error> {
    platform::outcome(self.handle.hotplug(block, id), format!("hotplug a block for {id}"))
      .map(|outcome| outcome.attached_filesystem())
  }

  /// `VZVirtualMachineInstance.registerMounts(id:rootfs:additionalMounts:)`.
  /// Without a hotplug provider it does nothing.
  pub fn register_mounts(
    &self,
    id: &str,
    rootfs: AttachedFilesystem,
    additional_mounts: Vec<Mount>,
  ) -> Result<(), Error> {
    platform::outcome(
      self.handle.register_mounts(id, rootfs, additional_mounts),
      format!("register {id}'s mounts"),
    )
    .map(drop)
  }

  /// `VZVirtualMachineInstance.releaseHotplug(id:)`. Without a hotplug
  /// provider it does nothing.
  pub fn release_hotplug(&self, id: &str) -> Result<(), Error> {
    platform::outcome(self.handle.release_hotplug(id), format!("release {id}'s hotplug")).map(drop)
  }

  /// `VZVirtualMachineInstance.hotplugVirtioFS(_:id:)`. Without a hotplug
  /// provider it does nothing.
  pub fn hotplug_virtio_fs(&self, mounts: Vec<Mount>, id: &str) -> Result<(), Error> {
    platform::outcome(
      self.handle.hotplug_virtio_fs(mounts, id),
      format!("hotplug {id}'s shares"),
    )
    .map(drop)
  }

  /// `VZVirtualMachineInstance.releaseVirtioFS(id:)`. Without a hotplug
  /// provider it does nothing.
  pub fn release_virtio_fs(&self, id: &str) -> Result<(), Error> {
    platform::outcome(self.handle.release_virtio_fs(id), format!("release {id}'s shares")).map(drop)
  }

  /// `VZVirtualMachineInstance.Configuration.installRosetta()`. Swift calls
  /// `fatalError` on a Mac that isn't Apple silicon.
  pub fn install_rosetta() -> Result<(), Error> {
    platform::outcome(ffi::cz_install_rosetta(), "install Rosetta").map(drop)
  }
}

impl fmt::Debug for VzVirtualMachineInstance {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("VzVirtualMachineInstance")
      .field("state", &self.state())
      .finish_non_exhaustive()
  }
}
