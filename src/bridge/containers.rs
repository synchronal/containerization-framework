//! The values a container and its processes are configured with, which Swift
//! reads and fills through setters.

use crate::containerization::container::Mount as RustMount;
use crate::containerization::container::UnixSocketConfiguration as RustUnixSocketConfiguration;
use crate::containerization::container::container_manager::CreateOptions as RustCreateOptions;
use crate::containerization::container::container_manager::RootfsCreateOptions as RustRootfsCreateOptions;
use crate::containerization::process::LinuxCapabilities as RustLinuxCapabilities;
use crate::containerization::process::LinuxProcessConfiguration as RustLinuxProcessConfiguration;
use crate::containerization::process::LinuxRLimit as RustLinuxRLimit;
use crate::containerization_oci::runtime::User as RustUser;

use super::ffi::CapabilitySet;
use super::ffi::RlimitKind;
use super::ffi::RuntimeKind;
use super::ffi::SocketDirection;

#[swift_bridge::bridge]
mod ffi {
  #[swift_bridge(already_declared)]
  enum RuntimeKind {}

  #[swift_bridge(already_declared)]
  enum SocketDirection {}

  #[swift_bridge(already_declared)]
  enum RlimitKind {}

  #[swift_bridge(already_declared)]
  enum CapabilitySet {}

