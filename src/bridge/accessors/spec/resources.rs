//! The OCI runtime spec's cgroup resources: memory, CPU, block I/O, device
//! rules and the rest.

use crate::bridge::accessors::entry_at;
use crate::bridge::accessors::present;
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
  getters! {
    list devices: runtime::LinuxDeviceCgroup => devices_len, devices_at;
    present memory: runtime::LinuxMemory => has_memory;
    present cpu: runtime::LinuxCPU => has_cpu;
  }

  pub(crate) fn pids_limit(&self) -> Option<i64> {
    self.pids.map(|pids| pids.limit)
  }

  getters! {
    present block_io: runtime::LinuxBlockIO => has_block_io;
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
    entry_at(present(&self.rdma), index).0
  }

  pub(crate) fn rdma_hcs_handles_at(&self, index: usize) -> Option<u32> {
    entry_at(present(&self.rdma), index).1.hcs_handles
  }

  pub(crate) fn rdma_hca_objects_at(&self, index: usize) -> Option<u32> {
    entry_at(present(&self.rdma), index).1.hca_objects
  }

  getters! {
    optional_map unified => has_unified, unified_len, unified_key_at, unified_value_at;
  }
}

impl runtime::LinuxMemory {
  getters! {
    copy limit: Option<i64>;
    copy reservation: Option<i64>;
    copy swap: Option<i64>;
    copy kernel: Option<i64>;
    copy kernel_tcp: Option<i64>;
    copy swappiness: Option<u64>;
    copy disable_oom_killer: Option<bool>;
    copy use_hierarchy: Option<bool>;
    copy check_before_update: Option<bool>;
  }
}

impl runtime::LinuxCPU {
  getters! {
    copy shares: Option<u64>;
    copy quota: Option<i64>;
    copy burst: Option<u64>;
    copy period: Option<u64>;
    copy realtime_runtime: Option<i64>;
    copy realtime_period: Option<i64>;
    str cpus;
    str mems;
    copy idle: Option<i64>;
  }
}

impl runtime::LinuxBlockIO {
  getters! {
    copy weight: Option<u16>;
    copy leaf_weight: Option<u16>;
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
  getters! {
    copy allow: bool;
    str r#type => device_type;
    copy major: Option<i64>;
    copy minor: Option<i64>;
    optional_str access;
  }
}
