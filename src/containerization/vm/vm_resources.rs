use crate::containerization::GIB;
use crate::containerization::MIB;

/// The VM a container runs in. `VMResources`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VMResources {
  pub cpus: u32,
  pub memory_in_bytes: u64,
}

impl VMResources {
  /// `VMResources.guestMemoryOverhead`.
  pub const GUEST_MEMORY_OVERHEAD: u64 = 128 * MIB;
}

/// `VMResources.default`: 4 CPUs and 1024 MiB.
impl Default for VMResources {
  fn default() -> Self {
    Self {
      cpus: 4,
      memory_in_bytes: GIB,
    }
  }
}
