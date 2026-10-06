//! `VZVirtualMachineManager`, and the options for its initializer:
//! [`ManagerOptions`].

use super::Kernel;
use super::Mount;
use super::VmConfiguration;
use super::VzVirtualMachineInstance;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::fmt;

/// The defaulted arguments of `VZVirtualMachineManager`'s initializer, after
/// the initial filesystem. [`Default`] is Swift's defaults. `group` and
/// `logger` are SwiftNIO's and swift-log's, and are left at `nil`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ManagerOptions {
  pub rosetta: bool,
  pub nested_virtualization: bool,
}

/// `VZVirtualMachineManager`, which boots VMs with Virtualization.framework.
pub struct VzVirtualMachineManager {
  pub(crate) handle: ffi::CzVirtualMachineManager,
}

// Swift's `VZVirtualMachineManager` is `Sendable`.
unsafe impl Send for VzVirtualMachineManager {}
unsafe impl Sync for VzVirtualMachineManager {}

impl VzVirtualMachineManager {
  /// `VZVirtualMachineManager(kernel:initialFilesystem:rosetta:nestedVirtualization:group:logger:)`.
  pub fn new(kernel: &Kernel, initial_filesystem: &Mount, options: ManagerOptions) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_virtual_machine_manager_new(
        kernel.clone(),
        initial_filesystem.clone(),
        options.rosetta,
        options.nested_virtualization,
      ),
      "make a virtual machine manager",
    )
    .map(|outcome| Self {
      handle: outcome.virtual_machine_manager(),
    })
  }

  /// `VZVirtualMachineManager.create(config:)`, with a `StandardVMConfig` of
  /// `config`. Swift returns `any VirtualMachineInstance`, which on macOS is
  /// always a `VZVirtualMachineInstance`.
  pub fn create(&self, config: &VmConfiguration) -> Result<VzVirtualMachineInstance, Error> {
    platform::outcome(self.handle.create(config.clone()), "create a virtual machine").map(|outcome| {
      VzVirtualMachineInstance {
        handle: outcome.virtual_machine_instance(),
      }
    })
  }
}

/// Swift's `VZVirtualMachineManager` is a struct, so a copy boots the same
/// kernel and initial filesystem.
impl Clone for VzVirtualMachineManager {
  fn clone(&self) -> Self {
    Self {
      handle: self.handle.duplicate(),
    }
  }
}

/// Without its fields, which Swift keeps private.
impl fmt::Debug for VzVirtualMachineManager {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("VzVirtualMachineManager")
      .finish_non_exhaustive()
  }
}
