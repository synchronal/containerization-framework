//===----------------------------------------------------------------------===//
// Rust's values for images, content and unpacking read into Containerization's,
// and the OCI image types read back by Rust field by field.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationEXT4
import ContainerizationExtras
import ContainerizationOCI
import Foundation

// MARK: Rust to Swift

extension Platform {
  init(_ platform: RustPlatformRef) {
    self.init(
      arch: platform.architecture().toString(),
      os: platform.os().toString(),
      osVersion: platform.osVersion()?.toString(),
      osFeatures: platform.hasOsFeatures() ? strings(platform.osFeaturesLen(), platform.osFeaturesAt) : nil,
      variant: platform.variant()?.toString()
    )
  }
}

extension Descriptor {
  init(_ descriptor: RustDescriptorRef) {
    self.init(
      mediaType: descriptor.mediaType().toString(),
      digest: descriptor.digest().toString(),
      size: descriptor.size(),
      urls: descriptor.hasUrls() ? strings(descriptor.urlsLen(), descriptor.urlsAt) : nil,
      annotations: descriptor.hasAnnotations()
        ? dictionary(descriptor.annotationsLen(), descriptor.annotationKeyAt, descriptor.annotationValueAt) : nil,
      platform: descriptor.hasPlatform() ? Platform(descriptor.platform()) : nil,
      artifactType: descriptor.artifactType()?.toString()
    )
  }
}

extension Index {
  init(_ index: RustIndexRef) {
    self.init(
      schemaVersion: index.schemaVersion(),
      mediaType: index.mediaType().toString(),
      manifests: (0..<index.manifestsLen()).map { Descriptor(index.manifestsAt($0)) },
      annotations: index.hasAnnotations()
        ? dictionary(index.annotationsLen(), index.annotationKeyAt, index.annotationValueAt) : nil,
      subject: index.hasSubject() ? Descriptor(index.subject()) : nil,
      artifactType: index.artifactType()?.toString()
    )
  }
}

extension ImageConfig {
  init(_ config: RustImageConfigRef) {
    self.init(
      user: config.user()?.toString(),
      env: config.hasEnv() ? strings(config.envLen(), config.envAt) : nil,
      entrypoint: config.hasEntrypoint() ? strings(config.entrypointLen(), config.entrypointAt) : nil,
      cmd: config.hasCmd() ? strings(config.cmdLen(), config.cmdAt) : nil,
      workingDir: config.workingDir()?.toString(),
      labels: config.hasLabels() ? dictionary(config.labelsLen(), config.labelKeyAt, config.labelValueAt) : nil,
      stopSignal: config.stopSignal()?.toString()
    )
  }
}

extension Containerization.Image.Description {
  init(_ description: RustImageDescription) {
    self.init(reference: description.reference().toString(), descriptor: Descriptor(description.descriptor()))
  }
}

extension EXT4Unpacker {
  init(_ unpacker: RustExt4Unpacker) {
    var journal: EXT4.JournalConfig?
    if unpacker.hasJournal() {
      let mode = unpacker.hasJournalMode() ? EXT4.JournalConfig.JournalMode(unpacker.journalMode()) : nil
      journal = EXT4.JournalConfig(size: unpacker.journalSize(), defaultMode: mode)
    }

    self.init(capacityInBytes: unpacker.capacityInBytes(), journal: journal)
  }
}

extension EXT4.JournalConfig.JournalMode {
  init(_ kind: JournalModeKind) {
    self =
      switch kind {
      case .Writeback: .writeback
      case .Ordered: .ordered
      case .Journal: .journal
      }
  }
}

// MARK: Swift to Rust

// The OCI image types, field by field: Rust builds its own. A field holding a
// list, a map, another of these types or an optional one of those comes back
// as an outcome of its own, holding `Absent` for `nil`.
extension CzOutcome {
  // `Platform`.

  func platformArchitecture() -> String {
    (taken() as Platform).architecture
  }

  func platformOs() -> String {
    (taken() as Platform).os
  }

  func platformOsVersion() -> String? {
    (taken() as Platform).osVersion
  }

  func platformOsFeatures() -> CzOutcome {
    CzOutcome.holding((taken() as Platform).osFeatures)
  }

  func platformVariant() -> String? {
    (taken() as Platform).variant
  }

  // `Descriptor`.

  func descriptorMediaType() -> String {
    (taken() as Descriptor).mediaType
  }

  func descriptorDigest() -> String {
    (taken() as Descriptor).digest
  }

  func descriptorSize() -> Int64 {
    (taken() as Descriptor).size
  }

  func descriptorUrls() -> CzOutcome {
    CzOutcome.holding((taken() as Descriptor).urls)
  }

  func descriptorAnnotations() -> CzOutcome {
    CzOutcome.holding((taken() as Descriptor).annotations)
  }

  func descriptorPlatform() -> CzOutcome {
    CzOutcome.holding((taken() as Descriptor).platform)
  }

  func descriptorArtifactType() -> String? {
    (taken() as Descriptor).artifactType
  }

  // `Index`.

  func indexSchemaVersion() -> Int {
    (taken() as Index).schemaVersion
  }

  func indexMediaType() -> String {
    (taken() as Index).mediaType
  }

  func indexManifests() -> CzOutcome {
    CzOutcome.holding((taken() as Index).manifests)
  }

  func indexAnnotations() -> CzOutcome {
    CzOutcome.holding((taken() as Index).annotations)
  }

  func indexSubject() -> CzOutcome {
    CzOutcome.holding((taken() as Index).subject)
  }

  func indexArtifactType() -> String? {
    (taken() as Index).artifactType
  }

  // `Manifest`.

  func manifestSchemaVersion() -> Int {
    (taken() as Manifest).schemaVersion
  }