  extern "Rust" {
    type RustRootfsCreateOptions;
    #[swift_bridge(swift_name = "hasWritableLayer")]
    fn has_writable_layer(self: &RustRootfsCreateOptions) -> bool;
    #[swift_bridge(swift_name = "writableLayer")]
    fn writable_layer(self: &RustRootfsCreateOptions) -> &RustMount;
    fn networking(self: &RustRootfsCreateOptions) -> bool;
    #[swift_bridge(swift_name = "vmCpus")]
    fn vm_cpus(self: &RustRootfsCreateOptions) -> u32;
    #[swift_bridge(swift_name = "vmMemoryInBytes")]
    fn vm_memory_in_bytes(self: &RustRootfsCreateOptions) -> u64;

    type RustMount;
    #[swift_bridge(swift_name = "mountType")]
    fn mount_type(self: &RustMount) -> &str;
    fn source(self: &RustMount) -> &str;
    fn destination(self: &RustMount) -> &str;
    #[swift_bridge(swift_name = "optionsLen")]
    fn options_len(self: &RustMount) -> usize;
    #[swift_bridge(swift_name = "optionsAt")]
    fn options_at(self: &RustMount, index: usize) -> &str;
    #[swift_bridge(swift_name = "runtimeKind")]
    fn runtime_kind(self: &RustMount) -> RuntimeKind;
    #[swift_bridge(swift_name = "runtimeOptionsLen")]
    fn runtime_options_len(self: &RustMount) -> usize;
    #[swift_bridge(swift_name = "runtimeOptionsAt")]
    fn runtime_options_at(self: &RustMount, index: usize) -> &str;

    type RustUnixSocketConfiguration;
    fn source(self: &RustUnixSocketConfiguration) -> String;
    fn destination(self: &RustUnixSocketConfiguration) -> String;
    fn permissions(self: &RustUnixSocketConfiguration) -> Option<u32>;
    fn direction(self: &RustUnixSocketConfiguration) -> SocketDirection;

    type RustUser;
    fn uid(self: &RustUser) -> u32;
    fn gid(self: &RustUser) -> u32;
    fn umask(self: &RustUser) -> Option<u32>;
    #[swift_bridge(swift_name = "additionalGidsLen")]
    fn additional_gids_len(self: &RustUser) -> usize;
    #[swift_bridge(swift_name = "additionalGidsAt")]
    fn additional_gids_at(self: &RustUser, index: usize) -> u32;
    fn username(self: &RustUser) -> &str;

    type RustLinuxCapabilities;
    #[swift_bridge(swift_name = "setLen")]
    fn set_len(self: &RustLinuxCapabilities, set: CapabilitySet) -> usize;
    #[swift_bridge(swift_name = "setAt")]
    fn set_at(self: &RustLinuxCapabilities, set: CapabilitySet, index: usize) -> &str;

    type RustLinuxProcessConfiguration;
    #[swift_bridge(swift_name = "setArguments")]
    fn set_arguments(self: &mut RustLinuxProcessConfiguration, arguments: Vec<String>);
    #[swift_bridge(swift_name = "setEnvironmentVariables")]
    fn set_environment_variables(self: &mut RustLinuxProcessConfiguration, environment_variables: Vec<String>);
    #[swift_bridge(swift_name = "setWorkingDirectory")]
    fn set_working_directory(self: &mut RustLinuxProcessConfiguration, working_directory: String);
    #[swift_bridge(swift_name = "setUser")]
    fn set_user(
      self: &mut RustLinuxProcessConfiguration,
      uid: u32,
      gid: u32,
      umask: Option<u32>,
      additional_gids: Vec<u32>,
      username: String,
    );
    #[swift_bridge(swift_name = "setNoNewPrivileges")]
    fn set_no_new_privileges(self: &mut RustLinuxProcessConfiguration, no_new_privileges: bool);
    #[swift_bridge(swift_name = "setCapabilities")]
    fn set_capabilities(
      self: &mut RustLinuxProcessConfiguration,
      bounding: Vec<String>,
      effective: Vec<String>,
      inheritable: Vec<String>,
      permitted: Vec<String>,
      ambient: Vec<String>,
    );
    #[swift_bridge(swift_name = "setTerminal")]
    fn set_terminal(self: &mut RustLinuxProcessConfiguration, terminal: bool);
    #[swift_bridge(swift_name = "clearRlimits")]
    fn clear_rlimits(self: &mut RustLinuxProcessConfiguration);
    #[swift_bridge(swift_name = "pushRlimit")]
    fn push_rlimit(self: &mut RustLinuxProcessConfiguration, kind: RlimitKind, hard: u64, soft: u64);
    #[swift_bridge(swift_name = "argumentsLen")]
    fn arguments_len(self: &RustLinuxProcessConfiguration) -> usize;
    #[swift_bridge(swift_name = "argumentsAt")]
    fn arguments_at(self: &RustLinuxProcessConfiguration, index: usize) -> &str;
    #[swift_bridge(swift_name = "environmentVariablesLen")]
    fn environment_variables_len(self: &RustLinuxProcessConfiguration) -> usize;
    #[swift_bridge(swift_name = "environmentVariablesAt")]
    fn environment_variables_at(self: &RustLinuxProcessConfiguration, index: usize) -> &str;
    #[swift_bridge(swift_name = "workingDirectory")]
    fn working_directory(self: &RustLinuxProcessConfiguration) -> &str;
    fn user(self: &RustLinuxProcessConfiguration) -> &RustUser;
    #[swift_bridge(swift_name = "rlimitsLen")]
    fn rlimits_len(self: &RustLinuxProcessConfiguration) -> usize;
    #[swift_bridge(swift_name = "rlimitKindAt")]
    fn rlimit_kind_at(self: &RustLinuxProcessConfiguration, index: usize) -> RlimitKind;
    #[swift_bridge(swift_name = "rlimitHardAt")]
    fn rlimit_hard_at(self: &RustLinuxProcessConfiguration, index: usize) -> u64;
    #[swift_bridge(swift_name = "rlimitSoftAt")]
    fn rlimit_soft_at(self: &RustLinuxProcessConfiguration, index: usize) -> u64;
    #[swift_bridge(swift_name = "noNewPrivileges")]
    fn no_new_privileges(self: &RustLinuxProcessConfiguration) -> bool;
    fn capabilities(self: &RustLinuxProcessConfiguration) -> &RustLinuxCapabilities;
    fn terminal(self: &RustLinuxProcessConfiguration) -> bool;
    fn stdin(self: &RustLinuxProcessConfiguration) -> Option<i32>;
    fn stdout(self: &RustLinuxProcessConfiguration) -> Option<i32>;
    fn stderr(self: &RustLinuxProcessConfiguration) -> Option<i32>;

    type RustCreateOptions;
    #[swift_bridge(swift_name = "rootfsSizeInBytes")]
    fn rootfs_size_in_bytes(self: &RustCreateOptions) -> u64;
    #[swift_bridge(swift_name = "writableLayerSizeInBytes")]
    fn writable_layer_size_in_bytes(self: &RustCreateOptions) -> Option<u64>;
    #[swift_bridge(swift_name = "readOnly")]
    fn read_only(self: &RustCreateOptions) -> bool;
    fn networking(self: &RustCreateOptions) -> bool;
    #[swift_bridge(swift_name = "vmCpus")]
    fn vm_cpus(self: &RustCreateOptions) -> u32;
    #[swift_bridge(swift_name = "vmMemoryInBytes")]
    fn vm_memory_in_bytes(self: &RustCreateOptions) -> u64;

    type RustLinuxRLimit;
    fn kind(self: &RustLinuxRLimit) -> RlimitKind;
    fn hard(self: &RustLinuxRLimit) -> u64;
    fn soft(self: &RustLinuxRLimit) -> u64;
  }
}
