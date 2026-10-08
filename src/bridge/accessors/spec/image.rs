//! The OCI image's configuration.

use crate::bridge::accessors::entry_at;
use crate::bridge::accessors::present;
use crate::containerization_oci::image;
use std::collections::BTreeMap;

impl image::ImageConfig {
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
