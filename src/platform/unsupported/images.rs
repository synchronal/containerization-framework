//! Images, content, registries, references and authentication.

use super::CzNetwork;
use super::CzOutcome;
use crate::containerization::container;
use crate::containerization::image;
use crate::containerization::vm;
use crate::containerization_oci;
use crate::platform;
use std::convert::Infallible;

taken!(
  local_content_store -> CzLocalContentStore,
  content -> CzContent,
  image_store -> CzImageStore,
  image -> CzImage,
  images -> CzImages,
  init_image -> CzInitImage,
  content_writer -> CzContentWriter,
  written_size -> i64,
  written_digest -> String,
  reference -> CzReference,
  authentication -> CzAuthentication,
  kernel_image -> CzKernelImage,
  ingest_session_id -> String,
  ingest_session_directory -> String,
  registry_client -> CzRegistryClient,
  parsed_digest_encoded -> String,
  descriptor -> containerization_oci::image::Descriptor,
  index -> containerization_oci::image::Index,
  manifest -> containerization_oci::image::Manifest,
  oci_image -> containerization_oci::image::Image,
);

handles!(
  CzLocalContentStore,
  CzContent,
  CzImageStore,
  CzImages,
  CzImage,
  CzInitImage,
  CzContentWriter,
  CzReference,
  CzAuthentication,
  CzKernelImage,
  CzRegistryClient,
);

failing!(
  cz_local_content_store_new(&str),
  cz_image_store_new(&str),
  cz_content_writer_new(&str),
  cz_registry_client_new(&str, bool, CzAuthentication),
  cz_registry_client_with_host(
    &str,
    Option<String>,
    Option<u16>,
    CzAuthentication,
    Option<String>,
    bool,
    isize,
    u64,
    usize,
  ),
  cz_content_writer_copy(&str, &str),
  cz_local_content_open(&str),
  cz_image_store_default(),
  cz_basic_authentication(&str, &str),
  cz_no_authentication(),
  cz_reference_new(&str, Option<String>, Option<String>, Option<String>),
  cz_reference_parse(&str),
  cz_reference_with_name(&str),
  cz_reference_resolve_domain(&str),
  cz_parsed_digest_parse(&str),
  cz_parsed_digest_parse_path_component(&str),
  cz_parsed_digest_is_valid(&str),
  cz_parsed_digest_description(&str),
  cz_parsed_digest_path(&str, &str),
);

pub(crate) fn cz_ext4_unpack(
  _unpacker: image::EXT4Unpacker,
  image: CzImage,
  _platform: containerization_oci::image::Platform,
  _at: &str,
  _progress: platform::Progress,
) -> CzOutcome {
  match image.0 {}
}

