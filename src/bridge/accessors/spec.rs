//! The OCI runtime spec's types, both ways: Rust reads Swift's field by field
//! from an outcome, and Swift reads Rust's through these getters.
//!
//! A small type that only one other holds, like `Box` in `Process` or
//! `LinuxSeccompArg` in `LinuxSyscall`, is read through its holder's getters.
//! An enum crosses as its `rawValue`.

use super::entry_at;
use crate::bridge::ffi;
use crate::containerization_oci;
use std::collections::BTreeMap;

impl ffi::CzOutcome {
  /// What an outcome holds as text, or `None` for `Absent`.
  pub(crate) fn optional_text(&self) -> Option<String> {
    self.optional(Self::text)
  }

  /// A held `[String: T]`, each value read by `read`.
  pub(super) fn map_of<T>(&self, read: impl Fn(&Self) -> T) -> BTreeMap<String, T> {
    self
      .entry_keys()
      .into_iter()
      .zip(self.entry_values().list(read))
      .collect()
  }

  /// The `Spec` an outcome holds, read field by field.
  pub(crate) fn spec(&self) -> containerization_oci::Spec {
    containerization_oci::Spec {
      version: self.spec_version(),
      hooks: self.spec_hooks().optional(Self::hooks),
      process: self.spec_process().optional(Self::process),
      hostname: self.spec_hostname(),
      domainname: self.spec_domainname(),
      mounts: self.spec_mounts().list(Self::oci_mount),
      annotations: self.spec_annotations().optional(Self::map),
      root: self.spec_root().optional(Self::root),
      linux: self.spec_linux().optional(Self::linux),
    }
  }

  pub(crate) fn process(&self) -> containerization_oci::Process {
    containerization_oci::Process {
      cwd: self.process_cwd(),
      env: self.process_env(),
      console_size: self
        .process_console_size()
        .optional(|size| containerization_oci::Box::new(size.box_height(), size.box_width())),
      selinux_label: self.process_selinux_label(),
      no_new_privileges: self.process_no_new_privileges(),
      command_line: self.process_command_line(),
      oom_score_adj: self.process_oom_score_adj(),
      capabilities: self
        .process_capabilities()
        .optional(Self::oci_linux_capabilities),
      apparmor_profile: self.process_apparmor_profile(),
      user: self.process_user().user(),
      rlimits: self.process_rlimits().list(Self::posix_rlimit),
      args: self.process_args(),
      terminal: self.process_terminal(),
    }
  }

  fn user(&self) -> containerization_oci::User {
    containerization_oci::User {
      uid: self.user_uid(),
      gid: self.user_gid(),
      umask: self.user_umask(),
      additional_gids: self.user_additional_gids(),
      username: self.user_username(),
    }
  }

  pub(crate) fn oci_linux_capabilities(&self) -> containerization_oci::LinuxCapabilities {
    let set = |set| self.capabilities_set(set).optional(Self::strings);

    containerization_oci::LinuxCapabilities {
      bounding: set(ffi::CapabilitySet::Bounding),
      effective: set(ffi::CapabilitySet::Effective),
      inheritable: set(ffi::CapabilitySet::Inheritable),
      permitted: set(ffi::CapabilitySet::Permitted),
      ambient: set(ffi::CapabilitySet::Ambient),
    }
  }

  fn root(&self) -> containerization_oci::Root {
    containerization_oci::Root {
      path: self.root_path(),
      readonly: self.root_readonly(),
    }
  }

  fn oci_mount(&self) -> containerization_oci::Mount {
    containerization_oci::Mount {
      r#type: self.oci_mount_type(),
      source: self.oci_mount_source(),
      destination: self.oci_mount_destination(),
      options: self.oci_mount_options(),
      uid_mappings: self
        .oci_mount_uid_mappings()
        .optional(|mappings| mappings.list(Self::id_mapping)),
      gid_mappings: self
        .oci_mount_gid_mappings()
        .optional(|mappings| mappings.list(Self::id_mapping)),
    }
  }

  fn hook(&self) -> containerization_oci::Hook {
    containerization_oci::Hook {
      path: self.hook_path(),
      args: self.hook_args(),
      env: self.hook_env(),
      timeout: self.hook_timeout(),
    }
  }

