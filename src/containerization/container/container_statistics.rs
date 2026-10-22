//! `ContainerStatistics`, and the statistics nested in it.

/// `ContainerStatistics`. [`super::LinuxContainer::statistics`] makes it, and
/// leaves each category it wasn't asked for as `None`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContainerStatistics {
  pub id: String,
  pub process: Option<ProcessStatistics>,
  pub memory: Option<MemoryStatistics>,
  pub cpu: Option<CPUStatistics>,
  pub block_io: Option<BlockIOStatistics>,
  pub networks: Option<Vec<NetworkStatistics>>,
  pub memory_events: Option<MemoryEventStatistics>,
  pub filesystem: Option<Vec<FilesystemStatistics>>,
  /// Per-protocol traffic for the network namespace, which the network
  /// category reports. `None` if the guest doesn't report it.
  pub network_protocols: Option<NetworkProtocolStatistics>,
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
pub struct CPUStatistics {
  pub usage_usec: u64,
  pub user_usec: u64,
  pub system_usec: u64,
  pub throttling_periods: u64,
  pub throttled_periods: u64,
  pub throttled_time_usec: u64,
}

/// `ContainerStatistics.BlockIOStatistics`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockIOStatistics {
  pub devices: Vec<BlockIODevice>,
}

/// `ContainerStatistics.BlockIODevice`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockIODevice {
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
  pub received_errors: Option<u64>,
  pub transmitted_errors: Option<u64>,
  /// Packets the device dropped, for example because its receive ring or
  /// transmit queue was full.
  pub received_dropped: u64,
  pub transmitted_dropped: u64,
}

/// `ContainerStatistics.NetworkProtocolCounters`, the packet and byte counters
/// for one protocol. [`Default`] uses Swift's initializer's defaults:
/// zero packets, and every other counter `None`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NetworkProtocolCounters {
  pub received_packets: u64,
  pub transmitted_packets: u64,
  /// `None` when the protocol has no byte counter. Linux only has one for IP
  /// as a whole.
  pub received_bytes: Option<u64>,
  pub transmitted_bytes: Option<u64>,
  /// Packets the stack discarded for lack of resources or a destination. The
  /// kernel attributes drops to different causes for each protocol, so they
  /// can be compared over time for one protocol but not across protocols.
  /// `None` when the protocol has no matching counter, and zero when none
  /// happened.
  pub received_drops: Option<u64>,
  pub transmitted_drops: Option<u64>,
  /// Packets that were malformed or failed processing. As with drops, the
  /// causes differ for each protocol.
  pub received_errors: Option<u64>,
  pub transmitted_errors: Option<u64>,
}

/// `ContainerStatistics.NetworkProtocolStatistics`, the per-protocol traffic
/// for a container's network namespace. It covers the whole namespace,
/// including loopback, not one interface, so containers that share a
/// namespace report the same values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkProtocolStatistics {
  /// Both IPv4 and IPv6 traffic, because the kernel shares its TCP counters
  /// between the two families.
  pub tcp: NetworkProtocolCounters,
  /// IPv4 only. [`Self::udp6`] counts IPv6.
  pub udp: NetworkProtocolCounters,
  /// IPv4 only. [`Self::icmp6`] counts IPv6.
  pub icmp: NetworkProtocolCounters,
  /// All IPv4 traffic, including what TCP, UDP and ICMP don't count, such as
  /// raw IP. [`Self::ip6`] counts IPv6.
  pub ip: NetworkProtocolCounters,
  /// IPv6 UDP. `None` when the guest has no IPv6 statistics.
  pub udp6: Option<NetworkProtocolCounters>,
  /// IPv6 ICMP. `None` when the guest has no IPv6 statistics.
  pub icmp6: Option<NetworkProtocolCounters>,
  /// All IPv6 traffic. `None` when the guest has no IPv6 statistics.
  pub ip6: Option<NetworkProtocolCounters>,
}

/// `ContainerStatistics.MemoryEventStatistics`, the counters in cgroup2's
/// `memory.events`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryEventStatistics {
  pub low: u64,
  pub high: u64,
  pub max: u64,
  pub oom: u64,
  pub oom_kill: u64,
}

/// `ContainerStatistics.FilesystemStatistics`, the `statfs(2)` figures for
/// one mount.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilesystemStatistics {
  pub mount_point: String,
  pub block_size: u64,
  pub blocks: u64,
  pub free_blocks: u64,
  pub inodes: u64,
  pub free_inodes: u64,
}
