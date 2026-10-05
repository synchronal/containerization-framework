//! Getters and setters for the values images, content and unpacking take: the
//! OCI `Platform` and `Descriptor`, `Image.Description`, `EXT4Unpacker`, the
//! rootfs `create`'s options, and a `ProgressHandler`.

use super::entry_at;
use crate::bridge::ffi;
use crate::containerization;
use crate::containerization::container_manager;
use crate::containerization::image;
use crate::containerization_ext4::ext4::journal_config::JournalMode;
use crate::containerization_extras::ProgressEvent;
use crate::containerization_oci;
use crate::platform::Progress;
use std::collections::BTreeMap;

impl Progress {
  pub(crate) fn is_some(&self) -> bool {
    self.0.is_some()
  }

  /// One batch of events, as a kind and a value each.
  pub(crate) fn call(&self, kinds: Vec<ffi::ProgressKind>, values: Vec<i64>) {
    let Some(handler) = &self.0 else {
      return;
    };
    let events: Vec<ProgressEvent> = kinds
      .into_iter()
      .zip(values)
      .map(|(kind, value)| match kind {
        ffi::ProgressKind::Items => ProgressEvent::AddItems(value as isize),
        ffi::ProgressKind::TotalItems => ProgressEvent::AddTotalItems(value as isize),
        ffi::ProgressKind::Size => ProgressEvent::AddSize(value),
        ffi::ProgressKind::TotalSize => ProgressEvent::AddTotalSize(value),
      })
      .collect();

    handler(&events);
  }
}

impl containerization_oci::Platform {
  pub(crate) fn architecture(&self) -> &str {
    &self.architecture
  }

  pub(crate) fn os(&self) -> &str {
    &self.os
  }

  pub(crate) fn os_version(&self) -> Option<&str> {
    self.os_version.as_deref()
  }

  pub(crate) fn has_os_features(&self) -> bool {
    self.os_features.is_some()
  }

  pub(crate) fn os_features_len(&self) -> usize {
    self.os_features.as_ref().map_or(0, Vec::len)
  }

  pub(crate) fn os_features_at(&self, index: usize) -> &str {
    &self
      .os_features
      .as_ref()
      .expect("Swift asks for a feature only below the length")[index]
  }

  pub(crate) fn variant(&self) -> Option<&str> {
    self.variant.as_deref()
  }
}

impl ffi::CzOutcome {
  /// The `Platform` an outcome holds, read field by field.
  pub(crate) fn platform(&self) -> containerization_oci::Platform {
    containerization_oci::Platform {
      architecture: self.platform_architecture(),
      os: self.platform_os(),
      os_version: self.platform_os_version(),
      os_features: self
        .platform_has_os_features()
        .then(|| self.platform_os_features()),
      variant: self.platform_variant(),
    }
  }
}

impl containerization_oci::Descriptor {
  pub(crate) fn media_type(&self) -> &str {
    &self.media_type
  }

  pub(crate) fn digest(&self) -> &str {
    &self.digest
  }

  pub(crate) fn size(&self) -> i64 {
    self.size
  }

  pub(crate) fn has_urls(&self) -> bool {
    self.urls.is_some()
  }

  pub(crate) fn urls_len(&self) -> usize {
    self.urls.as_ref().map_or(0, Vec::len)
  }

  pub(crate) fn urls_at(&self, index: usize) -> &str {
    &self
      .urls
      .as_ref()
      .expect("Swift asks for a URL only below the length")[index]
  }

  pub(crate) fn has_annotations(&self) -> bool {
    self.annotations.is_some()
  }

  pub(crate) fn annotations_len(&self) -> usize {
    self
      .annotations
      .as_ref()
      .map_or(0, |annotations| annotations.len())
  }

  pub(crate) fn annotation_key_at(&self, index: usize) -> &str {
    entry_at(self.annotations(), index).0
  }

  pub(crate) fn annotation_value_at(&self, index: usize) -> &str {
    entry_at(self.annotations(), index).1
  }

  fn annotations(&self) -> &BTreeMap<String, String> {
    self
      .annotations
      .as_ref()
      .expect("Swift asks for an annotation only below the length")
  }

  pub(crate) fn has_platform(&self) -> bool {
    self.platform.is_some()
  }

  pub(crate) fn platform(&self) -> &containerization_oci::Platform {
    self
      .platform
      .as_ref()
      .expect("Swift asks for the platform only after `has_platform`")
  }

  pub(crate) fn artifact_type(&self) -> Option<&str> {
    self.artifact_type.as_deref()
  }
}

impl image::Description {
  pub(crate) fn reference(&self) -> &str {
    &self.reference
  }

  pub(crate) fn descriptor(&self) -> &containerization_oci::Descriptor {
    &self.descriptor
  }
}

impl containerization::Ext4Unpacker {
  pub(crate) fn capacity_in_bytes(&self) -> u64 {
    self.capacity_in_bytes
  }

  pub(crate) fn has_journal(&self) -> bool {
    self.journal.is_some()
  }

  pub(crate) fn journal_size(&self) -> Option<u64> {
    self.journal.and_then(|journal| journal.size)
  }

  pub(crate) fn has_journal_mode(&self) -> bool {
    self
      .journal
      .and_then(|journal| journal.default_mode)
      .is_some()
  }

  pub(crate) fn journal_mode(&self) -> ffi::JournalModeKind {
    match self
      .journal
      .and_then(|journal| journal.default_mode)
      .expect("Swift asks for the mode only after `has_journal_mode`")
    {
      JournalMode::Writeback => ffi::JournalModeKind::Writeback,
      JournalMode::Ordered => ffi::JournalModeKind::Ordered,
      JournalMode::Journal => ffi::JournalModeKind::Journal,
    }
  }
}

impl container_manager::RootfsCreateOptions {
  pub(crate) fn has_writable_layer(&self) -> bool {
    self.writable_layer.is_some()
  }

  pub(crate) fn writable_layer(&self) -> &containerization::Mount {
    self
      .writable_layer
      .as_ref()
      .expect("Swift asks for the writable layer only after `has_writable_layer`")
  }

  pub(crate) fn networking(&self) -> bool {
    self.networking
  }

  pub(crate) fn vm_cpus(&self) -> u32 {
    self.vm.cpus
  }

  pub(crate) fn vm_memory_in_bytes(&self) -> u64 {
    self.vm.memory_in_bytes
  }
}
