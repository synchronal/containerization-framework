//! The OCI runtime spec's mounts.

use crate::bridge::ffi;
use crate::containerization_oci::runtime;

impl ffi::CzOutcome {
  pub(super) fn oci_mount(&self) -> runtime::Mount {
    runtime::Mount {
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
}

impl runtime::Mount {
  getters! {
    str r#type => mount_type;
    str source;
    str destination;
    list options: str => options_len, options_at;
    optional_list uid_mappings: runtime::LinuxIDMapping => has_uid_mappings, uid_mappings_len, uid_mappings_at;
    optional_list gid_mappings: runtime::LinuxIDMapping => has_gid_mappings, gid_mappings_len, gid_mappings_at;
  }
}
