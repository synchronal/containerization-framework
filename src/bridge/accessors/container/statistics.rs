//! Getters for the container statistics Swift reports.

use crate::bridge::ffi;
use crate::containerization::container;
use crate::containerization::container::container_statistics;

impl ffi::CzOutcome {
  /// The `ContainerStatistics` an outcome holds, each part read through the
  /// category that reports it.
  pub(crate) fn container_statistics(&self) -> container::ContainerStatistics {
    let part = |category: container::StatCategory| self.statistics_category(category.raw_value);

    container::ContainerStatistics {
      id: self.statistics_name(),
      process: part(container::StatCategory::PROCESS).optional(|process| {
        let [current, limit] = process.statistics_fields();
        container_statistics::ProcessStatistics { current, limit }
      }),
      memory: part(container::StatCategory::MEMORY).optional(|memory| {
        let [
          usage_bytes,
          limit_bytes,
          swap_usage_bytes,
          swap_limit_bytes,
          cache_bytes,
          kernel_stack_bytes,
          slab_bytes,
          page_faults,
          major_page_faults,
          inactive_file,
          anon,
          workingset_refault_anon,
          workingset_refault_file,
          pgsteal_kswapd,
          pgsteal_direct,
          pgsteal_khugepaged,
        ] = memory.statistics_fields();
        container_statistics::MemoryStatistics {
          usage_bytes,
          limit_bytes,
          swap_usage_bytes,
          swap_limit_bytes,
          cache_bytes,
          kernel_stack_bytes,
          slab_bytes,
          page_faults,
          major_page_faults,
          inactive_file,
          anon,
          workingset_refault_anon,
          workingset_refault_file,
          pgsteal_kswapd,
          pgsteal_direct,
          pgsteal_khugepaged,
        }
      }),
      cpu: part(container::StatCategory::CPU).optional(|cpu| {
        let [
          usage_usec,
          user_usec,
          system_usec,
          throttling_periods,
          throttled_periods,
          throttled_time_usec,
        ] = cpu.statistics_fields();
        container_statistics::CPUStatistics {
          usage_usec,
          user_usec,
          system_usec,
          throttling_periods,
          throttled_periods,
          throttled_time_usec,
        }
      }),
      block_io: part(container::StatCategory::BLOCK_IO).optional(|devices| container_statistics::BlockIOStatistics {
        devices: devices.list(|device| {
          let [major, minor, read_bytes, write_bytes, read_operations, write_operations] = device.statistics_fields();
          container_statistics::BlockIODevice {
            major,
            minor,
            read_bytes,
            write_bytes,
            read_operations,
            write_operations,
          }
        }),
      }),
      networks: part(container::StatCategory::NETWORK).optional(|networks| {
        networks.list(|network| {
          let [
            received_packets,
            transmitted_packets,
            received_bytes,
            transmitted_bytes,
            received_errors,
            transmitted_errors,
          ] = network.statistics_fields();
          container_statistics::NetworkStatistics {
            interface: network.statistics_name(),
            received_packets,
            transmitted_packets,
            received_bytes,
            transmitted_bytes,
            received_errors,
            transmitted_errors,
          }
        })
      }),
      memory_events: part(container::StatCategory::MEMORY_EVENTS).optional(|events| {
        let [low, high, max, oom, oom_kill] = events.statistics_fields();
        container_statistics::MemoryEventStatistics {
          low,
          high,
          max,
          oom,
          oom_kill,
        }
      }),
      filesystem: part(container::StatCategory::FILESYSTEM).optional(|filesystems| {
        filesystems.list(|filesystem| {
          let [block_size, blocks, free_blocks, inodes, free_inodes] = filesystem.statistics_fields();
          container_statistics::FilesystemStatistics {
            mount_point: filesystem.statistics_name(),
            block_size,
            blocks,
            free_blocks,
            inodes,
            free_inodes,
          }
        })
      }),
    }
  }

  /// A held `[ContainerStatistics]`.
  pub(crate) fn container_statistics_list(&self) -> Vec<container::ContainerStatistics> {
    self.list(Self::container_statistics)
  }

  /// The `UInt64` fields of the statistics held, which number `N`.
  fn statistics_fields<const N: usize>(&self) -> [u64; N] {
    let fields = self.statistics_numbers();
    let count = fields.len();

    fields
      .try_into()
      .unwrap_or_else(|_| panic!("Swift's statistics have {count} numbers, not {N}"))
  }
}
