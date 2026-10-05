//===----------------------------------------------------------------------===//
// `ImageStore`, `Image` and `InitImage`, from Containerization.
//===----------------------------------------------------------------------===//

import Containerization
import Foundation

import struct ContainerizationOCI.Platform

func openImageStore(path: RustStr) -> CzOutcome {
  let path = URL(filePath: path.toString())

  return CzOutcome { CzImageStore(try ImageStore(path: path)) }
}

// Made from the content store it takes, which swift-bridge can't pass as an
// argument.
extension CzLocalContentStore {
  func imageStore(path: RustStr) -> CzOutcome {
    let path = URL(filePath: path.toString())
    let store = store

    return CzOutcome { CzImageStore(try ImageStore(path: path, contentStore: store)) }
  }
}

final class CzImageStore: Sendable {
  let store: ImageStore

  init(_ store: ImageStore) {
    self.store = store
  }

  func path() -> String {
    store.path.path(percentEncoded: false)
  }

  func get(reference: RustStr, pull: Bool) -> CzOutcome {
    let reference = reference.toString()
    let store = store

    return CzOutcome { CzImage(try blocking { try await store.get(reference: reference, pull: pull) }) }
  }

  func list() -> CzOutcome {
    let store = store

    return CzOutcome { CzImages(try blocking { try await store.list() }) }
  }

  func delete(reference: RustStr, performCleanup: Bool) -> CzOutcome {
    let reference = reference.toString()
    let store = store

    return CzOutcome { try blocking { try await store.delete(reference: reference, performCleanup: performCleanup) } }
  }

  func tag(existing: RustStr, new: RustStr) -> CzOutcome {
    let existing = existing.toString()
    let new = new.toString()
    let store = store

    return CzOutcome { CzImage(try blocking { try await store.tag(existing: existing, new: new) }) }
  }

  func pull(reference: RustStr) -> CzOutcome {
    let reference = reference.toString()
    let store = store

    return CzOutcome { CzImage(try blocking { try await store.pull(reference: reference) }) }
  }

  func getInitImage(reference: RustStr) -> CzOutcome {
    let reference = reference.toString()
    let store = store

    return CzOutcome { CzInitImage(try blocking { try await store.getInitImage(reference: reference) }) }
  }

  func cleanUpOrphanedBlobs() -> CzOutcome {
    let store = store

    return CzOutcome {
      let (deleted, freed) = try blocking { try await store.cleanUpOrphanedBlobs() }
      return (deleted, freed)
    }
  }

  func calculateOrphanedBlobsSize() -> CzOutcome {
    let store = store

    return CzOutcome { try blocking { try await store.calculateOrphanedBlobsSize() } }
  }

  func create(description: RustImageDescription) -> CzOutcome {
    let description = Image.Description(description)
    let store = store

    return CzOutcome { CzImage(try blocking { try await store.create(description: description) }) }
  }

  func load(from: RustStr, progress: RustProgressHandler) -> CzOutcome {
    let directory = URL(filePath: from.toString())
    let progress = progressHandler(progress)
    let store = store

    return CzOutcome { CzImages(try blocking { try await store.load(from: directory, progress: progress) }) }
  }
}

/// An `[Image]`, by index.
final class CzImages {
  let images: [Image]

  init(_ images: [Image]) {
    self.images = images
  }

  func len() -> UInt {
    UInt(images.count)
  }

  func at(index: UInt) -> CzImage {
    CzImage(images[Int(index)])
  }
}

final class CzImage: Sendable {
  let image: Image

  init(_ image: Image) {
    self.image = image
  }

  /// Another handle on the same image, to pass where Rust would lend one.
  func duplicate() -> CzImage {
    CzImage(image)
  }

  func reference() -> String {
    image.reference
  }

  func digest() -> String {
    image.digest
  }

  func mediaType() -> String {
    image.mediaType
  }

  /// `descriptor`, which doesn't throw: Rust reads it without asking `error`.
  func descriptor() -> CzOutcome {
    CzOutcome.holding(image.descriptor)
  }

  func index() -> CzOutcome {
    let image = image

    return CzOutcome { try blocking { try await image.index() } }
  }

  func manifest(platform: RustPlatform) -> CzOutcome {
    let platform = Platform(platform)
    let image = image

    return CzOutcome { try blocking { try await image.manifest(for: platform) } }
  }

  func descriptorFor(platform: RustPlatform) -> CzOutcome {
    let platform = Platform(platform)
    let image = image

    return CzOutcome { try blocking { try await image.descriptor(for: platform) } }
  }

  func config(platform: RustPlatform) -> CzOutcome {
    let platform = Platform(platform)
    let image = image

    return CzOutcome { try blocking { try await image.config(for: platform) } }
  }

  func referencedDigests() -> CzOutcome {
    let image = image

    return CzOutcome { try blocking { try await image.referencedDigests() } }
  }

  func getContent(digest: RustStr) -> CzOutcome {
    let digest = digest.toString()
    let image = image

    return CzOutcome { CzContent(try blocking { try await image.getContent(digest: digest) }) }
  }
}

final class CzInitImage: Sendable {
  let image: InitImage

  init(_ image: InitImage) {
    self.image = image
  }

  func name() -> String {
    image.name
  }

  func initBlock(at: RustStr, platform: RustSystemPlatform) -> CzOutcome {
    let at = URL(filePath: at.toString())
    let image = image

    return CzOutcome {
      let platform = try SystemPlatform(platform)

      return try blocking { try await image.initBlock(at: at, for: platform) }
    }
  }
}
