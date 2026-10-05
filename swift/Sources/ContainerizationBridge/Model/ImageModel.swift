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

func currentPlatform() -> CzOutcome {
  CzOutcome { Platform.current }
}
