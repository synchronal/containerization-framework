//! Getters and setters for the values images, content and unpacking take: the
//! OCI `Platform` and `Descriptor`, `Image.Description`, `EXT4Unpacker`, the
//! rootfs `create`'s options, and a `ProgressHandler`.

use super::entry_at;
use super::ext4;
use super::present;
use crate::bridge::ffi;
use crate::containerization;
use crate::containerization::container::container_manager;
use crate::containerization::image;
use crate::containerization::vm::kernel;
use crate::containerization_extras;
use crate::containerization_oci;
use crate::platform;

impl platform::Progress {
  pub(crate) fn is_some(&self) -> bool {
    self.0.is_some()
  }

  /// One batch of events, as a kind and a value each.
  pub(crate) fn call(&self, kinds: Vec<ffi::ProgressKind>, values: Vec<i64>) {
    let Some(handler) = &self.0 else {
      return;
    };
    let events: Vec<containerization_extras::ProgressEvent> = kinds
      .into_iter()
      .zip(values)
      .map(|(kind, value)| match kind {
        ffi::ProgressKind::Items => containerization_extras::ProgressEvent::AddItems(value as isize),
        ffi::ProgressKind::TotalItems => containerization_extras::ProgressEvent::AddTotalItems(value as isize),
        ffi::ProgressKind::Size => containerization_extras::ProgressEvent::AddSize(value),
        ffi::ProgressKind::TotalSize => containerization_extras::ProgressEvent::AddTotalSize(value),
      })
      .collect();

    handler(&events);
  }
}

impl containerization_oci::image::Platform {
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
    &present(&self.os_features)[index]
  }

  pub(crate) fn variant(&self) -> Option<&str> {
    self.variant.as_deref()
  }
}

impl ffi::CzOutcome {
  /// The `Platform` an outcome holds, read field by field.
  pub(crate) fn platform(&self) -> containerization_oci::image::Platform {
    containerization_oci::image::Platform {
      architecture: self.platform_architecture(),
      os: self.platform_os(),
      os_version: self.platform_os_version(),
      os_features: self
        .platform_has_os_features()
        .then(|| self.platform_os_features()),
      variant: self.platform_variant(),
    }
  }

  /// The `Descriptor` an outcome holds, read field by field.
  pub(crate) fn descriptor(&self) -> containerization_oci::image::Descriptor {
    containerization_oci::image::Descriptor {
      media_type: self.descriptor_media_type(),
      digest: self.descriptor_digest(),
      size: self.descriptor_size(),
      urls: self.descriptor_urls().optional(Self::strings),
      annotations: self.descriptor_annotations().optional(Self::map),
      platform: self.descriptor_platform().optional(Self::platform),
      artifact_type: self.descriptor_artifact_type(),
    }
  }

  pub(crate) fn index(&self) -> containerization_oci::image::Index {
    containerization_oci::image::Index {
      schema_version: self.index_schema_version(),
      media_type: self.index_media_type(),
      manifests: self.index_manifests().list(Self::descriptor),
      annotations: self.index_annotations().optional(Self::map),
      subject: self.index_subject().optional(Self::descriptor),
      artifact_type: self.index_artifact_type(),
    }
  }

  pub(crate) fn manifest(&self) -> containerization_oci::image::Manifest {
    containerization_oci::image::Manifest {
      schema_version: self.manifest_schema_version(),
      media_type: self.manifest_media_type(),
      config: self.manifest_config().descriptor(),
      layers: self.manifest_layers().list(Self::descriptor),
      annotations: self.manifest_annotations().optional(Self::map),
      subject: self.manifest_subject().optional(Self::descriptor),
      artifact_type: self.manifest_artifact_type(),
    }
  }

  fn image_config(&self) -> containerization_oci::image::ImageConfig {
    containerization_oci::image::ImageConfig {
      user: self.image_config_user(),
      env: self.image_config_env().optional(Self::strings),
      entrypoint: self.image_config_entrypoint().optional(Self::strings),
      cmd: self.image_config_cmd().optional(Self::strings),
      working_dir: self.image_config_working_dir(),
      labels: self.image_config_labels().optional(Self::map),
      stop_signal: self.image_config_stop_signal(),
    }
  }

