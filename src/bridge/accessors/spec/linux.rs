//! The OCI runtime spec's Linux section: its namespaces, ID mappings, devices
//! and personality.

use crate::bridge::accessors::present;
use crate::bridge::ffi;
use crate::containerization_oci::runtime;

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
  getters! {
    copy container_id: u32;
    copy host_id: u32;
    copy size: u32;
  }
}

impl runtime::Linux {
  getters! {
    list uid_mappings: runtime::LinuxIDMapping => uid_mappings_len, uid_mappings_at;
    list gid_mappings: runtime::LinuxIDMapping => gid_mappings_len, gid_mappings_at;
    optional_map sysctl => has_sysctl, sysctl_len, sysctl_key_at, sysctl_value_at;
    present resources: runtime::LinuxResources => has_resources;
    str cgroups_path;
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

  getters! {
    list devices: runtime::LinuxDevice => devices_len, devices_at;
    present seccomp: runtime::LinuxSeccomp => has_seccomp;
    str rootfs_propagation;
    list masked_paths: str => masked_paths_len, masked_paths_at;
    list readonly_paths: str => readonly_paths_len, readonly_paths_at;
    str mount_label;
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
  getters! {
    str path;
    str r#type => device_type;
    copy major: i64;
    copy minor: i64;
    copy file_mode: Option<u32>;
    copy uid: Option<u32>;
    copy gid: Option<u32>;
  }
}
