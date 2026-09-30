//===----------------------------------------------------------------------===//
// The end of a build: the built rootfs, exported and stored as an image.
//
// The fragile step: users, modes, symlinks, hardlinks and extended attributes
// must survive EXT4Reader's inode reading and libarchive's pax format. Only a
// session on the built image verifies they did.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationEXT4
import ContainerizationOCI
import Foundation
import Synchronization
import SystemPackage

extension Build {
  /// Exports the built rootfs and writes it into the store as a single-layer
  /// image, returning the index descriptor the reference points at.
  ///
  /// The layer is an uncompressed tar: nothing pushes this image, and an
  /// uncompressed blob's digest is also its diffID, so both are correct in
  /// one pass. (Containerization's `InitImage.create` has a standing `TODO`
  /// for writing a gzip layer's compressed digest as its diffID.)
  static func ingest(
    rootfs: URL,
    plan: BuildPlan,
    base: ImageConfig?,
    environment: [String],
    platform: Platform,
    contentStore: ContentStore
  ) async throws -> Descriptor {
    let layer = rootfs.deletingLastPathComponent().appending(path: "layer.tar")
    try? FileManager.default.removeItem(at: layer)

    let reader = try EXT4.EXT4Reader(blockDevice: FilePath(rootfs.path(percentEncoded: false)))
    try reader.export(archive: FilePath(layer.path(percentEncoded: false)))

    let index = Box<Descriptor>()
    let user = plan.user ?? base?.user
    let workingDirectory = plan.workingDirectory ?? base?.workingDir
    // The base's labels carry over, as `LABEL` does; the plan's win on a
    // key both set.
    let labels = (base?.labels ?? [:]).merging(plan.labels) { _, mine in mine }
    let entrypoint = base?.entrypoint
    let command = base?.cmd

    try await contentStore.ingest { directory in
      let writer = try ContentWriter(for: directory)

      var result = try writer.create(from: layer)
      let layerDescriptor = Descriptor(
        mediaType: MediaTypes.imageLayer,
        digest: result.digest.digestString,
        size: result.size
      )
      let diffID = result.digest.digestString

      let config = ContainerizationOCI.Image(
        architecture: platform.architecture,
        os: platform.os,
        variant: platform.variant,
        config: ImageConfig(
          user: user,
          env: environment,
          entrypoint: entrypoint,
          cmd: command,
          workingDir: workingDirectory,
          labels: labels
        ),
        rootfs: Rootfs(type: "layers", diffIDs: [diffID])
      )
      result = try writer.create(from: config)
      let configDescriptor = Descriptor(
        mediaType: MediaTypes.imageConfig,
        digest: result.digest.digestString,
        size: result.size
      )

      result = try writer.create(from: Manifest(config: configDescriptor, layers: [layerDescriptor]))
      let manifestDescriptor = Descriptor(
        mediaType: MediaTypes.imageManifest,
        digest: result.digest.digestString,
        size: result.size,
        platform: platform
      )

      result = try writer.create(from: Index(manifests: [manifestDescriptor]))
      index.value = Descriptor(
        mediaType: MediaTypes.index,
        digest: result.digest.digestString,
        size: result.size
      )
    }

    try? FileManager.default.removeItem(at: layer)

    guard let descriptor = index.value else {
      throw BridgeError.notIngested(plan.tag)
    }

    return descriptor
  }
}

/// Carries a result out of `ingest`'s `@Sendable` body, which cannot return one
/// or capture a bare `Mutex`.
private final class Box<Value: Sendable>: Sendable {
  private let stored = Mutex<Value?>(nil)

  var value: Value? {
    get { stored.withLock { $0 } }
    set { stored.withLock { $0 = newValue } }
  }
}
