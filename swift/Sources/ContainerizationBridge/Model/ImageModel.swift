//===----------------------------------------------------------------------===//
// Rust's values for images, content and unpacking read into Containerization's,
// and a `Platform` filled back into Rust's.
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

extension Containerization.Image.Description {
  init(_ description: RustImageDescription) {
    self.init(reference: description.reference().toString(), descriptor: Descriptor(description.descriptor()))
  }
}

extension EXT4Unpacker {
  init(_ unpacker: RustExt4Unpacker) {
    var journal: EXT4.JournalConfig?
    if unpacker.hasJournal() {
      var mode: EXT4.JournalConfig.JournalMode?
      if unpacker.hasJournalMode() {
        mode =
          switch unpacker.journalMode() {
          case .Writeback: .writeback
          case .Ordered: .ordered
          case .Journal: .journal
          }
      }
      journal = EXT4.JournalConfig(size: unpacker.journalSize(), defaultMode: mode)
    }

    self.init(capacityInBytes: unpacker.capacityInBytes(), journal: journal)
  }
}

/// Rust's `ProgressHandler?`. Swift may call it from several threads at once,
/// and Rust's is `Sync`.
func progressHandler(_ handler: RustProgressHandler) -> ProgressHandler? {
  guard handler.isSome() else {
    return nil
  }
  nonisolated(unsafe) let handler = handler

  return { events in
    let kinds = RustVec<ProgressKind>()
    let values = RustVec<Int64>()
    for event in events {
      switch event {
      case .addItems(let value):
        kinds.push(value: .Items)
        values.push(value: Int64(value))
      case .addTotalItems(let value):
        kinds.push(value: .TotalItems)
        values.push(value: Int64(value))
      case .addSize(let value):
        kinds.push(value: .Size)
        values.push(value: value)
      case .addTotalSize(let value):
        kinds.push(value: .TotalSize)
        values.push(value: value)
      }
    }
    handler.call(kinds, values)
  }
}

// MARK: Swift to Rust

// The OCI image types, field by field: Rust builds its own. A field holding a
// list, a map, another of these types or an optional one of those comes back
// as an outcome of its own, holding `Absent` for `nil`.
extension CzOutcome {
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
