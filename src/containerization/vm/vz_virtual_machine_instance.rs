use super::AttachedFilesystem;
use super::VirtiofsLayout;
use super::VirtualMachineInstanceState;
use super::Vminitd;
use super::VsockListener;
use crate::containerization::container;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::collections::BTreeMap;
use std::fmt;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;

/// `VZVirtualMachineInstance`. Made by
/// [`super::VZVirtualMachineManager::create`], or reached through
/// [`crate::containerization::container::LinuxContainer::with_virtual_machine_instance`].
///
/// `vzVirtualMachine`, `vmQueue`, `withMountRegistry`, `withInstanceLock`,
/// `hotplugProvider` and the initializer taking a configuration closure
/// aren't bound.
pub struct VZVirtualMachineInstance {
  pub(crate) handle: ffi::CzVirtualMachineInstance,
}

// Swift's `VZVirtualMachineInstance` is `Sendable`.
unsafe impl Send for VZVirtualMachineInstance {}
unsafe impl Sync for VZVirtualMachineInstance {}

impl VZVirtualMachineInstance {
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

  /// `VZVirtualMachineInstance.dialAgent()`: a client of the agent running in
  /// the guest, connected on [`Vminitd::PORT`].
  pub fn dial_agent(&self) -> Result<Vminitd, Error> {
    platform::outcome(self.handle.dial_agent(), "dial the guest agent").map(|outcome| Vminitd {
      handle: outcome.vminitd(),
    })
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
  pub fn hotplug(&self, block: container::Mount, id: &str) -> Result<AttachedFilesystem, Error> {
    platform::outcome(self.handle.hotplug(block, id), format!("hotplug a block for {id}"))
      .map(|outcome| outcome.attached_filesystem())
  }

  /// `VZVirtualMachineInstance.registerMounts(id:rootfs:additionalMounts:)`.
  /// Without a hotplug provider it does nothing.
  pub fn register_mounts(
    &self,
    id: &str,
    rootfs: AttachedFilesystem,
    additional_mounts: Vec<container::Mount>,
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
  pub fn hotplug_virtio_fs(&self, mounts: Vec<container::Mount>, id: &str) -> Result<(), Error> {
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

impl fmt::Debug for VZVirtualMachineInstance {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("VZVirtualMachineInstance")
      .field("state", &self.state())
      .finish_non_exhaustive()
  }
}