  fn rootfs(&self) -> containerization_oci::image::Rootfs {
    containerization_oci::image::Rootfs {
      r#type: self.rootfs_type(),
      diff_ids: self.rootfs_diff_ids(),
    }
  }

  fn history(&self) -> containerization_oci::image::History {
    containerization_oci::image::History {
      created: self.history_created(),
      created_by: self.history_created_by(),
      author: self.history_author(),
      comment: self.history_comment(),
      empty_layer: self.history_empty_layer(),
    }
  }

  /// The `ContainerizationOCI.Image` an outcome holds.
  pub(crate) fn oci_image(&self) -> containerization_oci::image::Image {
    containerization_oci::image::Image {
      created: self.oci_image_created(),
      author: self.oci_image_author(),
      architecture: self.oci_image_architecture(),
      os: self.oci_image_os(),
      os_version: self.oci_image_os_version(),
      os_features: self.oci_image_os_features().optional(Self::strings),
      variant: self.oci_image_variant(),
      config: self.oci_image_config().optional(Self::image_config),
      rootfs: self.oci_image_rootfs().rootfs(),
      history: self
        .oci_image_history()
        .optional(|history| history.list(Self::history)),
    }
  }

  /// The `Kernel` an outcome holds.
  pub(crate) fn kernel(&self) -> containerization::vm::Kernel {
    containerization::vm::Kernel {
      path: self.kernel_path().into(),
      platform: containerization::vm::SystemPlatform {
        os: self.kernel_platform_os().into(),
        architecture: self.kernel_platform_architecture().into(),
      },
      command_line: kernel::CommandLine {
        kernel_args: self.kernel_kernel_args(),
        init_args: self.kernel_init_args(),
      },
    }
  }
}

impl containerization_oci::image::Descriptor {
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
    &present(&self.urls)[index]
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
    entry_at(present(&self.annotations), index).0
  }

  pub(crate) fn annotation_value_at(&self, index: usize) -> &str {
    entry_at(present(&self.annotations), index).1
  }

  pub(crate) fn has_platform(&self) -> bool {
    self.platform.is_some()
  }

  pub(crate) fn platform(&self) -> &containerization_oci::image::Platform {
    present(&self.platform)
  }

  pub(crate) fn artifact_type(&self) -> Option<&str> {
    self.artifact_type.as_deref()
  }
}

impl image::Description {
  pub(crate) fn reference(&self) -> &str {
    &self.reference
  }

  pub(crate) fn descriptor(&self) -> &containerization_oci::image::Descriptor {
    &self.descriptor
  }
}

impl image::EXT4Unpacker {
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
    ext4::has_journal_mode(self.journal)
  }

  pub(crate) fn journal_mode(&self) -> ffi::JournalModeKind {
    ext4::journal_mode(self.journal)
  }
}

impl container_manager::CreateWithRootfsOptions {
  pub(crate) fn has_writable_layer(&self) -> bool {
    self.writable_layer.is_some()
  }

  pub(crate) fn writable_layer(&self) -> &containerization::container::Mount {
    present(&self.writable_layer)
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

#[cfg(test)]
mod tests {
  use crate::bridge::ffi;
  use crate::containerization_oci;

  #[test]
  fn copies_swifts_media_types() {
    assert_eq!(ffi::cz_media_types(), containerization_oci::image::MediaTypes::ALL);
  }

  #[test]
  fn copies_swifts_digest_algorithm() {
    assert_eq!(
      ffi::cz_parsed_digest_algorithm(),
      containerization_oci::content::ParsedDigest::ALGORITHM
    );
  }

  #[test]
  fn copies_swifts_max_decoded_size() {
    assert_eq!(
      ffi::cz_local_content_max_decoded_size(),
      containerization_oci::content::Content::MAX_DECODED_SIZE
    );
  }

  #[test]
  fn copies_swifts_kernel_media_type() {
    assert_eq!(
      ffi::cz_kernel_image_media_type(),
      crate::containerization::image::KernelImage::MEDIA_TYPE
    );
  }

  #[test]
  fn copies_swifts_annotation_keys() {
    assert_eq!(
      ffi::cz_annotation_keys(),
      containerization_oci::image::AnnotationKeys::ALL
    );
  }
}
