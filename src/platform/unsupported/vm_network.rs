//! Virtual machines, their guest agent, vsock, vmnet networks and
//! interfaces, kernels, DNS and hosts.

use super::CzNetwork;
use super::CzOutcome;
use super::FilesystemOperationKind;
use super::HostsEntryName;
use super::InstanceState;
use super::LogLevel;
use super::VirtiofsLayoutKind;
use super::VmnetMode;
use crate::containerization::container;
use crate::containerization::network;
use crate::containerization::vm;
use crate::containerization_extras;
use crate::containerization_extras::address::IPv6Address;
use crate::containerization_oci;
use crate::containerization_os;
use std::collections::BTreeMap;
use std::convert::Infallible;

taken!(
  vmnet_network -> CzVmnetNetwork,
  optional_vmnet_interface -> Option<network::vmnet_network::Interface>,
  virtual_machine_manager -> CzVirtualMachineManager,
  virtual_machine_instance -> CzVirtualMachineInstance,
  vminitd -> CzVminitd,
  vsock_listener -> CzVsockListener,
  attached_filesystem -> vm::AttachedFilesystem,
  attached_filesystems_by_id -> BTreeMap<String, Vec<vm::AttachedFilesystem>>,
  kernel -> vm::Kernel,
  command_line_kernel_args -> Vec<String>,
  command_line_init_args -> Vec<String>,
  hosts_entry -> network::hosts::Entry,
);

handles!(
  CzVmnetNetwork,
  CzVmnetInterface,
  CzVirtualMachineManager,
  CzVirtualMachineInstance,
  CzVminitd,
  CzVsockListener,
);

failing!(
  cz_vmnet_network_new(VmnetMode, Option<u32>, u8, bool, IPv6Address, u8),
  cz_virtual_machine_manager_new(vm::Kernel, container::Mount, bool, bool),
  cz_install_rosetta(),
  cz_kernel_command_line_add_debug(Vec<String>, Vec<String>),
  cz_kernel_command_line_add_panic(Vec<String>, Vec<String>, i64),
  cz_kernel_command_line_set_agent_log_level(Vec<String>, Vec<String>, LogLevel),
  cz_dns_validate(network::Dns),
  cz_dns_resolv_conf(network::Dns),
  cz_hosts_file(network::Hosts),
  cz_hosts_entry_rendered(network::hosts::Entry),
  cz_hosts_entry_named(HostsEntryName, Option<String>),
  cz_system_platform_oci_platform(vm::SystemPlatform),
);

impl CzVmnetNetwork {
  pub(crate) fn as_network(&self) -> CzNetwork {
    match self.0 {}
  }

  pub(crate) fn subnet(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn prefix_v6(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn ipv4_gateway(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn ipv6_gateway(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn create_interface(&self, _id: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn create_interface_with_mtu(&self, _id: &str, _mtu: u32) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn create_interface_without_gateway(&self, _id: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn release_interface(&self, _id: &str) -> CzOutcome {
    match self.0 {}
  }
}

impl CzVmnetInterface {
  pub(crate) fn duplicate(&self) -> CzVmnetInterface {
    match self.0 {}
  }

  pub(crate) fn ipv4_address(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn ipv4_gateway(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn ipv6_address(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn ipv6_gateway(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn mac_address(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn mtu(&self) -> u32 {
    match self.0 {}
  }
}

impl CzVirtualMachineManager {
  pub(crate) fn duplicate(&self) -> CzVirtualMachineManager {
    match self.0 {}
  }

  pub(crate) fn create(&self, _config: vm::VmConfiguration) -> CzOutcome {
    match self.0 {}
  }
}

impl CzVirtualMachineInstance {
  pub(crate) fn state(&self) -> InstanceState {
    match self.0 {}
  }

  pub(crate) fn mounts(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn virtiofs_layout(&self) -> VirtiofsLayoutKind {
    match self.0 {}
  }

  pub(crate) fn start(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn stop(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn pause(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn resume(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn dial(&self, _port: u32) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn listen(&self, _port: u32) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn hotplug(&self, _block: container::Mount, _id: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn register_mounts(
    &self,
    _id: &str,
    _rootfs: vm::AttachedFilesystem,
    _additional_mounts: Vec<container::Mount>,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn release_hotplug(&self, _id: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn hotplug_virtio_fs(&self, _mounts: Vec<container::Mount>, _id: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn release_virtio_fs(&self, _id: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn dial_agent(&self) -> CzOutcome {
    match self.0 {}
  }
}

impl CzVminitd {
  pub(crate) fn standard_setup(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn close(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn filesystem_operation(
    &self,
    _operation: FilesystemOperationKind,
    _path: &str,
    _container_id: Option<String>,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn getenv(&self, _key: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn setenv(&self, _key: &str, _value: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn mount(&self, _mount: containerization_oci::runtime::Mount) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn umount(&self, _path: &str, _flags: i32) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn mkdir(&self, _path: &str, _all: bool, _perms: u32) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn kill(&self, _pid: i32, _signal: i32) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn sync(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn create_process(
    &self,
    _id: &str,
    _container_id: Option<String>,
    _stdin_port: Option<u32>,
    _stdout_port: Option<u32>,
    _stderr_port: Option<u32>,
    _oci_runtime_path: Option<String>,
    _configuration: containerization_oci::runtime::Spec,
    _has_options: bool,
    _options: Vec<u8>,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn start_process(&self, _id: &str, _container_id: Option<String>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn signal_process(&self, _id: &str, _container_id: Option<String>, _signal: i32) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn resize_process(
    &self,
    _id: &str,
    _container_id: Option<String>,
    _columns: u32,
    _rows: u32,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn wait_process(
    &self,
    _id: &str,
    _container_id: Option<String>,
    _timeout_in_seconds: Option<i64>,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn delete_process(&self, _id: &str, _container_id: Option<String>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn close_process_stdin(&self, _id: &str, _container_id: Option<String>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn up(&self, _name: &str, _mtu: Option<u32>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn down(&self, _name: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn address_add(&self, _name: &str, _address: containerization_extras::InterfaceAddress) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn route_add_link(&self, _name: &str, _route: containerization_extras::LinkRoute) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn route_add_default(&self, _name: &str, _route: containerization_extras::DefaultRoute) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn configure_dns(&self, _config: network::Dns, _location: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn configure_hosts(&self, _config: network::Hosts, _location: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn container_statistics(&self, _container_ids: Vec<String>, _categories: i64) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn setup_emulator(
    &self,
    _binary_path: &str,
    _configuration: containerization_os::binfmt::Entry,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn set_time(&self, _sec: i64, _usec: i32) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn sysctl(&self, _keys: Vec<String>, _values: Vec<String>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn stat(&self, _root: &str, _path: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn enable_rosetta(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn relay_socket(&self, _port: u32, _configuration: container::UnixSocketConfiguration) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn stop_socket_relay(&self, _configuration: container::UnixSocketConfiguration) -> CzOutcome {
    match self.0 {}
  }
}

impl CzVsockListener {
  pub(crate) fn port(&self) -> u32 {
    match self.0 {}
  }

  pub(crate) fn next(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn finish(&self) -> CzOutcome {
    match self.0 {}
  }
}
