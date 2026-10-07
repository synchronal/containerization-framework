//! The OCI runtime spec's mounts.

use super::present;
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

  pub(crate) fn uid_mappings_at(&self, index: usize) -> &runtime::LinuxIDMapping {
    &present(&self.uid_mappings)[index]
  }

  pub(crate) fn has_gid_mappings(&self) -> bool {
    self.gid_mappings.is_some()
  }

  pub(crate) fn gid_mappings_len(&self) -> usize {
    self.gid_mappings.as_ref().map_or(0, Vec::len)
  }

  pub(crate) fn gid_mappings_at(&self, index: usize) -> &runtime::LinuxIDMapping {
    &present(&self.gid_mappings)[index]
  }
}
