//===----------------------------------------------------------------------===//
// `EXT4.EXT4Reader`, from ContainerizationEXT4, and `EXT4Unpacker`, from
// Containerization.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationEXT4
import ContainerizationOCI
import Foundation
import SystemPackage

func openExt4Reader(blockDevice: RustStr) -> CzOutcome {
  let blockDevice = FilePath(blockDevice.toString())

  return CzOutcome { CzExt4Reader(try EXT4.EXT4Reader(blockDevice: blockDevice)) }
}

/// Unchecked: Rust's `Ext4Reader` isn't `Sync`, so one call runs at a time.
final class CzExt4Reader: @unchecked Sendable {
  let reader: EXT4.EXT4Reader

  init(_ reader: EXT4.EXT4Reader) {
    self.reader = reader
  }

  func export(archive: RustStr) -> CzOutcome {
    let archive = FilePath(archive.toString())

    return CzOutcome { try reader.export(archive: archive) }
  }
}

func unpackExt4(
  unpacker: RustExt4Unpacker,
  image: CzImage,
  platform: RustPlatform,
  at: RustStr,
  progress: RustProgressHandler
) -> CzOutcome {
  // Not `Sendable`, but a value only the task below uses.
  nonisolated(unsafe) let unpacker = EXT4Unpacker(unpacker)
  let image = image.image
  let platform = Platform(platform)
  let at = URL(filePath: at.toString())
  let progress = progressHandler(progress)

  return CzOutcome {
    try blocking { try await unpacker.unpack(image, for: platform, at: at, progress: progress) }
  }
}
