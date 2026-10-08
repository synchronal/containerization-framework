//! The OCI runtime spec's Linux section: its namespaces, ID mappings, devices
//! and personality.

use crate::bridge::accessors::entry_at;
use crate::bridge::accessors::present;
use crate::bridge::ffi;
use crate::containerization_oci::runtime;
use std::collections::BTreeMap;

impl ffi::CzOutcome {
  pub(super) fn linux(&self) -> runtime::Linux {
    runtime::Linux {
      uid_mappings: self.linux_uid_mappings().list(Self::id_mapping),
      gid_mappings: self.linux_gid_mappings().list(Self::id_mapping),
      sysctl: self.linux_sysctl().optional(Self::map),
      resources: self.linux_resources().optional(Self::resources),
      cgroups_path: self.linux_cgroups_path(),
      namespaces: self
        .linux_namespaces()
        .list(|namespace| runtime::LinuxNamespace {
          r#type: runtime::LinuxNamespaceType::from_swift(&namespace.namespace_type()),
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
        .optional(|personality| runtime::LinuxPersonality {
          domain: runtime::LinuxPersonalityDomain::from_swift(&personality.personality_domain()),
          flags: personality.personality_flags(),
        }),
    }
  }

  pub(super) fn id_mapping(&self) -> runtime::LinuxIDMapping {
    runtime::LinuxIDMapping {
      container_id: self.id_mapping_container_id(),
      host_id: self.id_mapping_host_id(),
      size: self.id_mapping_size(),
    }
  }

  fn device(&self) -> runtime::LinuxDevice {
    runtime::LinuxDevice {
      path: self.device_path(),
      r#type: self.device_type(),
      major: self.device_major(),
      minor: self.device_minor(),
      file_mode: self.device_file_mode(),
      uid: self.device_uid(),
      gid: self.device_gid(),
    }
  }
}

impl runtime::LinuxIDMapping {
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

impl runtime::Linux {
  pub(crate) fn uid_mappings_len(&self) -> usize {
    self.uid_mappings.len()
  }

  pub(crate) fn uid_mappings_at(&self, index: usize) -> &runtime::LinuxIDMapping {
    &self.uid_mappings[index]
  }

  pub(crate) fn gid_mappings_len(&self) -> usize {
    self.gid_mappings.len()
  }

  pub(crate) fn gid_mappings_at(&self, index: usize) -> &runtime::LinuxIDMapping {
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

  pub(crate) fn resources(&self) -> &runtime::LinuxResources {
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

  pub(crate) fn devices_at(&self, index: usize) -> &runtime::LinuxDevice {
    &self.devices[index]
  }

  pub(crate) fn has_seccomp(&self) -> bool {
    self.seccomp.is_some()
  }

  pub(crate) fn seccomp(&self) -> &runtime::LinuxSeccomp {
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

impl runtime::LinuxDevice {
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
