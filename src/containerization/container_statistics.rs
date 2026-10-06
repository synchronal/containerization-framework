//! `ContainerStatistics`, and the statistics nested in it.

/// `ContainerStatistics`. Made by [`super::LinuxContainer::statistics`],
/// with each category it wasn't asked for as `None`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContainerStatistics {
  pub id: String,
  pub process: Option<ProcessStatistics>,
  pub memory: Option<MemoryStatistics>,
  pub cpu: Option<CpuStatistics>,
  pub block_io: Option<BlockIoStatistics>,
  pub networks: Option<Vec<NetworkStatistics>>,
  pub memory_events: Option<MemoryEventStatistics>,
  pub filesystem: Option<Vec<FilesystemStatistics>>,
}

/// `ContainerStatistics.ProcessStatistics`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessStatistics {
  pub current: u64,
  pub limit: u64,
}

/// `ContainerStatistics.MemoryStatistics`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryStatistics {
  pub usage_bytes: u64,
  pub limit_bytes: u64,
  pub swap_usage_bytes: u64,
  pub swap_limit_bytes: u64,
  pub cache_bytes: u64,
  pub kernel_stack_bytes: u64,
  pub slab_bytes: u64,
  pub page_faults: u64,
  pub major_page_faults: u64,
  pub inactive_file: u64,
  pub anon: u64,
  pub workingset_refault_anon: u64,
  pub workingset_refault_file: u64,
  pub pgsteal_kswapd: u64,
  pub pgsteal_direct: u64,
  pub pgsteal_khugepaged: u64,
}

/// `ContainerStatistics.CPUStatistics`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CpuStatistics {
  pub usage_usec: u64,
  pub user_usec: u64,
  pub system_usec: u64,
  pub throttling_periods: u64,
  pub throttled_periods: u64,
  pub throttled_time_usec: u64,
}

/// `ContainerStatistics.BlockIOStatistics`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockIoStatistics {
  pub devices: Vec<BlockIoDevice>,
}

/// `ContainerStatistics.BlockIODevice`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockIoDevice {
  pub major: u64,
  pub minor: u64,
  pub read_bytes: u64,
  pub write_bytes: u64,
  pub read_operations: u64,
  pub write_operations: u64,
}

/// `ContainerStatistics.NetworkStatistics`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkStatistics {
  pub interface: String,
  pub received_packets: u64,
  pub transmitted_packets: u64,
  pub received_bytes: u64,
  pub transmitted_bytes: u64,
  pub received_errors: u64,
  pub transmitted_errors: u64,
}

/// `ContainerStatistics.MemoryEventStatistics`: cgroup2's `memory.events`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryEventStatistics {
  pub low: u64,
  pub high: u64,
  pub max: u64,
  pub oom: u64,
  pub oom_kill: u64,
}

/// `ContainerStatistics.FilesystemStatistics`: one mount's `statfs(2)`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilesystemStatistics {
  pub mount_point: String,
  pub block_size: u64,
  pub blocks: u64,
  pub free_blocks: u64,
  pub inodes: u64,
  pub free_inodes: u64,
}