impl CzReference {
  pub(crate) fn domain(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn resolved_domain(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn path(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn tag(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn digest(&self) -> Option<String> {
    match self.0 {}
  }

  pub(crate) fn name(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn description(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn with_tag(&self, _tag: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn with_digest(&self, _digest: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn normalize(&self) {
    match self.0 {}
  }
}

impl CzRegistryClient {
  pub(crate) fn ping(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn resolve(&self, _name: &str, _tag: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn fetch_data(&self, _name: &str, _descriptor: containerization_oci::image::Descriptor) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn fetch_blob(
    &self,
    _name: &str,
    _descriptor: containerization_oci::image::Descriptor,
    _into: &str,
    _progress: platform::Progress,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn catalog(&self, _prefix: Option<String>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn referrers(&self, _name: &str, _digest: &str, _artifact_type: Option<String>) -> CzOutcome {
    match self.0 {}
  }
}

impl CzAuthentication {
  pub(crate) fn duplicate(&self) -> CzAuthentication {
    match self.0 {}
  }

  pub(crate) fn token(&self) -> CzOutcome {
    match self.0 {}
  }
}

impl CzLocalContentStore {
  pub(crate) fn get(&self, _digest: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn duplicate(&self) -> CzLocalContentStore {
    match self.0 {}
  }

  pub(crate) fn new_ingest_session(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn complete_ingest_session(&self, _id: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn cancel_ingest_session(&self, _id: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn image(&self, _description: image::Description) -> CzImage {
    match self.0 {}
  }

  pub(crate) fn delete_digests(&self, _digests: Vec<String>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn delete_keeping(&self, _keeping: Vec<String>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn total_allocated_size(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn image_store(&self, _path: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn ingest(&self, _body: Box<dyn FnOnce(String) -> bool>) -> CzOutcome {
    match self.0 {}
  }
}

impl CzContentWriter {
  pub(crate) fn write(&self, _data: Vec<u8>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn create(&self, _from: &str) -> CzOutcome {
    match self.0 {}
  }
}

impl CzContent {
  pub(crate) fn is_some(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn path(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn digest(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn size(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn data(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn data_range(&self, _offset: u64, _length: usize) -> CzOutcome {
    match self.0 {}
  }
}

impl CzImageStore {
  pub(crate) fn path(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn create_init_image(
    &self,
    _reference: &str,
    _rootfs: &str,
    _platform: containerization_oci::image::Platform,
    _label_keys: Vec<String>,
    _label_values: Vec<String>,
    _content_store: CzLocalContentStore,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn create_kernel_image(
    &self,
    _reference: &str,
    _binaries: Vec<vm::Kernel>,
    _label_keys: Vec<String>,
    _label_values: Vec<String>,
    _content_store: CzLocalContentStore,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn get(&self, _reference: &str, _pull: bool) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn list(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn delete(&self, _reference: &str, _perform_cleanup: bool) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn tag(&self, _existing: &str, _new: &str) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn pull(
    &self,
    _reference: &str,
    _has_platform: bool,
    _platform: containerization_oci::image::Platform,
    _insecure: bool,
    _auth: CzAuthentication,
    _progress: platform::Progress,
    _max_concurrent_downloads: usize,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn push(
    &self,
    _reference: &str,
    _has_platform: bool,
    _platform: containerization_oci::image::Platform,
    _insecure: bool,
    _auth: CzAuthentication,
    _progress: platform::Progress,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn push_all(
    &self,
    _references: Vec<String>,
    _has_platform: bool,
    _platform: containerization_oci::image::Platform,
    _insecure: bool,
    _auth: CzAuthentication,
    _max_concurrent_uploads: usize,
    _progress: platform::Progress,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn get_init_image(
    &self,
    _reference: &str,
    _auth: CzAuthentication,
    _progress: platform::Progress,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn save(
    &self,
    _references: Vec<String>,
    _out: &str,
    _has_platform: bool,
    _platform: containerization_oci::image::Platform,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn clean_up_orphaned_blobs(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn calculate_orphaned_blobs_size(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn create(&self, _description: image::Description) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn load(&self, _from: &str, _progress: platform::Progress) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn container_manager(
    &self,
    _kernel: vm::Kernel,
    _initfs: container::Mount,
    _network: CzNetwork,
    _rosetta: bool,
    _nested_virtualization: bool,
  ) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn container_manager_with_initfs_reference(
    &self,
    _kernel: vm::Kernel,
    _initfs_reference: &str,
    _network: CzNetwork,
    _rosetta: bool,
    _nested_virtualization: bool,
  ) -> CzOutcome {
    match self.0 {}
  }
}

impl CzImages {
  pub(crate) fn len(&self) -> usize {
    match self.0 {}
  }

  pub(crate) fn at(&self, _index: usize) -> CzImage {
    match self.0 {}
  }
}

impl CzImage {
  pub(crate) fn duplicate(&self) -> CzImage {
    match self.0 {}
  }

  pub(crate) fn init_image(&self) -> CzInitImage {
    match self.0 {}
  }

  pub(crate) fn kernel_image(&self) -> CzKernelImage {
    match self.0 {}
  }

  pub(crate) fn reference(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn digest(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn media_type(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn descriptor(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn index(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn manifest(&self, _platform: containerization_oci::image::Platform) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn descriptor_for(&self, _platform: containerization_oci::image::Platform) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn config(&self, _platform: containerization_oci::image::Platform) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn referenced_digests(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn get_content(&self, _digest: &str) -> CzOutcome {
    match self.0 {}
  }
}

impl CzKernelImage {
  pub(crate) fn name(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn kernel(&self, _platform: vm::SystemPlatform) -> CzOutcome {
    match self.0 {}
  }
}

impl CzInitImage {
  pub(crate) fn name(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn init_block(&self, _at: &str, _platform: vm::SystemPlatform) -> CzOutcome {
    match self.0 {}
  }
}