  fn hooks(&self) -> containerization_oci::Hooks {
    let hooks = |kind| self.hooks_of(kind).list(Self::hook);

    containerization_oci::Hooks {
      prestart: hooks(ffi::HookKind::Prestart),
      create_runtime: hooks(ffi::HookKind::CreateRuntime),
      create_container: hooks(ffi::HookKind::CreateContainer),
      start_container: hooks(ffi::HookKind::StartContainer),
      poststart: hooks(ffi::HookKind::Poststart),
      poststop: hooks(ffi::HookKind::Poststop),
    }
  }

  fn linux(&self) -> containerization_oci::Linux {
    containerization_oci::Linux {
      uid_mappings: self.linux_uid_mappings().list(Self::id_mapping),
      gid_mappings: self.linux_gid_mappings().list(Self::id_mapping),
      sysctl: self.linux_sysctl().optional(Self::map),
      resources: self.linux_resources().optional(Self::resources),
      cgroups_path: self.linux_cgroups_path(),
      namespaces: self
        .linux_namespaces()
        .list(|namespace| containerization_oci::LinuxNamespace {
          r#type: containerization_oci::LinuxNamespaceType::from_swift(&namespace.namespace_type()),
          path: namespace.namespace_path(),
        }),
      devices: self.linux_devices().list(Self::device),
      seccomp: self.linux_seccomp().optional(Self::seccomp),
      rootfs_propagation: self.linux_rootfs_propagation(),
      masked_paths: self.linux_masked_paths(),
      readonly_paths: self.linux_readonly_paths(),
      mount_label: self.linux_mount_label(),
      personality: self
        .linux_personality()
        .optional(|personality| containerization_oci::LinuxPersonality {
          domain: containerization_oci::LinuxPersonalityDomain::from_swift(&personality.personality_domain()),
          flags: personality.personality_flags(),
        }),
    }
  }

  fn id_mapping(&self) -> containerization_oci::LinuxIDMapping {
    containerization_oci::LinuxIDMapping {
      container_id: self.id_mapping_container_id(),
      host_id: self.id_mapping_host_id(),
      size: self.id_mapping_size(),
    }
  }

  pub(crate) fn posix_rlimit(&self) -> containerization_oci::POSIXRlimit {
    containerization_oci::POSIXRlimit {
      r#type: self.rlimit_type(),
      hard: self.rlimit_hard(),
      soft: self.rlimit_soft(),
    }
  }

  fn resources(&self) -> containerization_oci::LinuxResources {
    containerization_oci::LinuxResources {
      devices: self.resources_devices().list(Self::device_cgroup),
      memory: self.resources_memory().optional(Self::memory),
      cpu: self.resources_cpu().optional(Self::cpu),
      pids: self
        .resources_pids()
        .optional(|pids| containerization_oci::LinuxPids {
          limit: pids.pids_limit(),
        }),
      block_io: self.resources_block_io().optional(Self::block_io),
      hugepage_limits: self
        .resources_hugepage_limits()
        .list(|limit| containerization_oci::LinuxHugepageLimit {
          pagesize: limit.hugepage_limit_pagesize(),
          limit: limit.hugepage_limit_limit(),
        }),
      network: self
        .resources_network()
        .optional(|network| containerization_oci::LinuxNetwork {
          class_id: network.network_class_id(),
          priorities: network
            .network_priorities()
            .list(|priority| containerization_oci::LinuxInterfacePriority {
              name: priority.interface_priority_name(),
              priority: priority.interface_priority_priority(),
            }),
        }),
      rdma: self.resources_rdma().optional(|rdma| {
        rdma.map_of(|rdma| containerization_oci::LinuxRdma {
          hcs_handles: rdma.rdma_hcs_handles(),
          hca_objects: rdma.rdma_hca_objects(),
        })
      }),
      unified: self.resources_unified().optional(Self::map),
    }
  }

