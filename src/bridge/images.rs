//! The OCI image values and the image types built from them, which Swift
//! reads.

use crate::containerization::image::Description as RustImageDescription;
use crate::containerization::image::EXT4Unpacker as RustExt4Unpacker;
use crate::containerization_oci::image::Descriptor as RustDescriptor;
use crate::containerization_oci::image::ImageConfig as RustImageConfig;
use crate::containerization_oci::image::Platform as RustPlatform;

use super::ffi::JournalModeKind;

#[swift_bridge::bridge]
mod ffi {
  #[swift_bridge(already_declared)]
  enum JournalModeKind {}

  extern "Rust" {
    type RustPlatform;
    fn architecture(self: &RustPlatform) -> &str;
    fn os(self: &RustPlatform) -> &str;
    #[swift_bridge(swift_name = "osVersion")]
    fn os_version(self: &RustPlatform) -> Option<&str>;
    #[swift_bridge(swift_name = "hasOsFeatures")]
    fn has_os_features(self: &RustPlatform) -> bool;
    #[swift_bridge(swift_name = "osFeaturesLen")]
    fn os_features_len(self: &RustPlatform) -> usize;
    #[swift_bridge(swift_name = "osFeaturesAt")]
    fn os_features_at(self: &RustPlatform, index: usize) -> &str;
    fn variant(self: &RustPlatform) -> Option<&str>;

    type RustDescriptor;
    #[swift_bridge(swift_name = "mediaType")]
    fn media_type(self: &RustDescriptor) -> &str;
    fn digest(self: &RustDescriptor) -> &str;
    fn size(self: &RustDescriptor) -> i64;
    #[swift_bridge(swift_name = "hasUrls")]
    fn has_urls(self: &RustDescriptor) -> bool;
    #[swift_bridge(swift_name = "urlsLen")]
    fn urls_len(self: &RustDescriptor) -> usize;
    #[swift_bridge(swift_name = "urlsAt")]
    fn urls_at(self: &RustDescriptor, index: usize) -> &str;
    #[swift_bridge(swift_name = "hasAnnotations")]
    fn has_annotations(self: &RustDescriptor) -> bool;
    #[swift_bridge(swift_name = "annotationsLen")]
    fn annotations_len(self: &RustDescriptor) -> usize;
    #[swift_bridge(swift_name = "annotationKeyAt")]
    fn annotation_key_at(self: &RustDescriptor, index: usize) -> &str;
    #[swift_bridge(swift_name = "annotationValueAt")]
    fn annotation_value_at(self: &RustDescriptor, index: usize) -> &str;
    #[swift_bridge(swift_name = "hasPlatform")]
    fn has_platform(self: &RustDescriptor) -> bool;
    fn platform(self: &RustDescriptor) -> &RustPlatform;
    #[swift_bridge(swift_name = "artifactType")]
    fn artifact_type(self: &RustDescriptor) -> Option<&str>;

    type RustImageDescription;
    fn reference(self: &RustImageDescription) -> &str;
    fn descriptor(self: &RustImageDescription) -> &RustDescriptor;

    type RustExt4Unpacker;
    #[swift_bridge(swift_name = "capacityInBytes")]
    fn capacity_in_bytes(self: &RustExt4Unpacker) -> u64;
    #[swift_bridge(swift_name = "hasJournal")]
    fn has_journal(self: &RustExt4Unpacker) -> bool;
    #[swift_bridge(swift_name = "journalSize")]
    fn journal_size(self: &RustExt4Unpacker) -> Option<u64>;
    #[swift_bridge(swift_name = "hasJournalMode")]
    fn has_journal_mode(self: &RustExt4Unpacker) -> bool;
    #[swift_bridge(swift_name = "journalMode")]
    fn journal_mode(self: &RustExt4Unpacker) -> JournalModeKind;

    type RustImageConfig;
    fn user(self: &RustImageConfig) -> Option<&str>;
    #[swift_bridge(swift_name = "hasEnv")]
    fn has_env(self: &RustImageConfig) -> bool;
    #[swift_bridge(swift_name = "envLen")]
    fn env_len(self: &RustImageConfig) -> usize;
    #[swift_bridge(swift_name = "envAt")]
    fn env_at(self: &RustImageConfig, index: usize) -> &str;
    #[swift_bridge(swift_name = "hasEntrypoint")]
    fn has_entrypoint(self: &RustImageConfig) -> bool;
    #[swift_bridge(swift_name = "entrypointLen")]
    fn entrypoint_len(self: &RustImageConfig) -> usize;
    #[swift_bridge(swift_name = "entrypointAt")]
    fn entrypoint_at(self: &RustImageConfig, index: usize) -> &str;
    #[swift_bridge(swift_name = "hasCmd")]
    fn has_cmd(self: &RustImageConfig) -> bool;
    #[swift_bridge(swift_name = "cmdLen")]
    fn cmd_len(self: &RustImageConfig) -> usize;
    #[swift_bridge(swift_name = "cmdAt")]
    fn cmd_at(self: &RustImageConfig, index: usize) -> &str;
    #[swift_bridge(swift_name = "workingDir")]
    fn working_dir(self: &RustImageConfig) -> Option<&str>;
    #[swift_bridge(swift_name = "hasLabels")]
    fn has_labels(self: &RustImageConfig) -> bool;
    #[swift_bridge(swift_name = "labelsLen")]
    fn labels_len(self: &RustImageConfig) -> usize;
    #[swift_bridge(swift_name = "labelKeyAt")]
    fn label_key_at(self: &RustImageConfig, index: usize) -> &str;
    #[swift_bridge(swift_name = "labelValueAt")]
    fn label_value_at(self: &RustImageConfig, index: usize) -> &str;
    #[swift_bridge(swift_name = "stopSignal")]
    fn stop_signal(self: &RustImageConfig) -> Option<&str>;
  }
}
