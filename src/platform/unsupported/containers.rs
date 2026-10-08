//! Container manager, Linux containers, pods, processes, mounts and signals.

use super::CzImage;
use super::CzImageStore;
use super::CzNetwork;
use super::CzOutcome;
use super::CzTerminal;
use super::CzVirtualMachineManager;
use super::FilesystemOperationKind;
use super::SocketDirection;
use crate::containerization::container;
use crate::containerization::process;
use crate::containerization::vm;
use crate::containerization_oci;
use crate::platform;
use std::convert::Infallible;

taken!(
  container_manager -> CzContainerManager,
  linux_container -> CzLinuxContainer,
  linux_process -> CzLinuxProcess,
  linux_pod -> CzLinuxPod,
  unix_socket_configuration -> CzUnixSocketConfiguration,
  container_statistics_list -> Vec<container::ContainerStatistics>,
  mount -> container::Mount,
  optional_mount -> Option<container::Mount>,
  container_statistics -> container::ContainerStatistics,
  exit_code -> i32,
  exited_at -> f64,
);

handles!(
  CzContainerManager,
  CzLinuxContainer,
  CzLinuxProcess,
  CzLinuxPod,
  CzUnixSocketConfiguration,
);

failing!(
  cz_container_manager_at_root(vm::Kernel, container::Mount, Option<String>, CzNetwork, bool, bool),
  cz_container_manager_at_root_with_initfs_reference(vm::Kernel, &str, Option<String>, CzNetwork, bool, bool),
  cz_container_manager_with_vmm(CzVirtualMachineManager, CzNetwork),
  cz_linux_container_new(
    &str,
    container::Mount,
    bool,
    container::Mount,
    CzVirtualMachineManager,
    u32,
    u64,
    container::linux_container::Configuration,
  ),
  cz_linux_pod_new(
    &str,
    CzVirtualMachineManager,
    u32,
    u64,
    container::linux_pod::Configuration
  ),
  cz_linux_rlimit_to_oci(process::LinuxRLimit),
  cz_linux_capabilities_to_oci(process::LinuxCapabilities),
  cz_unix_socket_configuration_new(&str, &str),
  cz_mount_clone(container::Mount, &str),
  cz_mount_tag_hash(container::Mount),
  cz_exit_status_new(i32),
  cz_linux_rlimit_kind_parse(&str),
  cz_linux_process_configuration_from_image_config(
    containerization_oci::image::ImageConfig,
    process::LinuxProcessConfiguration,
    platform::ConfigureProcess,
  ),
  cz_linux_process_configuration_set_terminal_io(
    process::LinuxProcessConfiguration,
    CzTerminal,
    platform::ConfigureProcess,
  ),
  cz_signal_parse(&str),
  cz_signal_parse_from(&str, Vec<String>, Vec<i32>),
  cz_signal_linux(),
  cz_signal_platform(),
  cz_signal_platform_name(i32),
  cz_signal_linux_signal(i32),
);

impl CzContainerManager {
  pub(crate) fn image_store(&self) -> CzImageStore {
    match self.0 {}
  }

