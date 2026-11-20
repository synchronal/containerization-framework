//===----------------------------------------------------------------------===//
// `SnapshotManager`, from Containerization.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationOCI
import Foundation

func snapshotManager(path: RustStr, capacityInBytes: UInt64) -> CzOutcome {
  let path = URL(filePath: path.toString())

  return CzOutcome { CzSnapshotManager(try SnapshotManager(path: path, capacityInBytes: capacityInBytes)) }
}

/// `SnapshotManager.key(of:for:)`, with `Platform.current` when
/// `hasPlatform` is false.
func snapshotKey(image: CzImage, hasPlatform: Bool, platform: RustPlatform) -> CzOutcome {
  let image = image.image
  let platform = hasPlatform ? Platform(platform) : .current

  return .awaiting { try await SnapshotManager.key(of: image, for: platform) }
}

final class CzSnapshotManager: Sendable {
  let snapshots: SnapshotManager

  init(_ snapshots: SnapshotManager) {
    self.snapshots = snapshots
  }

  func duplicate() -> CzSnapshotManager {
    CzSnapshotManager(snapshots)
  }

  func path() -> String {
    snapshots.path.path(percentEncoded: false)
  }

  func unpack(image: CzImage, hasPlatform: Bool, platform: RustPlatform, progress: RustProgressHandler)
    -> CzOutcome
  {
    let image = image.image
    let platform = hasPlatform ? Platform(platform) : .current
    let progress = progressHandler(progress)

    return .awaiting {
      try await self.snapshots.unpack(image, for: platform, progress: progress).path(percentEncoded: false)
    }
  }

  /// Holds the path, or `Absent` for `nil`.
  func get(image: CzImage, hasPlatform: Bool, platform: RustPlatform) -> CzOutcome {
    let image = image.image
    let platform = hasPlatform ? Platform(platform) : .current

    return CzOutcome {
      absent(try blocking { try await self.snapshots.get(image, for: platform) }?.path(percentEncoded: false))
    }
  }

  func clone(image: CzImage, hasPlatform: Bool, platform: RustPlatform, to: RustStr) -> CzOutcome {
    let image = image.image
    let platform = hasPlatform ? Platform(platform) : .current
    let to = URL(filePath: to.toString())

    return .awaiting { try await self.snapshots.clone(image, for: platform, to: to) }
  }

  func delete(keeping: CzImageList) -> CzOutcome {
    let images = keeping.images

    return CzOutcome {
      // Rebuilt without its labels, as the exact `([String], UInt64)` that
      // `strings` and `number` cast to.
      let (deleted, freed) = try blocking { try await self.snapshots.delete(keeping: images) }
      return (deleted, freed)
    }
  }

  func asSnapshots() -> CzSnapshots {
    CzSnapshots(snapshots)
  }
}

/// A `SnapshotManager?`, for the container manager's inits.
final class CzSnapshots: Sendable {
  let snapshots: SnapshotManager?

  init(_ snapshots: SnapshotManager?) {
    self.snapshots = snapshots
  }
}

func noSnapshots() -> CzSnapshots {
  CzSnapshots(nil)
}

/// Unchecked: Rust fills it on one thread before handing it to `delete`.
final class CzImageList: @unchecked Sendable {
  var images: [Containerization.Image] = []

  func push(image: CzImage) {
    images.append(image.image)
  }
}

func newImageList() -> CzImageList {
  CzImageList()
}

extension CzOutcome {
  func snapshotManager() -> CzSnapshotManager { taken() }
}