  fn memory(&self) -> containerization_oci::LinuxMemory {
    containerization_oci::LinuxMemory {
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

  fn cpu(&self) -> containerization_oci::LinuxCPU {
    containerization_oci::LinuxCPU {
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

  fn block_io(&self) -> containerization_oci::LinuxBlockIO {
    let throttle = |kind| {
      self
        .block_io_throttle(kind)
        .list(|device| containerization_oci::LinuxThrottleDevice {
          major: device.throttle_device_major(),
          minor: device.throttle_device_minor(),
          rate: device.throttle_device_rate(),
        })
    };

    containerization_oci::LinuxBlockIO {
      weight: self.block_io_weight(),
      leaf_weight: self.block_io_leaf_weight(),
      weight_device: self
        .block_io_weight_device()
        .list(|device| containerization_oci::LinuxWeightDevice {
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

  fn device(&self) -> containerization_oci::LinuxDevice {
    containerization_oci::LinuxDevice {
      path: self.device_path(),
      r#type: self.device_type(),
      major: self.device_major(),
      minor: self.device_minor(),
      file_mode: self.device_file_mode(),
      uid: self.device_uid(),
      gid: self.device_gid(),
    }
  }

  fn device_cgroup(&self) -> containerization_oci::LinuxDeviceCgroup {
    containerization_oci::LinuxDeviceCgroup {
      allow: self.device_cgroup_allow(),
      r#type: self.device_cgroup_type(),
      major: self.device_cgroup_major(),
      minor: self.device_cgroup_minor(),
      access: self.device_cgroup_access(),
    }
  }

  /// The `LinuxSeccomp` an outcome holds, read field by field.
  pub(crate) fn seccomp(&self) -> containerization_oci::LinuxSeccomp {
    containerization_oci::LinuxSeccomp {
      default_action: containerization_oci::LinuxSeccompAction::from_swift(&self.seccomp_default_action()),
      default_errno_ret: self.seccomp_default_errno_ret(),
      architectures: self
        .seccomp_architectures()
        .iter()
        .map(|arch| containerization_oci::Arch::from_swift(arch))
        .collect(),
      flags: self
        .seccomp_flags()
        .iter()
        .map(|flag| containerization_oci::LinuxSeccompFlag::from_swift(flag))
        .collect(),
      listener_path: self.seccomp_listener_path(),
      listener_metadata: self.seccomp_listener_metadata(),
      syscalls: self.seccomp_syscalls().list(Self::syscall),
    }
  }

  fn syscall(&self) -> containerization_oci::LinuxSyscall {
    containerization_oci::LinuxSyscall {
      names: self.syscall_names(),
      action: containerization_oci::LinuxSeccompAction::from_swift(&self.syscall_action()),
      errno_ret: self.syscall_errno_ret(),
      args: self
        .syscall_args()
        .list(|arg| containerization_oci::LinuxSeccompArg {
          index: arg.seccomp_arg_index(),
          value: arg.seccomp_arg_value(),
          value_two: arg.seccomp_arg_value_two(),
          op: containerization_oci::LinuxSeccompOperator::from_swift(&arg.seccomp_arg_op()),
        }),
    }
  }

  pub(crate) fn runtime_spec_version(&self) -> containerization_oci::RuntimeSpecVersion {
    containerization_oci::RuntimeSpecVersion {
      major: self.runtime_spec_version_major(),
      minor: self.runtime_spec_version_minor(),
      patch: self.runtime_spec_version_patch(),
      dev: self.runtime_spec_version_dev(),
    }
  }
}

/// What an optional field holds. Swift asks for it only after `has_`.
fn present<T>(value: &Option<T>) -> &T {
  value
    .as_ref()
    .expect("Swift asks for an optional field only after its `has_` getter")
}

/// The `index`th entry of a sorted map of values other than strings.
fn nth<T>(map: &BTreeMap<String, T>, index: usize) -> (&str, &T) {
  map
    .iter()
    .nth(index)
    .map(|(key, value)| (key.as_str(), value))
    .expect("Swift asks for an entry only below the length")
}

impl containerization_oci::Spec {
  pub(crate) fn version(&self) -> &str {
    &self.version
  }

  pub(crate) fn has_hooks(&self) -> bool {
    self.hooks.is_some()
  }

  pub(crate) fn hooks(&self) -> &containerization_oci::Hooks {
    present(&self.hooks)
  }

  pub(crate) fn has_process(&self) -> bool {
    self.process.is_some()
  }

  pub(crate) fn process(&self) -> &containerization_oci::Process {
    present(&self.process)
  }

  pub(crate) fn hostname(&self) -> &str {
    &self.hostname
  }

  pub(crate) fn domainname(&self) -> &str {
    &self.domainname
  }

  pub(crate) fn mounts_len(&self) -> usize {
    self.mounts.len()
  }

  pub(crate) fn mounts_at(&self, index: usize) -> &containerization_oci::Mount {
    &self.mounts[index]
  }

  pub(crate) fn has_annotations(&self) -> bool {
    self.annotations.is_some()
  }

  pub(crate) fn annotations_len(&self) -> usize {
    self.annotations.as_ref().map_or(0, BTreeMap::len)
  }

  pub(crate) fn annotation_key_at(&self, index: usize) -> &str {
    entry_at(present(&self.annotations), index).0
  }

  pub(crate) fn annotation_value_at(&self, index: usize) -> &str {
    entry_at(present(&self.annotations), index).1
  }

  pub(crate) fn has_root(&self) -> bool {
    self.root.is_some()
  }

  pub(crate) fn root_path(&self) -> &str {
    &present(&self.root).path
  }

  pub(crate) fn root_readonly(&self) -> bool {
    present(&self.root).readonly
  }

  pub(crate) fn has_linux(&self) -> bool {
    self.linux.is_some()
  }

  pub(crate) fn linux(&self) -> &containerization_oci::Linux {
    present(&self.linux)
  }
}

impl containerization_oci::Process {
  pub(crate) fn cwd(&self) -> &str {
    &self.cwd
  }

  pub(crate) fn env_len(&self) -> usize {
    self.env.len()
  }

  pub(crate) fn env_at(&self, index: usize) -> &str {
    &self.env[index]
  }

  pub(crate) fn has_console_size(&self) -> bool {
    self.console_size.is_some()
  }

  pub(crate) fn console_height(&self) -> usize {
    present(&self.console_size).height
  }

  pub(crate) fn console_width(&self) -> usize {
    present(&self.console_size).width
  }

  pub(crate) fn selinux_label(&self) -> &str {
    &self.selinux_label
  }

  pub(crate) fn no_new_privileges(&self) -> bool {
    self.no_new_privileges
  }

  pub(crate) fn command_line(&self) -> &str {
    &self.command_line
  }

  pub(crate) fn oom_score_adj(&self) -> Option<isize> {
    self.oom_score_adj
  }

  pub(crate) fn has_capabilities(&self) -> bool {
    self.capabilities.is_some()
  }

  pub(crate) fn capabilities(&self) -> &containerization_oci::LinuxCapabilities {
    present(&self.capabilities)
  }

  pub(crate) fn apparmor_profile(&self) -> &str {
    &self.apparmor_profile
  }

  pub(crate) fn user(&self) -> &containerization_oci::User {
    &self.user
  }

  pub(crate) fn rlimits_len(&self) -> usize {
    self.rlimits.len()
  }

  pub(crate) fn rlimit_type_at(&self, index: usize) -> &str {
    &self.rlimits[index].r#type
  }

  pub(crate) fn rlimit_hard_at(&self, index: usize) -> u64 {
    self.rlimits[index].hard
  }

  pub(crate) fn rlimit_soft_at(&self, index: usize) -> u64 {
    self.rlimits[index].soft
  }

  pub(crate) fn args_len(&self) -> usize {
    self.args.len()
  }

  pub(crate) fn args_at(&self, index: usize) -> &str {
    &self.args[index]
  }

  pub(crate) fn terminal(&self) -> bool {
    self.terminal
  }
}

impl containerization_oci::LinuxCapabilities {
  pub(crate) fn has_set(&self, set: ffi::CapabilitySet) -> bool {
    self.set(set).is_some()
  }

  pub(crate) fn set_len(&self, set: ffi::CapabilitySet) -> usize {
    self.set(set).as_ref().map_or(0, Vec::len)
  }

  pub(crate) fn set_at(&self, set: ffi::CapabilitySet, index: usize) -> &str {
    &present(self.set(set))[index]
  }

  fn set(&self, set: ffi::CapabilitySet) -> &Option<Vec<String>> {
    match set {
      ffi::CapabilitySet::Bounding => &self.bounding,
      ffi::CapabilitySet::Effective => &self.effective,
      ffi::CapabilitySet::Inheritable => &self.inheritable,
      ffi::CapabilitySet::Permitted => &self.permitted,
      ffi::CapabilitySet::Ambient => &self.ambient,
    }
  }
}

impl containerization_oci::Mount {
  pub(crate) fn mount_type(&self) -> &str {
    &self.r#type
  }

  pub(crate) fn source(&self) -> &str {
    &self.source
  }

  pub(crate) fn destination(&self) -> &str {
    &self.destination
  }

  pub(crate) fn options_len(&self) -> usize {
    self.options.len()
  }

  pub(crate) fn options_at(&self, index: usize) -> &str {
    &self.options[index]
  }

  pub(crate) fn has_uid_mappings(&self) -> bool {
    self.uid_mappings.is_some()
  }

  pub(crate) fn uid_mappings_len(&self) -> usize {
    self.uid_mappings.as_ref().map_or(0, Vec::len)
  }

  pub(crate) fn uid_mappings_at(&self, index: usize) -> &containerization_oci::LinuxIDMapping {
    &present(&self.uid_mappings)[index]
  }

  pub(crate) fn has_gid_mappings(&self) -> bool {
    self.gid_mappings.is_some()
  }

  pub(crate) fn gid_mappings_len(&self) -> usize {
    self.gid_mappings.as_ref().map_or(0, Vec::len)
  }

  pub(crate) fn gid_mappings_at(&self, index: usize) -> &containerization_oci::LinuxIDMapping {
    &present(&self.gid_mappings)[index]
  }
}

impl containerization_oci::LinuxIDMapping {
  pub(crate) fn container_id(&self) -> u32 {
    self.container_id
  }

  pub(crate) fn host_id(&self) -> u32 {
    self.host_id
  }

  pub(crate) fn size(&self) -> u32 {
    self.size
  }
}

impl containerization_oci::Hook {
  pub(crate) fn path(&self) -> &str {
    &self.path
  }

  pub(crate) fn args_len(&self) -> usize {
    self.args.len()
  }

  pub(crate) fn args_at(&self, index: usize) -> &str {
    &self.args[index]
  }

  pub(crate) fn env_len(&self) -> usize {
    self.env.len()
  }

  pub(crate) fn env_at(&self, index: usize) -> &str {
    &self.env[index]
  }

  pub(crate) fn timeout(&self) -> Option<isize> {
    self.timeout
  }
}

impl containerization_oci::Hooks {
  pub(crate) fn hooks_len(&self, kind: ffi::HookKind) -> usize {
    self.of(kind).len()
  }

  pub(crate) fn hooks_at(&self, kind: ffi::HookKind, index: usize) -> &containerization_oci::Hook {
    &self.of(kind)[index]
  }

  fn of(&self, kind: ffi::HookKind) -> &[containerization_oci::Hook] {
    match kind {
      ffi::HookKind::Prestart => &self.prestart,
      ffi::HookKind::CreateRuntime => &self.create_runtime,
      ffi::HookKind::CreateContainer => &self.create_container,
      ffi::HookKind::StartContainer => &self.start_container,
      ffi::HookKind::Poststart => &self.poststart,
      ffi::HookKind::Poststop => &self.poststop,
    }
  }
}

impl containerization_oci::Linux {
  pub(crate) fn uid_mappings_len(&self) -> usize {
    self.uid_mappings.len()
  }

  pub(crate) fn uid_mappings_at(&self, index: usize) -> &containerization_oci::LinuxIDMapping {
    &self.uid_mappings[index]
  }

  pub(crate) fn gid_mappings_len(&self) -> usize {
    self.gid_mappings.len()
  }

  pub(crate) fn gid_mappings_at(&self, index: usize) -> &containerization_oci::LinuxIDMapping {
    &self.gid_mappings[index]
  }

  pub(crate) fn has_sysctl(&self) -> bool {
    self.sysctl.is_some()
  }

  pub(crate) fn sysctl_len(&self) -> usize {
    self.sysctl.as_ref().map_or(0, BTreeMap::len)
  }

  pub(crate) fn sysctl_key_at(&self, index: usize) -> &str {
    entry_at(present(&self.sysctl), index).0
  }

  pub(crate) fn sysctl_value_at(&self, index: usize) -> &str {
    entry_at(present(&self.sysctl), index).1
  }

  pub(crate) fn has_resources(&self) -> bool {
    self.resources.is_some()
  }

  pub(crate) fn resources(&self) -> &containerization_oci::LinuxResources {
    present(&self.resources)
  }

  pub(crate) fn cgroups_path(&self) -> &str {
    &self.cgroups_path
  }

  pub(crate) fn namespaces_len(&self) -> usize {
    self.namespaces.len()
  }

  pub(crate) fn namespace_type_at(&self, index: usize) -> &str {
    self.namespaces[index].r#type.raw_value()
  }

  pub(crate) fn namespace_path_at(&self, index: usize) -> &str {
    &self.namespaces[index].path
  }

  pub(crate) fn devices_len(&self) -> usize {
    self.devices.len()
  }

  pub(crate) fn devices_at(&self, index: usize) -> &containerization_oci::LinuxDevice {
    &self.devices[index]
  }

  pub(crate) fn has_seccomp(&self) -> bool {
    self.seccomp.is_some()
  }

  pub(crate) fn seccomp(&self) -> &containerization_oci::LinuxSeccomp {
    present(&self.seccomp)
  }

  pub(crate) fn rootfs_propagation(&self) -> &str {
    &self.rootfs_propagation
  }

  pub(crate) fn masked_paths_len(&self) -> usize {
    self.masked_paths.len()
  }

  pub(crate) fn masked_paths_at(&self, index: usize) -> &str {
    &self.masked_paths[index]
  }

  pub(crate) fn readonly_paths_len(&self) -> usize {
    self.readonly_paths.len()
  }

  pub(crate) fn readonly_paths_at(&self, index: usize) -> &str {
    &self.readonly_paths[index]
  }

  pub(crate) fn mount_label(&self) -> &str {
    &self.mount_label
  }

  pub(crate) fn has_personality(&self) -> bool {
    self.personality.is_some()
  }

  pub(crate) fn personality_domain(&self) -> &str {
    present(&self.personality).domain.raw_value()
  }

  pub(crate) fn personality_flags_len(&self) -> usize {
    present(&self.personality).flags.len()
  }

  pub(crate) fn personality_flags_at(&self, index: usize) -> &str {
    &present(&self.personality).flags[index]
  }
}

impl containerization_oci::LinuxResources {
  pub(crate) fn devices_len(&self) -> usize {
    self.devices.len()
  }

  pub(crate) fn devices_at(&self, index: usize) -> &containerization_oci::LinuxDeviceCgroup {
    &self.devices[index]
  }

  pub(crate) fn has_memory(&self) -> bool {
    self.memory.is_some()
  }

  pub(crate) fn memory(&self) -> &containerization_oci::LinuxMemory {
    present(&self.memory)
  }

  pub(crate) fn has_cpu(&self) -> bool {
    self.cpu.is_some()
  }

  pub(crate) fn cpu(&self) -> &containerization_oci::LinuxCPU {
    present(&self.cpu)
  }

  pub(crate) fn pids_limit(&self) -> Option<i64> {
    self.pids.map(|pids| pids.limit)
  }

  pub(crate) fn has_block_io(&self) -> bool {
    self.block_io.is_some()
  }

  pub(crate) fn block_io(&self) -> &containerization_oci::LinuxBlockIO {
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

impl containerization_oci::LinuxMemory {
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

impl containerization_oci::LinuxCPU {
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

impl containerization_oci::LinuxBlockIO {
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

  fn throttle(&self, kind: ffi::ThrottleKind) -> &[containerization_oci::LinuxThrottleDevice] {
    match kind {
      ffi::ThrottleKind::ReadBps => &self.throttle_read_bps_device,
      ffi::ThrottleKind::WriteBps => &self.throttle_write_bps_device,
      ffi::ThrottleKind::ReadIops => &self.throttle_read_iops_device,
      ffi::ThrottleKind::WriteIops => &self.throttle_write_iops_device,
    }
  }
}

impl containerization_oci::LinuxDevice {
  pub(crate) fn path(&self) -> &str {
    &self.path
  }

  pub(crate) fn device_type(&self) -> &str {
    &self.r#type
  }

  pub(crate) fn major(&self) -> i64 {
    self.major
  }

  pub(crate) fn minor(&self) -> i64 {
    self.minor
  }

  pub(crate) fn file_mode(&self) -> Option<u32> {
    self.file_mode
  }

  pub(crate) fn uid(&self) -> Option<u32> {
    self.uid
  }

  pub(crate) fn gid(&self) -> Option<u32> {
    self.gid
  }
}

impl containerization_oci::LinuxDeviceCgroup {
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

impl containerization_oci::LinuxSeccomp {
  pub(crate) fn default_action(&self) -> &str {
    self.default_action.raw_value()
  }

  pub(crate) fn default_errno_ret(&self) -> Option<usize> {
    self.default_errno_ret
  }

  pub(crate) fn architectures_len(&self) -> usize {
    self.architectures.len()
  }

  pub(crate) fn architectures_at(&self, index: usize) -> &str {
    self.architectures[index].raw_value()
  }

  pub(crate) fn flags_len(&self) -> usize {
    self.flags.len()
  }

  pub(crate) fn flags_at(&self, index: usize) -> &str {
    self.flags[index].raw_value()
  }

  pub(crate) fn listener_path(&self) -> &str {
    &self.listener_path
  }

  pub(crate) fn listener_metadata(&self) -> &str {
    &self.listener_metadata
  }

  pub(crate) fn syscalls_len(&self) -> usize {
    self.syscalls.len()
  }

  pub(crate) fn syscalls_at(&self, index: usize) -> &containerization_oci::LinuxSyscall {
    &self.syscalls[index]
  }
}

impl containerization_oci::LinuxSyscall {
  pub(crate) fn names_len(&self) -> usize {
    self.names.len()
  }

  pub(crate) fn names_at(&self, index: usize) -> &str {
    &self.names[index]
  }

  pub(crate) fn action(&self) -> &str {
    self.action.raw_value()
  }

  pub(crate) fn errno_ret(&self) -> Option<usize> {
    self.errno_ret
  }

  pub(crate) fn args_len(&self) -> usize {
    self.args.len()
  }

  pub(crate) fn arg_index_at(&self, index: usize) -> usize {
    self.args[index].index
  }

  pub(crate) fn arg_value_at(&self, index: usize) -> u64 {
    self.args[index].value
  }

  pub(crate) fn arg_value_two_at(&self, index: usize) -> u64 {
    self.args[index].value_two
  }

  pub(crate) fn arg_op_at(&self, index: usize) -> &str {
    self.args[index].op.raw_value()
  }
}

impl containerization_oci::ImageConfig {
  pub(crate) fn user(&self) -> Option<&str> {
    self.user.as_deref()
  }

  pub(crate) fn has_env(&self) -> bool {
    self.env.is_some()
  }

  pub(crate) fn env_len(&self) -> usize {
    self.env.as_ref().map_or(0, Vec::len)
  }

  pub(crate) fn env_at(&self, index: usize) -> &str {
    &present(&self.env)[index]
  }

  pub(crate) fn has_entrypoint(&self) -> bool {
    self.entrypoint.is_some()
  }

  pub(crate) fn entrypoint_len(&self) -> usize {
    self.entrypoint.as_ref().map_or(0, Vec::len)
  }

  pub(crate) fn entrypoint_at(&self, index: usize) -> &str {
    &present(&self.entrypoint)[index]
  }

  pub(crate) fn has_cmd(&self) -> bool {
    self.cmd.is_some()
  }

  pub(crate) fn cmd_len(&self) -> usize {
    self.cmd.as_ref().map_or(0, Vec::len)
  }

  pub(crate) fn cmd_at(&self, index: usize) -> &str {
    &present(&self.cmd)[index]
  }

  pub(crate) fn working_dir(&self) -> Option<&str> {
    self.working_dir.as_deref()
  }

  pub(crate) fn has_labels(&self) -> bool {
    self.labels.is_some()
  }

  pub(crate) fn labels_len(&self) -> usize {
    self.labels.as_ref().map_or(0, BTreeMap::len)
  }

  pub(crate) fn label_key_at(&self, index: usize) -> &str {
    entry_at(present(&self.labels), index).0
  }

  pub(crate) fn label_value_at(&self, index: usize) -> &str {
    entry_at(present(&self.labels), index).1
  }

  pub(crate) fn stop_signal(&self) -> Option<&str> {
    self.stop_signal.as_deref()
  }
}

#[cfg(test)]
mod tests {
  use crate::bridge::ffi;
  use crate::containerization_oci;

  /// Each enum's raw values, in Rust's order, against Swift's.
  #[test]
  fn copies_swifts_raw_values() {
    fn raw_values<T: Copy>(all: &[T], raw_value: fn(T) -> &'static str) -> Vec<&'static str> {
      all.iter().map(|case| raw_value(*case)).collect()
    }

    let copies = [
      (
        "LinuxNamespaceType",
        raw_values(
          containerization_oci::LinuxNamespaceType::ALL,
          containerization_oci::LinuxNamespaceType::raw_value,
        ),
      ),
      (
        "LinuxPersonalityDomain",
        raw_values(
          containerization_oci::LinuxPersonalityDomain::ALL,
          containerization_oci::LinuxPersonalityDomain::raw_value,
        ),
      ),
      (
        "LinuxSeccompFlag",
        raw_values(
          containerization_oci::LinuxSeccompFlag::ALL,
          containerization_oci::LinuxSeccompFlag::raw_value,
        ),
      ),
      (
        "Arch",
        raw_values(containerization_oci::Arch::ALL, containerization_oci::Arch::raw_value),
      ),
      (
        "LinuxSeccompAction",
        raw_values(
          containerization_oci::LinuxSeccompAction::ALL,
          containerization_oci::LinuxSeccompAction::raw_value,
        ),
      ),
      (
        "LinuxSeccompOperator",
        raw_values(
          containerization_oci::LinuxSeccompOperator::ALL,
          containerization_oci::LinuxSeccompOperator::raw_value,
        ),
      ),
      (
        "ContainerState",
        raw_values(
          containerization_oci::ContainerState::ALL,
          containerization_oci::ContainerState::raw_value,
        ),
      ),
    ];

    for (name, copy) in copies {
      assert_eq!(ffi::cz_raw_values(name), copy, "{name}");
    }
  }

  #[test]
  fn copies_swifts_seccomp_fd_name() {
    assert_eq!(ffi::cz_seccomp_fd_name(), containerization_oci::SECCOMP_FD_NAME);
  }

  /// Each `Default` against Swift's initializer with no arguments.
  #[test]
  fn defaults_as_swift_does() {
    assert_eq!(ffi::cz_default("Spec").spec(), containerization_oci::Spec::default());
    assert_eq!(
      ffi::cz_default("Process").process(),
      containerization_oci::Process::default()
    );
    assert_eq!(
      ffi::cz_default("LinuxCapabilities").oci_linux_capabilities(),
      containerization_oci::LinuxCapabilities::default()
    );
    assert_eq!(ffi::cz_default("Linux").linux(), containerization_oci::Linux::default());
    assert_eq!(
      ffi::cz_default("LinuxResources").resources(),
      containerization_oci::LinuxResources::default()
    );
    assert_eq!(
      ffi::cz_default("LinuxMemory").memory(),
      containerization_oci::LinuxMemory::default()
    );
    assert_eq!(
      ffi::cz_default("LinuxCPU").cpu(),
      containerization_oci::LinuxCPU::default()
    );
  }
}
