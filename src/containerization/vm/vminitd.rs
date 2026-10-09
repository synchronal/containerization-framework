use crate::containerization::container;
use crate::containerization::network;
use crate::containerization::process;
use crate::containerization_extras;
use crate::containerization_oci::runtime;
use crate::containerization_os;
use crate::containerization_os::linux::binfmt;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::collections::BTreeMap;
use std::fmt;

/// `Vminitd`, the client of the agent running in the guest. Made by
/// [`super::VZVirtualMachineInstance::dial_agent`].
///
/// Like Swift's, it keeps its connection until [`Vminitd::close`] closes it.
/// `init(connection:group:)` takes a NIO event loop group and isn't bound;
/// nor are `grpcClient` and `copy(direction:...)`, which take gRPC and
/// generated protobuf types. Nor is `writeFile(path:data:flags:mode:)`:
/// `WriteFileFlags` has no public initializer, so only Containerization
/// itself can make the flags it takes.
pub struct Vminitd {
  pub(crate) handle: ffi::CzVminitd,
}

// Swift's `Vminitd` is `Sendable`.
unsafe impl Send for Vminitd {}
unsafe impl Sync for Vminitd {}

impl Vminitd {
  /// `Vminitd.port`: the vsock port the agent listens on.
  pub const PORT: u32 = 1024;

  /// `Vminitd.standardSetup()`.
  pub fn standard_setup(&self) -> Result<(), Error> {
    platform::outcome(self.handle.standard_setup(), "set up the guest").map(drop)
  }

  /// `Vminitd.close()`.
  pub fn close(&self) -> Result<(), Error> {
    platform::outcome(self.handle.close(), "close the guest agent's connection").map(drop)
  }

  /// `Vminitd.filesystemOperation(operation:path:containerID:)`. Swift's
  /// `containerID` defaults to `nil`.
  pub fn filesystem_operation(
    &self,
    operation: container::FilesystemOperation,
    path: &str,
    container_id: Option<&str>,
  ) -> Result<(), Error> {
    platform::outcome(
      self
        .handle
        .filesystem_operation(operation.into(), path, container_id.map(str::to_string)),
      format!("{operation:?} {path} in the guest"),
    )
    .map(drop)
  }

  /// `Vminitd.getenv(key:)`.
  pub fn getenv(&self, key: &str) -> Result<String, Error> {
    platform::outcome(self.handle.getenv(key), format!("read {key} in the guest")).map(|outcome| outcome.text())
  }

  /// `Vminitd.setenv(key:value:)`.
  pub fn setenv(&self, key: &str, value: &str) -> Result<(), Error> {
    platform::outcome(self.handle.setenv(key, value), format!("set {key} in the guest")).map(drop)
  }

  /// `Vminitd.mount(_:)`.
  pub fn mount(&self, mount: &runtime::Mount) -> Result<(), Error> {
    platform::outcome(
      self.handle.mount(mount.clone()),
      format!("mount {} in the guest", mount.destination),
    )
    .map(drop)
  }

  /// `Vminitd.umount(path:flags:)`.
  pub fn umount(&self, path: &str, flags: i32) -> Result<(), Error> {
    platform::outcome(self.handle.umount(path, flags), format!("unmount {path} in the guest")).map(drop)
  }

  /// `Vminitd.mkdir(path:all:perms:)`.
  pub fn mkdir(&self, path: &str, all: bool, perms: u32) -> Result<(), Error> {
    platform::outcome(self.handle.mkdir(path, all, perms), format!("make {path} in the guest")).map(drop)
  }

  /// `Vminitd.kill(pid:signal:)`, which returns the guest's result.
  pub fn kill(&self, pid: i32, signal: i32) -> Result<i32, Error> {
    platform::outcome(
      self.handle.kill(pid, signal),
      format!("signal process {pid} in the guest"),
    )
    .map(|outcome| outcome.int32())
  }

  /// `Vminitd.sync()`.
  pub fn sync(&self) -> Result<(), Error> {
    platform::outcome(self.handle.sync(), "sync the guest's filesystems").map(drop)
  }

