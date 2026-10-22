//! Getters and setters for the values images, content and unpacking take: the
//! OCI `Platform`, `Descriptor`, `Index` and `ImageConfig`,
//! `Image.Description` and `EXT4Unpacker`.

use super::ext4;
use crate::bridge::ffi;
use crate::containerization;
use crate::containerization::image;
use crate::containerization::vm::kernel;
use crate::containerization_oci;

impl containerization_oci::image::Platform {
  getters! {
    str architecture;
    str os;
    optional_str os_version;
    optional_list os_features: str => has_os_features, os_features_len, os_features_at;
    optional_str variant;
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

  /// The `[Platform]` an outcome holds.
  pub(crate) fn platforms(&self) -> Vec<containerization_oci::image::Platform> {
    self.list(Self::platform)
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
  getters! {
    str media_type;
    str digest;
    copy size: i64;
    optional_list urls: str => has_urls, urls_len, urls_at;
    optional_map annotations => has_annotations, annotations_len, annotation_key_at, annotation_value_at;
    present platform: containerization_oci::image::Platform => has_platform;
    optional_str artifact_type;
  }
}

impl containerization_oci::image::Index {
  getters! {
    copy schema_version: isize;
    str media_type;
    list manifests: containerization_oci::image::Descriptor => manifests_len, manifests_at;
    optional_map annotations => has_annotations, annotations_len, annotation_key_at, annotation_value_at;
    present subject: containerization_oci::image::Descriptor => has_subject;
    optional_str artifact_type;
  }
}

impl containerization_oci::image::ImageConfig {
  getters! {
    optional_str user;
    optional_list env: str => has_env, env_len, env_at;
    optional_list entrypoint: str => has_entrypoint, entrypoint_len, entrypoint_at;
    optional_list cmd: str => has_cmd, cmd_len, cmd_at;
    optional_str working_dir;
    optional_map labels => has_labels, labels_len, label_key_at, label_value_at;
    optional_str stop_signal;
  }
}

impl image::Description {
  getters! {
    str reference;
    borrow descriptor: containerization_oci::image::Descriptor;
  }
}

impl image::EXT4Unpacker {
  getters! {
    copy capacity_in_bytes: u64;
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
