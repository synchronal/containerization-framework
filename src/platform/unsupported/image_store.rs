//! Stand-ins for `ImageStore.swift`: `ImageStore`, `Image`, `InitImage` and
//! `KernelImage`.

use super::CzAuthentication;
use super::CzLocalContentStore;
use super::CzOutcome;
use crate::containerization::image;
use crate::containerization::vm;
use crate::containerization_oci;
use crate::platform;
use std::convert::Infallible;

taken!(
  image_store -> CzImageStore,
  image -> CzImage,
  images -> CzImages,
  init_image -> CzInitImage,
  kernel_image -> CzKernelImage,
  kernel -> vm::Kernel,
);

handles!(CzImageStore, CzImages, CzImage, CzInitImage, CzKernelImage);

failing!(cz_image_store_new(&str), cz_image_store_default());

// `ImageStore(path:contentStore:)` and `Image(description:contentStore:)`.
uninhabited!(CzLocalContentStore {
  fn image_store(&str) -> CzOutcome;
  fn image(image::Description) -> CzImage;
});

uninhabited!(CzImageStore {
  fn path() -> String;
  fn create_init_image(
    &str,
    &str,
    containerization_oci::image::Platform,
    Vec<String>,
    Vec<String>,
    CzLocalContentStore,
  ) -> CzOutcome;
  fn create_kernel_image(&str, Vec<vm::Kernel>, Vec<String>, Vec<String>, CzLocalContentStore) -> CzOutcome;
  fn get(&str, bool) -> CzOutcome;
  fn get_or_nil(&str) -> CzOutcome;
  fn list() -> CzOutcome;
  fn delete(&str, bool) -> CzOutcome;
  fn tag(&str, &str) -> CzOutcome;
  fn pull(
    &str,
    bool,
    containerization_oci::image::Platform,
    bool,
    CzAuthentication,
    platform::Progress,
    usize,
  ) -> CzOutcome;
  fn platforms(&str, bool, CzAuthentication) -> CzOutcome;
  fn push(
    &str,
    bool,
    containerization_oci::image::Platform,
    bool,
    CzAuthentication,
    platform::Progress,
  ) -> CzOutcome;
  fn push_all(
    Vec<String>,
    bool,
    containerization_oci::image::Platform,
    bool,
    CzAuthentication,
    usize,
    platform::Progress,
  ) -> CzOutcome;
  fn get_init_image(&str, CzAuthentication, platform::Progress) -> CzOutcome;
  fn save(Vec<String>, &str, bool, containerization_oci::image::Platform) -> CzOutcome;
  fn clean_up_orphaned_blobs() -> CzOutcome;
  fn calculate_orphaned_blobs_size() -> CzOutcome;
  fn create(image::Description) -> CzOutcome;
  fn load(&str, platform::Progress) -> CzOutcome;
});

uninhabited!(CzImages {
  fn len() -> usize;
  fn at(usize) -> CzImage;
});

uninhabited!(CzImage {
  fn duplicate() -> CzImage;
  fn init_image() -> CzInitImage;
  fn kernel_image() -> CzKernelImage;
  fn reference() -> String;
  fn digest() -> String;
  fn media_type() -> String;
  fn descriptor() -> CzOutcome;
  fn index() -> CzOutcome;
  fn manifest(containerization_oci::image::Platform) -> CzOutcome;
  fn descriptor_for(containerization_oci::image::Platform) -> CzOutcome;
  fn config(containerization_oci::image::Platform) -> CzOutcome;
  fn referenced_digests() -> CzOutcome;
  fn get_content(&str) -> CzOutcome;
});

uninhabited!(CzKernelImage {
  fn duplicate() -> CzKernelImage;
  fn name() -> String;
  fn kernel(vm::SystemPlatform) -> CzOutcome;
});

uninhabited!(CzInitImage {
  fn duplicate() -> CzInitImage;
  fn name() -> String;
  fn init_block(&str, vm::SystemPlatform) -> CzOutcome;
});
