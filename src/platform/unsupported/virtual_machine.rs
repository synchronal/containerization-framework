//! Stand-ins for `VirtualMachine.swift`: `VZVirtualMachineManager`,
//! `VZVirtualMachineInstance` and `VsockListener`.

use super::CzOutcome;
use super::InstanceState;
use super::VirtiofsLayoutKind;
use crate::containerization::container;
use crate::containerization::vm;
use std::convert::Infallible;

taken!(
  virtual_machine_manager -> CzVirtualMachineManager,
  virtual_machine_instance -> CzVirtualMachineInstance,
  vsock_listener -> CzVsockListener,
);

handles!(CzVirtualMachineManager, CzVirtualMachineInstance, CzVsockListener);

failing!(
  cz_virtual_machine_manager_new(vm::Kernel, container::Mount, bool, bool),
  cz_install_rosetta(),
);

uninhabited!(CzVirtualMachineManager {
  fn duplicate() -> CzVirtualMachineManager;
  fn create(vm::VMConfiguration) -> CzOutcome;
});

uninhabited!(CzVirtualMachineInstance {
  fn state() -> InstanceState;
  fn mounts() -> CzOutcome;
  fn virtiofs_layout() -> VirtiofsLayoutKind;
  fn start() -> CzOutcome;
  fn stop() -> CzOutcome;
  fn pause() -> CzOutcome;
  fn resume() -> CzOutcome;
  fn dial(u32) -> CzOutcome;
  fn listen(u32) -> CzOutcome;
  fn hotplug(container::Mount, &str) -> CzOutcome;
  fn register_mounts(&str, vm::AttachedFilesystem, Vec<container::Mount>) -> CzOutcome;
  fn release_hotplug(&str) -> CzOutcome;
  fn hotplug_virtio_fs(Vec<container::Mount>, &str) -> CzOutcome;
  fn release_virtio_fs(&str) -> CzOutcome;
});

uninhabited!(CzVsockListener {
  fn port() -> u32;
  fn next() -> CzOutcome;
  fn finish() -> CzOutcome;
});
