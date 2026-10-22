//! Stand-ins for `Vminitd.swift`: the guest agent.

use super::CzOutcome;
use super::CzUnixSocketConfiguration;
use super::CzVirtualMachineInstance;
use super::FilesystemOperationKind;
use crate::containerization::network;
use crate::containerization_extras;
use crate::containerization_oci;
use crate::containerization_os;
use std::convert::Infallible;

taken!(vminitd -> CzVminitd);

handles!(CzVminitd);

uninhabited!(CzVirtualMachineInstance {
  fn dial_agent() -> CzOutcome;
});

uninhabited!(CzVminitd {
  fn duplicate() -> CzVminitd;
  fn standard_setup() -> CzOutcome;
  fn close() -> CzOutcome;
  fn filesystem_operation(FilesystemOperationKind, &str, Option<String>) -> CzOutcome;
  fn getenv(&str) -> CzOutcome;
  fn setenv(&str, &str) -> CzOutcome;
  fn mount(containerization_oci::runtime::Mount) -> CzOutcome;
  fn umount(&str, i32) -> CzOutcome;
  fn mkdir(&str, bool, u32) -> CzOutcome;
  fn kill(i32, i32) -> CzOutcome;
  fn sync() -> CzOutcome;
  fn create_process(
    &str,
    Option<String>,
    Option<u32>,
    Option<u32>,
    Option<u32>,
    Option<String>,
    containerization_oci::runtime::Spec,
    bool,
    Vec<u8>,
  ) -> CzOutcome;
  fn start_process(&str, Option<String>) -> CzOutcome;
  fn signal_process(&str, Option<String>, i32) -> CzOutcome;
  fn resize_process(&str, Option<String>, u32, u32) -> CzOutcome;
  fn wait_process(&str, Option<String>, Option<i64>) -> CzOutcome;
  fn delete_process(&str, Option<String>) -> CzOutcome;
  fn close_process_stdin(&str, Option<String>) -> CzOutcome;
  fn up(&str, Option<u32>) -> CzOutcome;
  fn down(&str) -> CzOutcome;
  fn address_add(&str, containerization_extras::InterfaceAddress) -> CzOutcome;
  fn route_add_link(&str, containerization_extras::LinkRoute) -> CzOutcome;
  fn route_add_default(&str, containerization_extras::DefaultRoute) -> CzOutcome;
  fn configure_dns(network::DNS, &str) -> CzOutcome;
  fn configure_hosts(network::Hosts, &str) -> CzOutcome;
  fn container_statistics(Vec<String>, i64) -> CzOutcome;
  fn setup_emulator(&str, containerization_os::linux::binfmt::Entry) -> CzOutcome;
  fn set_time(i64, i32) -> CzOutcome;
  fn sysctl(Vec<String>, Vec<String>) -> CzOutcome;
  fn stat(&str, &str) -> CzOutcome;
  fn enable_rosetta() -> CzOutcome;
  fn relay_socket(u32, CzUnixSocketConfiguration) -> CzOutcome;
  fn stop_socket_relay(CzUnixSocketConfiguration) -> CzOutcome;
});