  /// `Vminitd.createProcess(id:containerID:stdinPort:stdoutPort:stderrPort:ociRuntimePath:configuration:options:)`.
  #[allow(clippy::too_many_arguments)]
  pub fn create_process(
    &self,
    id: &str,
    container_id: Option<&str>,
    stdin_port: Option<u32>,
    stdout_port: Option<u32>,
    stderr_port: Option<u32>,
    oci_runtime_path: Option<&str>,
    configuration: &runtime::Spec,
    options: Option<&[u8]>,
  ) -> Result<(), Error> {
    platform::outcome(
      self.handle.create_process(
        id,
        container_id.map(str::to_string),
        stdin_port,
        stdout_port,
        stderr_port,
        oci_runtime_path.map(str::to_string),
        configuration.clone(),
        options.is_some(),
        options.unwrap_or_default().to_vec(),
      ),
      format!("create process {id} in the guest"),
    )
    .map(drop)
  }

  /// `Vminitd.startProcess(id:containerID:)`, which returns the process's
  /// pid.
  pub fn start_process(&self, id: &str, container_id: Option<&str>) -> Result<i32, Error> {
    platform::outcome(
      self
        .handle
        .start_process(id, container_id.map(str::to_string)),
      format!("start process {id} in the guest"),
    )
    .map(|outcome| outcome.int32())
  }

  /// `Vminitd.signalProcess(id:containerID:signal:)`.
  pub fn signal_process(&self, id: &str, container_id: Option<&str>, signal: i32) -> Result<(), Error> {
    platform::outcome(
      self
        .handle
        .signal_process(id, container_id.map(str::to_string), signal),
      format!("signal process {id} in the guest"),
    )
    .map(drop)
  }

  /// `Vminitd.resizeProcess(id:containerID:columns:rows:)`.
  pub fn resize_process(&self, id: &str, container_id: Option<&str>, columns: u32, rows: u32) -> Result<(), Error> {
    platform::outcome(
      self
        .handle
        .resize_process(id, container_id.map(str::to_string), columns, rows),
      format!("resize process {id} in the guest"),
    )
    .map(drop)
  }

  /// `Vminitd.waitProcess(id:containerID:timeoutInSeconds:)`. Swift's
  /// `timeoutInSeconds` defaults to `nil`, which waits for as long as it
  /// takes.
  pub fn wait_process(
    &self,
    id: &str,
    container_id: Option<&str>,
    timeout_in_seconds: Option<i64>,
  ) -> Result<process::ExitStatus, Error> {
    platform::outcome(
      self
        .handle
        .wait_process(id, container_id.map(str::to_string), timeout_in_seconds),
      format!("wait for process {id} in the guest"),
    )
    .map(|outcome| platform::exit_status(&outcome))
  }

  /// `Vminitd.deleteProcess(id:containerID:)`.
  pub fn delete_process(&self, id: &str, container_id: Option<&str>) -> Result<(), Error> {
    platform::outcome(
      self
        .handle
        .delete_process(id, container_id.map(str::to_string)),
      format!("delete process {id} in the guest"),
    )
    .map(drop)
  }

  /// `Vminitd.closeProcessStdin(id:containerID:)`.
  pub fn close_process_stdin(&self, id: &str, container_id: Option<&str>) -> Result<(), Error> {
    platform::outcome(
      self
        .handle
        .close_process_stdin(id, container_id.map(str::to_string)),
      format!("close process {id}'s stdin in the guest"),
    )
    .map(drop)
  }

  /// `Vminitd.up(name:mtu:)`. Swift's `mtu` defaults to `nil`.
  pub fn up(&self, name: &str, mtu: Option<u32>) -> Result<(), Error> {
    platform::outcome(self.handle.up(name, mtu), format!("bring {name} up in the guest")).map(drop)
  }

  /// `Vminitd.down(name:)`.
  pub fn down(&self, name: &str) -> Result<(), Error> {
    platform::outcome(self.handle.down(name), format!("bring {name} down in the guest")).map(drop)
  }

  /// `Vminitd.addressAdd(name:address:)`.
  pub fn address_add(&self, name: &str, address: &containerization_extras::InterfaceAddress) -> Result<(), Error> {
    platform::outcome(
      self.handle.address_add(name, address.clone()),
      format!("add an address to {name} in the guest"),
    )
    .map(drop)
  }

