//! Stand-ins for `LinuxPod.swift`.

use super::CzOutcome;
use super::CzUnixSocketConfiguration;
use super::CzVirtualMachineManager;
use super::FilesystemOperationKind;
use crate::containerization::container;
use crate::containerization::process;
use crate::platform;
use std::convert::Infallible;

taken!(linux_pod -> CzLinuxPod);

handles!(CzLinuxPod);

failing!(cz_linux_pod_new(
  &str,
  CzVirtualMachineManager,
  u32,
  u64,
  container::linux_pod::Configuration
));

uninhabited!(CzLinuxPod {
  fn id() -> String;
  fn config(container::linux_pod::Configuration, platform::ConfigurePod);
  fn vm_cpus() -> u32;
  fn vm_memory_in_bytes() -> u64;
  fn add_container(
    &str,
    container::Mount,
    container::linux_pod::ContainerConfiguration,
    platform::ConfigurePodContainer,
  ) -> CzOutcome;
  fn create() -> CzOutcome;
  fn start_container(&str) -> CzOutcome;
  fn stop_container(&str) -> CzOutcome;
  fn stop() -> CzOutcome;
  fn kill_container(&str, i32) -> CzOutcome;
  fn wait_container(&str, Option<i64>) -> CzOutcome;
  fn resize_container(&str, u16, u16) -> CzOutcome;
  fn exec_in_container(&str, &str, process::LinuxProcessConfiguration, platform::ConfigureProcess) -> CzOutcome;
  fn list_containers() -> Vec<String>;
  fn statistics(bool, Vec<String>, i64) -> CzOutcome;
  fn dial_vsock(u32) -> CzOutcome;
  fn virtual_machine_instance() -> CzOutcome;
  fn filesystem_operation(&str, FilesystemOperationKind, &str) -> CzOutcome;
  fn close_container_stdin(&str) -> CzOutcome;
  fn relay_unix_socket(&str, CzUnixSocketConfiguration) -> CzOutcome;
});
