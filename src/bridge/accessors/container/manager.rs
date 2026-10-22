//! Getters for what a container manager is asked to create, with or without
//! a rootfs.

use crate::containerization::container;

impl container::container_manager::CreateOptions {
  getters! {
    copy rootfs_size_in_bytes: u64;
    copy writable_layer_size_in_bytes: Option<u64>;
    copy read_only: bool;
    copy networking: bool;
  }

  pub(crate) fn vm_cpus(&self) -> u32 {
    self.vm.cpus
  }

  pub(crate) fn vm_memory_in_bytes(&self) -> u64 {
    self.vm.memory_in_bytes
  }
}

impl container::container_manager::CreateWithRootfsOptions {
  getters! {
    present writable_layer: container::Mount => has_writable_layer;
    copy networking: bool;
  }

  pub(crate) fn vm_cpus(&self) -> u32 {
    self.vm.cpus
  }

  pub(crate) fn vm_memory_in_bytes(&self) -> u64 {
    self.vm.memory_in_bytes
  }
}
