//! The OCI runtime spec's cgroup resources: memory, CPU, block I/O, device
//! rules and the rest.

use super::nth;
use super::present;
use crate::bridge::accessors::entry_at;
use crate::bridge::ffi;
use crate::containerization_oci::runtime;
use std::collections::BTreeMap;

impl ffi::CzOutcome {
  pub(super) fn resources(&self) -> runtime::LinuxResources {
    runtime::LinuxResources {
      devices: self.resources_devices().list(Self::device_cgroup),
      memory: self.resources_memory().optional(Self::memory),
      cpu: self.resources_cpu().optional(Self::cpu),
      pids: self.resources_pids().optional(|pids| runtime::LinuxPids {
        limit: pids.pids_limit(),
      }),
      block_io: self.resources_block_io().optional(Self::block_io),
      hugepage_limits: self
        .resources_hugepage_limits()
        .list(|limit| runtime::LinuxHugepageLimit {
          pagesize: limit.hugepage_limit_pagesize(),
          limit: limit.hugepage_limit_limit(),
        }),
      network: self
        .resources_network()
        .optional(|network| runtime::LinuxNetwork {
          class_id: network.network_class_id(),
          priorities: network
            .network_priorities()
            .list(|priority| runtime::LinuxInterfacePriority {
              name: priority.interface_priority_name(),
              priority: priority.interface_priority_priority(),
            }),
        }),
      rdma: self.resources_rdma().optional(|rdma| {
        rdma.map_of(|rdma| runtime::LinuxRdma {
          hcs_handles: rdma.rdma_hcs_handles(),
          hca_objects: rdma.rdma_hca_objects(),
        })
      }),
      unified: self.resources_unified().optional(Self::map),
    }
  }

  pub(super) fn memory(&self) -> runtime::LinuxMemory {
    runtime::LinuxMemory {
      limit: self.memory_limit(),
      reservation: self.memory_reservation(),
      swap: self.memory_swap(),
      kernel: self.memory_kernel(),
      kernel_tcp: self.memory_kernel_tcp(),
      swappiness: self.memory_swappiness(),
      disable_oom_killer: self.memory_disable_oom_killer(),
      use_hierarchy: self.memory_use_hierarchy(),
      check_before_update: self.memory_check_before_update(),
    }
  }

  pub(super) fn cpu(&self) -> runtime::LinuxCPU {
    runtime::LinuxCPU {
      shares: self.cpu_shares(),
      quota: self.cpu_quota(),
      burst: self.cpu_burst(),
      period: self.cpu_period(),
      realtime_runtime: self.cpu_realtime_runtime(),
      realtime_period: self.cpu_realtime_period(),
      cpus: self.cpu_cpus(),
      mems: self.cpu_mems(),
      idle: self.cpu_idle(),
    }
  }

  fn block_io(&self) -> runtime::LinuxBlockIO {
    let throttle = |kind| {
      self
        .block_io_throttle(kind)
        .list(|device| runtime::LinuxThrottleDevice {
          major: device.throttle_device_major(),
          minor: device.throttle_device_minor(),
          rate: device.throttle_device_rate(),
        })
    };

    runtime::LinuxBlockIO {
      weight: self.block_io_weight(),
      leaf_weight: self.block_io_leaf_weight(),
      weight_device: self
        .block_io_weight_device()
        .list(|device| runtime::LinuxWeightDevice {
          major: device.weight_device_major(),
          minor: device.weight_device_minor(),
          weight: device.weight_device_weight(),
          leaf_weight: device.weight_device_leaf_weight(),
        }),
      throttle_read_bps_device: throttle(ffi::ThrottleKind::ReadBps),
      throttle_write_bps_device: throttle(ffi::ThrottleKind::WriteBps),
      throttle_read_iops_device: throttle(ffi::ThrottleKind::ReadIops),
      throttle_write_iops_device: throttle(ffi::ThrottleKind::WriteIops),
    }
  }

