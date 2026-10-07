//! Stand-ins for the bridge on every platform but macOS: the same types and
//! functions, so the wrappers are written once and a cross-platform workspace
//! still compiles.
//!
//! Nothing can make a Swift object here: every constructor's outcome is a
//! failure, and every handle is uninhabited, so its methods never run.

use crate::containerization::container;
use crate::containerization::network;
use crate::containerization::vm;

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

impl From<vm::kernel::LogLevel> for LogLevel {
  fn from(level: vm::kernel::LogLevel) -> Self {
    match level {
      vm::kernel::LogLevel::Trace => Self::Trace,
      vm::kernel::LogLevel::Debug => Self::Debug,
      vm::kernel::LogLevel::Info => Self::Info,
      vm::kernel::LogLevel::Notice => Self::Notice,
      vm::kernel::LogLevel::Warning => Self::Warning,
      vm::kernel::LogLevel::Error => Self::Error,
      vm::kernel::LogLevel::Critical => Self::Critical,
    }
  }
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

/// `FilesystemOperation`.
pub(crate) enum FilesystemOperationKind {
  Freeze,
  Thaw,
  Trim,
}

impl From<container::FilesystemOperation> for FilesystemOperationKind {
  fn from(operation: container::FilesystemOperation) -> Self {
    match operation {
      container::FilesystemOperation::Freeze => Self::Freeze,
      container::FilesystemOperation::Thaw => Self::Thaw,
      container::FilesystemOperation::Trim => Self::Trim,
    }
  }
}

/// vmnet's `operating_modes_t`.
pub(crate) enum VmnetMode {
  Shared,
  Host,
  Bridged,
}

impl From<network::vmnet_network::Mode> for VmnetMode {
  fn from(mode: network::vmnet_network::Mode) -> Self {
    match mode {
      network::vmnet_network::Mode::Shared => Self::Shared,
      network::vmnet_network::Mode::Host => Self::Host,
      network::vmnet_network::Mode::Bridged => Self::Bridged,
    }
  }
}

/// `VirtualMachineInstanceState`.
pub(crate) enum InstanceState {
  Starting,
  Running,
  Stopped,
  Stopping,
  Unknown,
}

impl From<InstanceState> for vm::VirtualMachineInstanceState {
  fn from(state: InstanceState) -> Self {
    match state {
      InstanceState::Starting => Self::Starting,
      InstanceState::Running => Self::Running,
      InstanceState::Stopped => Self::Stopped,
      InstanceState::Stopping => Self::Stopping,
      InstanceState::Unknown => Self::Unknown,
    }
  }
}

/// `VirtiofsLayout`.
pub(crate) enum VirtiofsLayoutKind {
  Unified,
  PerTag,
}

impl From<VirtiofsLayoutKind> for vm::VirtiofsLayout {
  fn from(layout: VirtiofsLayoutKind) -> Self {
    match layout {
      VirtiofsLayoutKind::Unified => Self::Unified,
      VirtiofsLayoutKind::PerTag => Self::PerTag,
    }
  }
}

/// A `Network?`, which is only ever `nil` here.
pub(crate) struct CzNetwork;

pub(crate) fn cz_no_network() -> CzNetwork {
  CzNetwork
}

/// Always a failure, so nothing is taken from it.
pub(crate) struct CzOutcome;

impl CzOutcome {
  pub(crate) fn error(&self) -> Option<String> {
    Some(MACOS_ONLY.to_string())
  }
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

macro_rules! handles {
  ($($name:ident),* $(,)?) => {
    $(pub(crate) struct $name(Infallible);)*
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

mod archive_ext4_io;
mod containers;
mod images;
mod os;
mod values;
mod vm_network;

pub(crate) use self::archive_ext4_io::*;
pub(crate) use self::containers::*;
pub(crate) use self::images::*;
pub(crate) use self::os::*;
pub(crate) use self::values::*;
pub(crate) use self::vm_network::*;
