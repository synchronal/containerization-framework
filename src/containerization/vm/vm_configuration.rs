use super::BootLog;
use crate::containerization::GIB;
use crate::containerization::container::Mount;
use crate::containerization::network::Interface;
use std::collections::BTreeMap;

/// `VMConfiguration`, which [`super::VzVirtualMachineManager::create`] takes.
/// `StandardVMConfig` only wraps one, so Rust takes it directly. `extensions`
/// isn't bound.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VmConfiguration {
  pub cpus: u32,
  pub memory_in_bytes: u64,
  pub interfaces: Vec<Interface>,
  /// Mounts by the ID of what they're for, such as a container's.
  pub mounts_by_id: BTreeMap<String, Vec<Mount>>,
  pub boot_log: Option<BootLog>,
  pub nested_virtualization: bool,
}

/// `VMConfiguration()`: 4 CPUs and 1024 MiB.
impl Default for VmConfiguration {
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
