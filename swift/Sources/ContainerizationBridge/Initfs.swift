//===----------------------------------------------------------------------===//
// The init image a VM boots into, unpacked where the caller says. Provisioning
// puts it there; a session or build mounts it, unpacking it if still missing.
//===----------------------------------------------------------------------===//

import Containerization
import Foundation

/// The init image, unpacked to the caller's path, for
/// `ContainerManager(initfs:)`. What `ContainerManager(initfsReference:)`
/// does, minus its fixed `initfs.ext4`.
enum Initfs {
    /// The image at `path`, read-only; unpacked first if nothing is there.
    static func mount(
        _ reference: String,
        at path: URL,
        in imageStore: ImageStore
    ) async throws -> Containerization.Mount {
        if !FileManager.default.fileExists(atPath: path.path(percentEncoded: false)) {
            try await unpack(reference, to: path, in: imageStore)
        }

        return .block(format: "ext4", source: path.absolutePath(), destination: "/", options: ["ro"])
    }

    /// Unpacked aside and moved into place: no half-written file, no sharing
    /// between concurrent first boots.
    private static func unpack(_ reference: String, to path: URL, in imageStore: ImageStore) async throws {
        let directory = path.deletingLastPathComponent()
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)

        let image = try await imageStore.getInitImage(reference: reference)
        let partial = directory.appending(path: "\(UUID().uuidString).partial")
        defer { try? FileManager.default.removeItem(at: partial) }

        _ = try await image.initBlock(at: partial, for: .linuxArm)

        do {
            try FileManager.default.moveItem(at: partial, to: path)
        } catch where FileManager.default.fileExists(atPath: path.path(percentEncoded: false)) {
            // Another boot unpacked it first; its copy is equivalent.
        }
    }
}
