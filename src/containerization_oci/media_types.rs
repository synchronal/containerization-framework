/// `MediaTypes`: Swift's static strings, as associated constants.
#[derive(Clone, Copy, Debug)]
pub struct MediaTypes;

impl MediaTypes {
  /// `MediaTypes.descriptor`.
  pub const DESCRIPTOR: &str = "application/vnd.oci.descriptor.v1+json";
  /// `MediaTypes.layoutHeader`.
  pub const LAYOUT_HEADER: &str = "application/vnd.oci.layout.header.v1+json";
  /// `MediaTypes.index`.
  pub const INDEX: &str = "application/vnd.oci.image.index.v1+json";
  /// `MediaTypes.imageManifest`.
  pub const IMAGE_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
  /// `MediaTypes.imageConfig`.
  pub const IMAGE_CONFIG: &str = "application/vnd.oci.image.config.v1+json";
  /// `MediaTypes.emptyJSON`.
  pub const EMPTY_JSON: &str = "application/vnd.oci.empty.v1+json";
  /// `MediaTypes.dockerManifest`.
  pub const DOCKER_MANIFEST: &str = "application/vnd.docker.distribution.manifest.v2+json";
  /// `MediaTypes.dockerManifestList`.
  pub const DOCKER_MANIFEST_LIST: &str = "application/vnd.docker.distribution.manifest.list.v2+json";
  /// `MediaTypes.dockerImageConfig`.
  pub const DOCKER_IMAGE_CONFIG: &str = "application/vnd.docker.container.image.v1+json";
  /// `MediaTypes.imageLayer`.
  pub const IMAGE_LAYER: &str = "application/vnd.oci.image.layer.v1.tar";
  /// `MediaTypes.imageLayerGzip`.
  pub const IMAGE_LAYER_GZIP: &str = "application/vnd.oci.image.layer.v1.tar+gzip";
  /// `MediaTypes.imageLayerZstd`.
  pub const IMAGE_LAYER_ZSTD: &str = "application/vnd.oci.image.layer.v1.tar+zstd";
  /// `MediaTypes.dockerImageLayer`.
  pub const DOCKER_IMAGE_LAYER: &str = "application/vnd.docker.image.rootfs.diff.tar";
  /// `MediaTypes.dockerImageLayerGzip`.
  pub const DOCKER_IMAGE_LAYER_GZIP: &str = "application/vnd.docker.image.rootfs.diff.tar.gzip";
  /// `MediaTypes.dockerImageLayerZstd`.
  pub const DOCKER_IMAGE_LAYER_ZSTD: &str = "application/vnd.docker.image.rootfs.diff.tar.zstd";
  /// `MediaTypes.inTotoAttestationBlob`.
  pub const IN_TOTO_ATTESTATION_BLOB: &str = "application/vnd.in-toto+json";

  /// Every constant, in the order `mediaTypes()` in the bridge lists Swift's.
  #[cfg(all(test, target_os = "macos"))]
  pub(crate) const ALL: [&str; 16] = [
    Self::DESCRIPTOR,
    Self::LAYOUT_HEADER,
    Self::INDEX,
    Self::IMAGE_MANIFEST,
    Self::IMAGE_CONFIG,
    Self::EMPTY_JSON,
    Self::DOCKER_MANIFEST,
    Self::DOCKER_MANIFEST_LIST,
    Self::DOCKER_IMAGE_CONFIG,
    Self::IMAGE_LAYER,
    Self::IMAGE_LAYER_GZIP,
    Self::IMAGE_LAYER_ZSTD,
    Self::DOCKER_IMAGE_LAYER,
    Self::DOCKER_IMAGE_LAYER_GZIP,
    Self::DOCKER_IMAGE_LAYER_ZSTD,
    Self::IN_TOTO_ATTESTATION_BLOB,
  ];
}
