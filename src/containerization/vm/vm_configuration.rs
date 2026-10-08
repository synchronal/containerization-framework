use super::BootLog;
use crate::containerization::GIB;
use crate::containerization::container;
use crate::containerization::network;
use std::collections::BTreeMap;

/// `VMConfiguration`, which [`super::VZVirtualMachineManager::create`] takes.
/// `StandardVMConfig` only wraps one, so Rust takes it directly. `extensions`
/// isn't bound.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VMConfiguration {
  pub cpus: u32,
  pub memory_in_bytes: u64,
  pub interfaces: Vec<network::Interface>,
  /// Mounts by the ID of what they're for, such as a container's.
  pub mounts_by_id: BTreeMap<String, Vec<container::Mount>>,
  pub boot_log: Option<BootLog>,
  pub nested_virtualization: bool,
}

/// `VMConfiguration()`: 4 CPUs and 1024 MiB.
impl Default for VMConfiguration {
  fn default() -> Self {
    Self {
      cpus: 4,
      memory_in_bytes: GIB,
      interfaces: Vec::new(),
      mounts_by_id: BTreeMap::new(),
      boot_log: None,
      nested_virtualization: false,
    }
  }
}
