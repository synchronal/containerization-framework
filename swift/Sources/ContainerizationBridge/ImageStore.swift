//===----------------------------------------------------------------------===//
// `ImageStore`, `Image`, `InitImage` and `KernelImage`, from Containerization.
//===----------------------------------------------------------------------===//

import Containerization
import Foundation

import struct ContainerizationOCI.Platform

func openImageStore(path: RustStr) -> CzOutcome {
  let path = URL(filePath: path.toString())

  return CzOutcome { CzImageStore(try ImageStore(path: path)) }
}

func defaultImageStore() -> CzOutcome {
  CzOutcome { CzImageStore(ImageStore.default) }
}

// Made from the content store they take, which swift-bridge can't pass as an
// argument.
extension CzLocalContentStore {
  func imageStore(path: RustStr) -> CzOutcome {
    let path = URL(filePath: path.toString())
    let store = store

    return CzOutcome { CzImageStore(try ImageStore(path: path, contentStore: store)) }
  }

  func image(description: RustImageDescription) -> CzImage {
    CzImage(Image(description: Image.Description(description), contentStore: store))
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

  func pull(
    reference: RustStr,
    hasPlatform: Bool,
    platform: RustPlatform,
    insecure: Bool,
    auth: CzAuthentication,
    progress: RustProgressHandler,
    maxConcurrentDownloads: UInt
  ) -> CzOutcome {
    let reference = reference.toString()
    let platform = hasPlatform ? Platform(platform) : nil
    let auth = auth.authentication
    let progress = progressHandler(progress)
    let store = store

    return CzOutcome {
      CzImage(
        try blocking {
          try await store.pull(
            reference: reference,
            platform: platform,
            insecure: insecure,
            auth: auth,
            progress: progress,
            maxConcurrentDownloads: Int(maxConcurrentDownloads)
          )
        })
    }
  }

  func push(
    reference: RustStr,
    hasPlatform: Bool,
    platform: RustPlatform,
    insecure: Bool,
    auth: CzAuthentication,
    progress: RustProgressHandler
  ) -> CzOutcome {
    let reference = reference.toString()
    let platform = hasPlatform ? Platform(platform) : nil
    let auth = auth.authentication
    let progress = progressHandler(progress)
    let store = store

    return CzOutcome {
      try blocking {
        try await store.push(
          reference: reference, platform: platform, insecure: insecure, auth: auth, progress: progress)
      }
    }
  }

  func pushAll(
    references: RustVec<RustString>,
    hasPlatform: Bool,
    platform: RustPlatform,
    insecure: Bool,
    auth: CzAuthentication,
    maxConcurrentUploads: UInt,
    progress: RustProgressHandler
  ) -> CzOutcome {
    let references = strings(references)
    let platform = hasPlatform ? Platform(platform) : nil
    let auth = auth.authentication
    let progress = progressHandler(progress)
    let store = store

    return CzOutcome {
      try blocking {
        try await store.push(
          references: references,
          platform: platform,
          insecure: insecure,
          auth: auth,
          maxConcurrentUploads: Int(maxConcurrentUploads),
          progress: progress
        )
      }
    }
  }

  func getInitImage(reference: RustStr, auth: CzAuthentication, progress: RustProgressHandler) -> CzOutcome {
    let reference = reference.toString()
    let auth = auth.authentication
    let progress = progressHandler(progress)
    let store = store

    return CzOutcome {
      CzInitImage(try blocking { try await store.getInitImage(reference: reference, auth: auth, progress: progress) })
    }
  }

  func save(references: RustVec<RustString>, out: RustStr, hasPlatform: Bool, platform: RustPlatform) -> CzOutcome {
    let references = strings(references)
    let out = URL(filePath: out.toString())
    let platform = hasPlatform ? Platform(platform) : nil
    let store = store

    return CzOutcome { try blocking { try await store.save(references: references, out: out, platform: platform) } }
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

  func createInitImage(
    reference: RustStr,
    rootfs: RustStr,
    platform: RustPlatform,
    labelKeys: RustVec<RustString>,
    labelValues: RustVec<RustString>,
    contentStore: CzLocalContentStore
  ) -> CzOutcome {
    let reference = reference.toString()
    let rootfs = URL(filePath: rootfs.toString())
    let platform = Platform(platform)
    let labels = dictionary(labelKeys, labelValues)
    let contentStore = contentStore.store
    let store = store

    return CzOutcome {
      CzInitImage(
        try blocking {
          try await InitImage.create(
            reference: reference,
            rootfs: rootfs,
            platform: platform,
            labels: labels,
            imageStore: store,
            contentStore: contentStore
          )
        })
    }
  }

  func createKernelImage(
    reference: RustStr,
    binaries: RustVec<RustKernel>,
    labelKeys: RustVec<RustString>,
    labelValues: RustVec<RustString>,
    contentStore: CzLocalContentStore
  ) -> CzOutcome {
    let reference = reference.toString()
    let labels = dictionary(labelKeys, labelValues)
    let contentStore = contentStore.store
    let store = store

    return CzOutcome {
      let binaries = try binaries.map { try Kernel($0) }

      return CzKernelImage(
        try blocking {
          try await KernelImage.create(
            reference: reference,
            binaries: binaries,
            labels: labels,
            imageStore: store,
            contentStore: contentStore
          )
        })
    }
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

  func initImage() -> CzInitImage {
    CzInitImage(InitImage(image: image))
  }

  func kernelImage() -> CzKernelImage {
    CzKernelImage(KernelImage(image: image))
  }
}

final class CzKernelImage: Sendable {
  let image: KernelImage

  init(_ image: KernelImage) {
    self.image = image
  }

  func name() -> String {
    image.name
  }

  func kernel(platform: RustSystemPlatform) -> CzOutcome {
    let image = image

    return CzOutcome {
      let platform = try SystemPlatform(platform)

      return try blocking { try await image.kernel(for: platform) }
    }
  }
}

func kernelImageMediaType() -> String {
  KernelImage.mediaType
}

extension CzOutcome {
  func kernelImage() -> CzKernelImage { taken() }

  // A `Kernel`, field by field: Rust builds its own.

  func kernelPath() -> String {
    (taken() as Kernel).path.path(percentEncoded: false)
  }

  func kernelPlatformOs() -> PlatformOs {
    switch (taken() as Kernel).platform.os {
    case .linux: .Linux
    case .darwin: .Darwin
    }
  }

  func kernelPlatformArchitecture() -> PlatformArchitecture {
    switch (taken() as Kernel).platform.architecture {
    case .arm64: .Arm64
    case .amd64: .Amd64
    }
  }

  func kernelKernelArgs() -> RustVec<RustString> {
    rustStrings((taken() as Kernel).commandLine.kernelArgs)
  }

  func kernelInitArgs() -> RustVec<RustString> {
    rustStrings((taken() as Kernel).commandLine.initArgs)
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