  fn device_cgroup(&self) -> runtime::LinuxDeviceCgroup {
    runtime::LinuxDeviceCgroup {
      allow: self.device_cgroup_allow(),
      r#type: self.device_cgroup_type(),
      major: self.device_cgroup_major(),
      minor: self.device_cgroup_minor(),
      access: self.device_cgroup_access(),
    }
  }
}

impl runtime::LinuxResources {
  pub(crate) fn devices_len(&self) -> usize {
    self.devices.len()
  }

  pub(crate) fn devices_at(&self, index: usize) -> &runtime::LinuxDeviceCgroup {
    &self.devices[index]
  }

  pub(crate) fn has_memory(&self) -> bool {
    self.memory.is_some()
  }

  pub(crate) fn memory(&self) -> &runtime::LinuxMemory {
    present(&self.memory)
  }

  pub(crate) fn has_cpu(&self) -> bool {
    self.cpu.is_some()
  }

  pub(crate) fn cpu(&self) -> &runtime::LinuxCPU {
    present(&self.cpu)
  }

  pub(crate) fn pids_limit(&self) -> Option<i64> {
    self.pids.map(|pids| pids.limit)
  }

  pub(crate) fn has_block_io(&self) -> bool {
    self.block_io.is_some()
  }

  pub(crate) fn block_io(&self) -> &runtime::LinuxBlockIO {
    present(&self.block_io)
  }

  pub(crate) fn hugepage_limits_len(&self) -> usize {
    self.hugepage_limits.len()
  }

  pub(crate) fn hugepage_limit_pagesize_at(&self, index: usize) -> &str {
    &self.hugepage_limits[index].pagesize
  }

  pub(crate) fn hugepage_limit_limit_at(&self, index: usize) -> u64 {
    self.hugepage_limits[index].limit
  }

  pub(crate) fn has_network(&self) -> bool {
    self.network.is_some()
  }

  pub(crate) fn network_class_id(&self) -> Option<u32> {
    present(&self.network).class_id
  }

  pub(crate) fn network_priorities_len(&self) -> usize {
    present(&self.network).priorities.len()
  }

  pub(crate) fn network_priority_name_at(&self, index: usize) -> &str {
    &present(&self.network).priorities[index].name
  }

  pub(crate) fn network_priority_at(&self, index: usize) -> u32 {
    present(&self.network).priorities[index].priority
  }

  pub(crate) fn has_rdma(&self) -> bool {
    self.rdma.is_some()
  }

  pub(crate) fn rdma_len(&self) -> usize {
    self.rdma.as_ref().map_or(0, BTreeMap::len)
  }

  pub(crate) fn rdma_key_at(&self, index: usize) -> &str {
    nth(present(&self.rdma), index).0
  }

  pub(crate) fn rdma_hcs_handles_at(&self, index: usize) -> Option<u32> {
    nth(present(&self.rdma), index).1.hcs_handles
  }

  pub(crate) fn rdma_hca_objects_at(&self, index: usize) -> Option<u32> {
    nth(present(&self.rdma), index).1.hca_objects
  }

  pub(crate) fn has_unified(&self) -> bool {
    self.unified.is_some()
  }

  pub(crate) fn unified_len(&self) -> usize {
    self.unified.as_ref().map_or(0, BTreeMap::len)
  }

  pub(crate) fn unified_key_at(&self, index: usize) -> &str {
    entry_at(present(&self.unified), index).0
  }

  pub(crate) fn unified_value_at(&self, index: usize) -> &str {
    entry_at(present(&self.unified), index).1
  }
}

impl runtime::LinuxMemory {
  pub(crate) fn limit(&self) -> Option<i64> {
    self.limit
  }

  pub(crate) fn reservation(&self) -> Option<i64> {
    self.reservation
  }

  pub(crate) fn swap(&self) -> Option<i64> {
    self.swap
  }