  pub(crate) fn create_from_reference(
    &self,
    _id: &str,
    _reference: &str,
    _options: container::container_manager::CreateOptions,
    _progress: platform::Progress,
    _seed: container::linux_container::Configuration,
    _configuration: platform::ConfigureContainer,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn create(
    &self,
    _id: &str,
    _image: CzImage,
    _options: container::container_manager::CreateOptions,
    _progress: platform::Progress,
    _seed: container::linux_container::Configuration,
    _configuration: platform::ConfigureContainer,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn create_with_rootfs(
    &self,
    _id: &str,
    _image: CzImage,
    _rootfs: container::Mount,
    _options: container::container_manager::CreateWithRootfsOptions,
    _seed: container::linux_container::Configuration,
    _configuration: platform::ConfigureContainer,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn release_network(&self, _id: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn delete(&self, _id: &str) -> CzOutcome {
    match self.0 {}
  }
}

impl CzLinuxContainer {
  pub(crate) fn id(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn virtual_machine_instance(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn rootfs(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn writable_layer(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn config(
    &self,
    _seed: container::linux_container::Configuration,
    _receive: platform::ConfigureContainer,
  ) {
    match self.0 {}
  }

  pub(crate) fn vm_cpus(&self) -> u32 {
    match self.0 {}
  }

  pub(crate) fn vm_memory_in_bytes(&self) -> u64 {
    match self.0 {}
  }

  pub(crate) fn exec_with(
    &self,
    _id: &str,
    _seed: process::LinuxProcessConfiguration,
    _configuration: platform::ConfigureProcess,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn dial_vsock(&self, _port: u32) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn statistics(&self, _categories: i64) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn filesystem_operation(&self, _operation: FilesystemOperationKind, _path: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn copy_in(
    &self,
    _source: &str,
    _destination: &str,
    _mode: u32,
    _create_parents: bool,
    _chunk_size: usize,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn copy_out(
    &self,
    _source: &str,
    _destination: &str,
    _create_parents: bool,
    _chunk_size: usize,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn create(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn start(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn stop(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn kill(&self, _signal: i32) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn wait(&self, _timeout_in_seconds: Option<i64>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn resize(&self, _width: u16, _height: u16) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn exec(&self, _id: &str, _configuration: process::LinuxProcessConfiguration) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn close_stdin(&self) -> CzOutcome {
    match self.0 {}
  }
}

impl CzLinuxPod {
  pub(crate) fn id(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn config(&self, _seed: container::linux_pod::Configuration, _receive: platform::ConfigurePod) {
    match self.0 {}
  }

  pub(crate) fn vm_cpus(&self) -> u32 {
    match self.0 {}
  }

  pub(crate) fn vm_memory_in_bytes(&self) -> u64 {
    match self.0 {}
  }

  pub(crate) fn add_container(
    &self,
    _id: &str,
    _rootfs: container::Mount,
    _seed: container::linux_pod::ContainerConfiguration,
    _configuration: platform::ConfigurePodContainer,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn create(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn start_container(&self, _id: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn stop_container(&self, _id: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn stop(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn kill_container(&self, _id: &str, _signal: i32) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn wait_container(&self, _id: &str, _timeout_in_seconds: Option<i64>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn resize_container(&self, _id: &str, _width: u16, _height: u16) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn exec_in_container(
    &self,
    _id: &str,
    _process_id: &str,
    _seed: process::LinuxProcessConfiguration,
    _configuration: platform::ConfigureProcess,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn list_containers(&self) -> Vec<String> {
    match self.0 {}
  }

  pub(crate) fn statistics(
    &self,
    _has_container_ids: bool,
    _container_ids: Vec<String>,
    _categories: i64,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn dial_vsock(&self, _port: u32) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn virtual_machine_instance(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn filesystem_operation(&self, _id: &str, _operation: FilesystemOperationKind, _path: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn close_container_stdin(&self, _id: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn relay_unix_socket(&self, _id: &str, _socket: CzUnixSocketConfiguration) -> CzOutcome {
    match self.0 {}
  }
}

impl CzUnixSocketConfiguration {
  pub(crate) fn duplicate(&self) -> CzUnixSocketConfiguration {
    match self.0 {}
  }

  pub(crate) fn id(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn source(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn destination(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn permissions(&self) -> Option<u16> {
    match self.0 {}
  }

  pub(crate) fn direction(&self) -> SocketDirection {
    match self.0 {}
  }

  pub(crate) fn set_source(&self, _source: &str) {
    match self.0 {}
  }

  pub(crate) fn set_destination(&self, _destination: &str) {
    match self.0 {}
  }

  pub(crate) fn set_permissions(&self, _permissions: Option<u16>) {
    match self.0 {}
  }

  pub(crate) fn set_direction(&self, _direction: SocketDirection) {
    match self.0 {}
  }
}

impl CzLinuxProcess {
  pub(crate) fn id(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn owning_container(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn pid(&self) -> i32 {
    match self.0 {}
  }

  pub(crate) fn start(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn kill(&self, _signal: i32) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn resize(&self, _width: u16, _height: u16) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn close_stdin(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn wait(&self, _timeout_in_seconds: Option<i64>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn delete(&self) -> CzOutcome {
    match self.0 {}
  }
}
