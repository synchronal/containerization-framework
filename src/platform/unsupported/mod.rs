//! Stand-ins for the bridge on every platform but macOS: the same types and
//! functions, so the wrappers are written once and a cross-platform workspace
//! still compiles.
//!
//! Nothing can make a Swift object here: every constructor's outcome is a
//! failure, and every handle is uninhabited, so its methods never run.

const MACOS_ONLY: &str = "Containerization.framework is macOS only";

/// `ProgressEvent`, less its value.
pub(crate) enum ProgressKind {
  Items,
  TotalItems,
  Size,
  TotalSize,
}

/// swift-log's `Logger.Level`.
pub(crate) enum LogLevel {
  Trace,
  Debug,
  Info,
  Notice,
  Warning,
  Error,
  Critical,
}

/// `Hosts.Entry`'s static constructors.
pub(crate) enum HostsEntryName {
  LocalHostIpv4,
  LocalHostIpv6,
  Ipv6LocalNet,
  Ipv6MulticastPrefix,
  Ipv6AllNodes,
  Ipv6AllRouters,
}

/// `FileDescriptorOps.EntryType`. Only Swift makes one.
#[allow(dead_code)]
pub(crate) enum FdOpsEntryType {
  Regular,
  Directory,
  Symlink,
  Other,
}

/// `FileDescriptorOps.SymlinkPolicy`.
pub(crate) enum FdOpsSymlinkPolicy {
  Refuse,
  FollowBeneath,
}

/// `FilesystemOperation`.
pub(crate) enum FilesystemOperationKind {
  Freeze,
  Thaw,
  Trim,
}

/// vmnet's `operating_modes_t`.
pub(crate) enum VmnetMode {
  Shared,
  Host,
  Bridged,
}

/// `UnixSocketConfiguration.Direction`.
pub(crate) enum SocketDirection {
  Into,
  OutOf,
}

/// `VirtualMachineInstanceState`. Only Swift makes one.
#[allow(dead_code)]
pub(crate) enum InstanceState {
  Starting,
  Running,
  Stopped,
  Stopping,
  Unknown,
}

/// `VirtiofsLayout`. Only Swift makes one.
#[allow(dead_code)]
pub(crate) enum VirtiofsLayoutKind {
  Unified,
  PerTag,
}

macro_rules! taken {
  ($($name:ident -> $type:ty),* $(,)?) => {
    impl CzOutcome {
      $(pub(crate) fn $name(&self) -> $type {
        unreachable!("a failed outcome holds nothing")
      })*
    }
  };
}

/// Declares uninhabited handles. The field is visible to every stand-in file,
/// since a method's Swift file can differ from its type's, as in
/// `CzVirtualMachineInstance::dial_agent`.
macro_rules! handles {
  ($($name:ident),* $(,)?) => {
    $(pub(crate) struct $name(pub(super) Infallible);)*
  };
}

/// Declares free functions, by their argument types, that always fail here.
macro_rules! failing {
  ($($name:ident($($type:ty),* $(,)?)),* $(,)?) => {
    $(pub(crate) fn $name($(_: $type),*) -> CzOutcome {
      CzOutcome
    })*
  };
}

/// Declares a handle's methods, by their argument types after `&self`. None
/// of them runs, since the handle is uninhabited.
macro_rules! uninhabited {
  ($handle:ident { $(fn $name:ident($($type:ty),* $(,)?) $(-> $output:ty)?;)* }) => {
    impl $handle {
      $(pub(crate) fn $name(&self, $(_: $type),*) $(-> $output)? {
        match self.0 {}
      })*
    }
  };
}

// One file per Swift file, as `bridge::ffi` has one `extern "Swift"` block
// per Swift file. A reader `taken!` declares is in the file of the Swift
// model whose getters it reads. The files that hold only readers declare no
// items, so they aren't re-exported.
mod address_model;
mod addresses;
mod archive;
mod authentication;
mod bridged;
mod cloud_hypervisor_model;
mod container_manager;
mod container_model;
mod content_writer;
mod ext4;
mod extras;
mod file_descriptor_ops;
mod image_model;
mod image_store;
mod io;
mod linux_container;
mod linux_pod;
mod local_content_store;
mod os;
mod reference;
mod registry_client;
mod spec;
mod spec_model;
mod unix_socket_configuration;
mod values;
mod virtual_machine;
mod vm_model;
mod vminitd;
mod vmnet_network;

pub(crate) use self::addresses::*;
pub(crate) use self::archive::*;
pub(crate) use self::authentication::*;
pub(crate) use self::bridged::*;
pub(crate) use self::container_manager::*;
pub(crate) use self::content_writer::*;
pub(crate) use self::ext4::*;
pub(crate) use self::extras::*;
pub(crate) use self::file_descriptor_ops::*;
pub(crate) use self::image_model::*;
pub(crate) use self::image_store::*;
pub(crate) use self::io::*;
pub(crate) use self::linux_container::*;
pub(crate) use self::linux_pod::*;
pub(crate) use self::local_content_store::*;
pub(crate) use self::os::*;
pub(crate) use self::reference::*;
pub(crate) use self::registry_client::*;
pub(crate) use self::spec::*;
pub(crate) use self::unix_socket_configuration::*;
pub(crate) use self::values::*;
pub(crate) use self::virtual_machine::*;
pub(crate) use self::vminitd::*;
pub(crate) use self::vmnet_network::*;