  pub(crate) fn kernel(&self) -> Option<i64> {
    self.kernel
  }

  pub(crate) fn kernel_tcp(&self) -> Option<i64> {
    self.kernel_tcp
  }

  pub(crate) fn swappiness(&self) -> Option<u64> {
    self.swappiness
  }

  pub(crate) fn disable_oom_killer(&self) -> Option<bool> {
    self.disable_oom_killer
  }

  pub(crate) fn use_hierarchy(&self) -> Option<bool> {
    self.use_hierarchy
  }

  pub(crate) fn check_before_update(&self) -> Option<bool> {
    self.check_before_update
  }
}

impl runtime::LinuxCPU {
  pub(crate) fn shares(&self) -> Option<u64> {
    self.shares
  }

  pub(crate) fn quota(&self) -> Option<i64> {
    self.quota
  }

  pub(crate) fn burst(&self) -> Option<u64> {
    self.burst
  }

  pub(crate) fn period(&self) -> Option<u64> {
    self.period
  }

  pub(crate) fn realtime_runtime(&self) -> Option<i64> {
    self.realtime_runtime
  }

  pub(crate) fn realtime_period(&self) -> Option<i64> {
    self.realtime_period
  }

  pub(crate) fn cpus(&self) -> &str {
    &self.cpus
  }

  pub(crate) fn mems(&self) -> &str {
    &self.mems
  }

  pub(crate) fn idle(&self) -> Option<i64> {
    self.idle
  }
}

impl runtime::LinuxBlockIO {
  pub(crate) fn weight(&self) -> Option<u16> {
    self.weight
  }

  pub(crate) fn leaf_weight(&self) -> Option<u16> {
    self.leaf_weight
  }

  pub(crate) fn weight_device_len(&self) -> usize {
    self.weight_device.len()
  }

  pub(crate) fn weight_device_major_at(&self, index: usize) -> i64 {
    self.weight_device[index].major
  }

  pub(crate) fn weight_device_minor_at(&self, index: usize) -> i64 {
    self.weight_device[index].minor
  }

  pub(crate) fn weight_device_weight_at(&self, index: usize) -> Option<u16> {
    self.weight_device[index].weight
  }

  pub(crate) fn weight_device_leaf_weight_at(&self, index: usize) -> Option<u16> {
    self.weight_device[index].leaf_weight
  }

  pub(crate) fn throttle_len(&self, kind: ffi::ThrottleKind) -> usize {
    self.throttle(kind).len()
  }

  pub(crate) fn throttle_major_at(&self, kind: ffi::ThrottleKind, index: usize) -> i64 {
    self.throttle(kind)[index].major
  }

  pub(crate) fn throttle_minor_at(&self, kind: ffi::ThrottleKind, index: usize) -> i64 {
    self.throttle(kind)[index].minor
  }

  pub(crate) fn throttle_rate_at(&self, kind: ffi::ThrottleKind, index: usize) -> u64 {
    self.throttle(kind)[index].rate
  }

  fn throttle(&self, kind: ffi::ThrottleKind) -> &[runtime::LinuxThrottleDevice] {
    match kind {
      ffi::ThrottleKind::ReadBps => &self.throttle_read_bps_device,
      ffi::ThrottleKind::WriteBps => &self.throttle_write_bps_device,
      ffi::ThrottleKind::ReadIops => &self.throttle_read_iops_device,
      ffi::ThrottleKind::WriteIops => &self.throttle_write_iops_device,
    }
  }
}

impl runtime::LinuxDeviceCgroup {
  pub(crate) fn allow(&self) -> bool {
    self.allow
  }

  pub(crate) fn device_type(&self) -> &str {
    &self.r#type
  }

  pub(crate) fn major(&self) -> Option<i64> {
    self.major
  }

  pub(crate) fn minor(&self) -> Option<i64> {
    self.minor
  }

  pub(crate) fn access(&self) -> Option<&str> {
    self.access.as_deref()
  }
}
