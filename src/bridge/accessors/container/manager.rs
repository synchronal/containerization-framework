//! Getters for what a container manager is asked to create, and the
//! filesystem operations it performs.

use crate::bridge::ffi;
use crate::containerization::container;

impl From<container::FilesystemOperation> for ffi::FilesystemOperationKind {
  fn from(operation: container::FilesystemOperation) -> Self {
    match operation {
      container::FilesystemOperation::Freeze => Self::Freeze,
      container::FilesystemOperation::Thaw => Self::Thaw,
      container::FilesystemOperation::Trim => Self::Trim,
    }
  }
}

impl container::container_manager::CreateOptions {
  pub(crate) fn rootfs_size_in_bytes(&self) -> u64 {
    self.rootfs_size_in_bytes
  }

  pub(crate) fn writable_layer_size_in_bytes(&self) -> Option<u64> {
    self.writable_layer_size_in_bytes
  }

  pub(crate) fn read_only(&self) -> bool {
    self.read_only
  }

  pub(crate) fn networking(&self) -> bool {
    self.networking
  }

  pub(crate) fn vm_cpus(&self) -> u32 {
    self.vm.cpus
  }

  pub(crate) fn vm_memory_in_bytes(&self) -> u64 {
    self.vm.memory_in_bytes
  }
}
