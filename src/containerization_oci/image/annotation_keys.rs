/// `AnnotationKeys`: Swift's static strings, as associated constants.
#[derive(Clone, Copy, Debug)]
pub struct AnnotationKeys;

impl AnnotationKeys {
  /// `AnnotationKeys.containerizationIndexIndirect`.
  pub const CONTAINERIZATION_INDEX_INDIRECT: &str = "com.apple.containerization.index.indirect";
  /// `AnnotationKeys.containerizationImageName`.
  pub const CONTAINERIZATION_IMAGE_NAME: &str = "com.apple.containerization.image.name";
  /// `AnnotationKeys.containerdImageName`.
  pub const CONTAINERD_IMAGE_NAME: &str = "io.containerd.image.name";
  /// `AnnotationKeys.openContainersImageName`.
  pub const OPEN_CONTAINERS_IMAGE_NAME: &str = "org.opencontainers.image.ref.name";

  /// Every constant, in the order `annotationKeys()` in the bridge lists
  /// Swift's.
  #[cfg(all(test, target_os = "macos"))]
  pub(crate) const ALL: [&str; 4] = [
    Self::CONTAINERIZATION_INDEX_INDIRECT,
    Self::CONTAINERIZATION_IMAGE_NAME,
    Self::CONTAINERD_IMAGE_NAME,
    Self::OPEN_CONTAINERS_IMAGE_NAME,
  ];
}