  /// `Vminitd.routeAddLink(name:route:)`.
  pub fn route_add_link(&self, name: &str, route: &containerization_extras::LinkRoute) -> Result<(), Error> {
    platform::outcome(
      self.handle.route_add_link(name, route.clone()),
      format!("add a link route to {name} in the guest"),
    )
    .map(drop)
  }

  /// `Vminitd.routeAddDefault(name:route:)`.
  pub fn route_add_default(&self, name: &str, route: &containerization_extras::DefaultRoute) -> Result<(), Error> {
    platform::outcome(
      self.handle.route_add_default(name, route.clone()),
      format!("add a default route to {name} in the guest"),
    )
    .map(drop)
  }

  /// `Vminitd.configureDNS(config:location:)`.
  pub fn configure_dns(&self, config: &network::DNS, location: &str) -> Result<(), Error> {
    platform::outcome(
      self.handle.configure_dns(config.clone(), location),
      format!("configure DNS at {location} in the guest"),
    )
    .map(drop)
  }

  /// `Vminitd.configureHosts(config:location:)`.
  pub fn configure_hosts(&self, config: &network::Hosts, location: &str) -> Result<(), Error> {
    platform::outcome(
      self.handle.configure_hosts(config.clone(), location),
      format!("configure hosts at {location} in the guest"),
    )
    .map(drop)
  }

  /// `Vminitd.containerStatistics(containerIDs:categories:)`.
  pub fn container_statistics(
    &self,
    container_ids: &[&str],
    categories: container::StatCategory,
  ) -> Result<Vec<container::ContainerStatistics>, Error> {
    platform::outcome(
      self.handle.container_statistics(
        container_ids.iter().map(|id| id.to_string()).collect(),
        categories.raw_value,
      ),
      "read container statistics in the guest",
    )
    .map(|outcome| outcome.container_statistics_list())
  }

  /// `Vminitd.setupEmulator(binaryPath:configuration:)`.
  pub fn setup_emulator(&self, binary_path: &str, configuration: &binfmt::Entry) -> Result<(), Error> {
    platform::outcome(
      self
        .handle
        .setup_emulator(binary_path, configuration.clone()),
      format!("set up {binary_path} as an emulator in the guest"),
    )
    .map(drop)
  }

  /// `Vminitd.setTime(sec:usec:)`.
  pub fn set_time(&self, sec: i64, usec: i32) -> Result<(), Error> {
    platform::outcome(self.handle.set_time(sec, usec), "set the guest's time").map(drop)
  }

  /// `Vminitd.sysctl(settings:)`.
  pub fn sysctl(&self, settings: &BTreeMap<String, String>) -> Result<(), Error> {
    platform::outcome(
      self
        .handle
        .sysctl(settings.keys().cloned().collect(), settings.values().cloned().collect()),
      "set sysctls in the guest",
    )
    .map(drop)
  }

  /// `Vminitd.stat(root:path:)`: `path`'s metadata, resolved as if `root`
  /// were the guest's root directory.
  pub fn stat(&self, root: &str, path: &str) -> Result<containerization_os::Stat, Error> {
    platform::outcome(self.handle.stat(root, path), format!("stat {path} in the guest")).map(|outcome| outcome.stat())
  }

  /// `Vminitd.enableRosetta()`.
  pub fn enable_rosetta(&self) -> Result<(), Error> {
    platform::outcome(self.handle.enable_rosetta(), "enable Rosetta in the guest").map(drop)
  }

  /// `Vminitd.relaySocket(port:configuration:)`.
  pub fn relay_socket(&self, port: u32, configuration: &container::UnixSocketConfiguration) -> Result<(), Error> {
    platform::outcome(
      self
        .handle
        .relay_socket(port, configuration.handle.duplicate()),
      format!("relay a socket over vsock port {port}"),
    )
    .map(drop)
  }

  /// `Vminitd.stopSocketRelay(configuration:)`.
  pub fn stop_socket_relay(&self, configuration: &container::UnixSocketConfiguration) -> Result<(), Error> {
    platform::outcome(
      self
        .handle
        .stop_socket_relay(configuration.handle.duplicate()),
      "stop a socket relay",
    )
    .map(drop)
  }
}

impl fmt::Debug for Vminitd {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.debug_struct("Vminitd").finish_non_exhaustive()
  }
}
