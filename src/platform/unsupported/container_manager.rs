//! Stand-ins for `ContainerManager.swift`.

use super::CzImage;
use super::CzImageStore;
use super::CzNetwork;
use super::CzOutcome;
use super::CzVirtualMachineManager;
use crate::containerization::container;
use crate::containerization::vm;
use crate::platform;
use std::convert::Infallible;

taken!(container_manager -> CzContainerManager);

handles!(CzContainerManager);

failing!(
  cz_container_manager_at_root(vm::Kernel, container::Mount, Option<String>, CzNetwork, bool, bool),
  cz_container_manager_at_root_with_initfs_reference(vm::Kernel, &str, Option<String>, CzNetwork, bool, bool),
  cz_container_manager_with_vmm(CzVirtualMachineManager, CzNetwork),
);

// `ContainerManager`'s inits that take an image store.
uninhabited!(CzImageStore {
  fn container_manager(vm::Kernel, container::Mount, CzNetwork, bool, bool) -> CzOutcome;
  fn container_manager_with_initfs_reference(vm::Kernel, &str, CzNetwork, bool, bool) -> CzOutcome;
});

uninhabited!(CzContainerManager {
  fn image_store() -> CzImageStore;
  fn create_from_reference(
    &str,
    &str,
    container::container_manager::CreateOptions,
    platform::Progress,
    container::linux_container::Configuration,
    platform::ConfigureContainer,
  ) -> CzOutcome;
  fn create(
    &str,
    CzImage,
    container::container_manager::CreateOptions,
    platform::Progress,
    container::linux_container::Configuration,
    platform::ConfigureContainer,
  ) -> CzOutcome;
  fn create_with_rootfs(
    &str,
    CzImage,
    container::Mount,
    container::container_manager::CreateWithRootfsOptions,
    container::linux_container::Configuration,
    platform::ConfigureContainer,
  ) -> CzOutcome;
  fn release_network(&str) -> CzOutcome;
  fn delete(&str) -> CzOutcome;
});
