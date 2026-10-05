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

  // Each outcome holds the size and the digest, as its `digestString`.

  func write(data: RustVec<UInt8>) -> CzOutcome {
    let data = Data(data)

    return CzOutcome {
      let (size, digest) = try writer.write(data)
      return Written(size: size, digest: digest.digestString)
    }
  }

  func create(from: RustStr) -> CzOutcome {
    let url = URL(filePath: from.toString())

    return CzOutcome {
      let (size, digest) = try writer.create(from: url)
      return Written(size: size, digest: digest.digestString)
    }
  }
}

func copyContent(from: RustStr, destination: RustStr) -> CzOutcome {
  let url = URL(filePath: from.toString())
  let destination = URL(filePath: destination.toString())

  return CzOutcome {
    let (size, digest) = try ContentWriter.copy(from: url, destination: destination)
    return Written(size: size, digest: digest.digestString)
  }
}

/// What a `ContentWriter` wrote.
struct Written {
  let size: Int64
  let digest: String
}
