//===----------------------------------------------------------------------===//
// `ContentWriter`, from ContainerizationOCI.
//===----------------------------------------------------------------------===//

import ContainerizationOCI
import Foundation

func openContentWriter(base: RustStr) -> CzOutcome {
  let base = URL(filePath: base.toString())

  return CzOutcome { CzContentWriter(try ContentWriter(for: base)) }
}

/// Unchecked: Rust's `ContentWriter` isn't `Sync`, so one call runs at a time.
final class CzContentWriter: @unchecked Sendable {
  let writer: ContentWriter

  init(_ writer: ContentWriter) {
    self.writer = writer
  }

  /// The size and the digest, as its `digestString`.
  func create(from: RustStr) -> CzOutcome {
    let url = URL(filePath: from.toString())

    return CzOutcome {
      let (size, digest) = try writer.create(from: url)
      return Written(size: size, digest: digest.digestString)
    }
  }
}

/// What a `ContentWriter` wrote.
struct Written {
  let size: Int64
  let digest: String
}