  func manifestMediaType() -> String? {
    (taken() as Manifest).mediaType
  }

  func manifestConfig() -> CzOutcome {
    CzOutcome.holding((taken() as Manifest).config)
  }

  func manifestLayers() -> CzOutcome {
    CzOutcome.holding((taken() as Manifest).layers)
  }

  func manifestAnnotations() -> CzOutcome {
    CzOutcome.holding((taken() as Manifest).annotations)
  }

  func manifestSubject() -> CzOutcome {
    CzOutcome.holding((taken() as Manifest).subject)
  }

  func manifestArtifactType() -> String? {
    (taken() as Manifest).artifactType
  }

  // `ImageConfig`.

  func imageConfigUser() -> String? {
    (taken() as ImageConfig).user
  }

  func imageConfigEnv() -> CzOutcome {
    CzOutcome.holding((taken() as ImageConfig).env)
  }

  func imageConfigEntrypoint() -> CzOutcome {
    CzOutcome.holding((taken() as ImageConfig).entrypoint)
  }

  func imageConfigCmd() -> CzOutcome {
    CzOutcome.holding((taken() as ImageConfig).cmd)
  }

  func imageConfigWorkingDir() -> String? {
    (taken() as ImageConfig).workingDir
  }

  func imageConfigLabels() -> CzOutcome {
    CzOutcome.holding((taken() as ImageConfig).labels)
  }

  func imageConfigStopSignal() -> String? {
    (taken() as ImageConfig).stopSignal
  }

  // `Rootfs`.

  func rootfsType() -> String {
    (taken() as Rootfs).type
  }

  func rootfsDiffIDs() -> RustVec<RustString> {
    rustStrings((taken() as Rootfs).diffIDs)
  }

  // `History`.

  func historyCreated() -> String? {
    (taken() as History).created
  }

  func historyCreatedBy() -> String? {
    (taken() as History).createdBy
  }

  func historyAuthor() -> String? {
    (taken() as History).author
  }

  func historyComment() -> String? {
    (taken() as History).comment
  }

  func historyEmptyLayer() -> Bool? {
    (taken() as History).emptyLayer
  }

  // `ContainerizationOCI.Image`.

  func ociImageCreated() -> String? {
    (taken() as ContainerizationOCI.Image).created
  }

  func ociImageAuthor() -> String? {
    (taken() as ContainerizationOCI.Image).author
  }

  func ociImageArchitecture() -> String {
    (taken() as ContainerizationOCI.Image).architecture
  }

  func ociImageOs() -> String {
    (taken() as ContainerizationOCI.Image).os
  }

  func ociImageOsVersion() -> String? {
    (taken() as ContainerizationOCI.Image).osVersion
  }

  func ociImageOsFeatures() -> CzOutcome {
    CzOutcome.holding((taken() as ContainerizationOCI.Image).osFeatures)
  }

  func ociImageVariant() -> String? {
    (taken() as ContainerizationOCI.Image).variant
  }

  func ociImageConfig() -> CzOutcome {
    CzOutcome.holding((taken() as ContainerizationOCI.Image).config)
  }

  func ociImageRootfs() -> CzOutcome {
    CzOutcome.holding((taken() as ContainerizationOCI.Image).rootfs)
  }

  func ociImageHistory() -> CzOutcome {
    CzOutcome.holding((taken() as ContainerizationOCI.Image).history)
  }
}

/// Swift's `MediaTypes`, in the order of Rust's `MediaTypes::ALL`, for a test
/// that compares Rust's copies with them.
func mediaTypes() -> RustVec<RustString> {
  rustStrings([
    MediaTypes.descriptor,
    MediaTypes.layoutHeader,
    MediaTypes.index,
    MediaTypes.imageManifest,
    MediaTypes.imageConfig,
    MediaTypes.emptyJSON,
    MediaTypes.dockerManifest,
    MediaTypes.dockerManifestList,
    MediaTypes.dockerImageConfig,
    MediaTypes.imageLayer,
    MediaTypes.imageLayerGzip,
    MediaTypes.imageLayerZstd,
    MediaTypes.dockerImageLayer,
    MediaTypes.dockerImageLayerGzip,
    MediaTypes.dockerImageLayerZstd,
    MediaTypes.inTotoAttestationBlob,
  ])
}

/// Swift's `AnnotationKeys`, in the order of Rust's `AnnotationKeys::ALL`.
func annotationKeys() -> RustVec<RustString> {
  rustStrings([
    AnnotationKeys.containerizationIndexIndirect,
    AnnotationKeys.containerizationImageName,
    AnnotationKeys.containerdImageName,
    AnnotationKeys.openContainersImageName,
  ])
}

func currentPlatform() -> CzOutcome {
  CzOutcome { Platform.current }
}

func parsePlatform(platform: RustStr) -> CzOutcome {
  let platform = platform.toString()

  return CzOutcome { try Platform(from: platform) }
}

func platformDescription(platform: RustPlatform) -> CzOutcome {
  let platform = Platform(platform)

  return CzOutcome { platform.description }
}

func platformEquals(lhs: RustPlatform, rhs: RustPlatform) -> CzOutcome {
  let lhs = Platform(lhs)
  let rhs = Platform(rhs)

  return CzOutcome { lhs == rhs }
}

func platformMatches(lhs: RustPlatform, rhs: RustPlatform) -> CzOutcome {
  let lhs = Platform(lhs)
  let rhs = Platform(rhs)

  return CzOutcome { lhs ~= rhs }
}

func indexRunnablePlatforms(index: RustIndex) -> CzOutcome {
  let index = Index(index)

  return CzOutcome { index.runnablePlatforms }
}
