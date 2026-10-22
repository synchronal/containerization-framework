//! Stand-ins for `LinuxContainer.swift`: `LinuxContainer`, `LinuxProcess` and
//! `ContainerStatistics`.

use super::CzOutcome;
use super::CzVirtualMachineManager;
use super::FilesystemOperationKind;
use crate::containerization::container;
use crate::containerization::process;
use crate::platform;
use std::convert::Infallible;

taken!(
  linux_container -> CzLinuxContainer,
  linux_process -> CzLinuxProcess,
  container_statistics -> container::ContainerStatistics,
  container_statistics_list -> Vec<container::ContainerStatistics>,
);

handles!(CzLinuxContainer, CzLinuxProcess);

failing!(cz_linux_container_new(
  &str,
  container::Mount,
  bool,
  container::Mount,
  CzVirtualMachineManager,
  u32,
  u64,
  container::linux_container::Configuration,
));

uninhabited!(CzLinuxContainer {
  fn id() -> String;
  fn virtual_machine_instance() -> CzOutcome;
  fn rootfs() -> CzOutcome;
  fn writable_layer() -> CzOutcome;
  fn config(container::linux_container::Configuration, platform::ConfigureContainer);
  fn vm_cpus() -> u32;
  fn vm_memory_in_bytes() -> u64;
  fn exec_with(&str, process::LinuxProcessConfiguration, platform::ConfigureProcess) -> CzOutcome;
  fn dial_vsock(u32) -> CzOutcome;
  fn statistics(i64) -> CzOutcome;
  fn filesystem_operation(FilesystemOperationKind, &str) -> CzOutcome;
  fn copy_in(&str, &str, u32, bool, usize) -> CzOutcome;
  fn copy_out(&str, &str, bool, usize) -> CzOutcome;
  fn create() -> CzOutcome;
  fn start() -> CzOutcome;
  fn stop() -> CzOutcome;
  fn kill(i32) -> CzOutcome;
  fn wait(Option<i64>) -> CzOutcome;
  fn resize(u16, u16) -> CzOutcome;
  fn exec(&str, process::LinuxProcessConfiguration) -> CzOutcome;
  fn close_stdin() -> CzOutcome;
});

uninhabited!(CzLinuxProcess {
  fn id() -> String;
  fn owning_container() -> Option<String>;
  fn pid() -> i32;
  fn start() -> CzOutcome;
  fn kill(i32) -> CzOutcome;
  fn resize(u16, u16) -> CzOutcome;
  fn close_stdin() -> CzOutcome;
  fn wait(Option<i64>) -> CzOutcome;
  fn delete() -> CzOutcome;
});
