//===----------------------------------------------------------------------===//
// `ContentWriter`, from ContainerizationOCI.
//===----------------------------------------------------------------------===//

import ContainerizationOCI
import Crypto
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

    return CzOutcome { Written(try writer.write(data)) }
  }

  func create(from: RustStr) -> CzOutcome {
    let url = URL(filePath: from.toString())

    return CzOutcome { Written(try writer.create(from: url)) }
  }
}

func copyContent(from: RustStr, destination: RustStr) -> CzOutcome {
  let url = URL(filePath: from.toString())
  let destination = URL(filePath: destination.toString())

  return CzOutcome { Written(try ContentWriter.copy(from: url, destination: destination)) }
}

/// What a `ContentWriter` wrote.
struct Written {
  let size: Int64
  let digest: String

  /// The size and digest a write returns.
  init(_ written: (Int64, SHA256.Digest)) {
    size = written.0
    digest = written.1.digestString
  }
}

extension CzOutcome {
  func contentWriter() -> CzContentWriter { taken() }

  func writtenSize() -> Int64 {
    (taken() as Written).size
  }

  func writtenDigest() -> String {
    (taken() as Written).digest
  }
}
