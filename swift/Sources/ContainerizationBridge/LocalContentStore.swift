//===----------------------------------------------------------------------===//
// `LocalContentStore` and `Content`, from ContainerizationOCI.
//===----------------------------------------------------------------------===//

import ContainerizationOCI
import Foundation

func openLocalContentStore(path: RustStr) -> CzOutcome {
  let path = URL(filePath: path.toString())

  return CzOutcome { CzLocalContentStore(try LocalContentStore(path: path)) }
}

final class CzLocalContentStore: Sendable {
  let store: LocalContentStore

  init(_ store: LocalContentStore) {
    self.store = store
  }

  func get(digest: RustStr) -> CzOutcome {
    let digest = digest.toString()
    let store = store

    return CzOutcome { CzContent(try blocking { try await store.get(digest: digest) }) }
  }

  func deleteDigests(digests: RustVec<RustString>) -> CzOutcome {
    let digests = strings(digests)
    let store = store

    return CzOutcome { try blocking { try await store.delete(digests: digests) } }
  }

  func deleteKeeping(keeping: RustVec<RustString>) -> CzOutcome {
    let keeping = strings(keeping)
    let store = store

    return CzOutcome { try blocking { try await store.delete(keeping: keeping) } }
  }

  func totalAllocatedSize() -> CzOutcome {
    let store = store

    return CzOutcome { try blocking { try await store.totalAllocatedSize() } }
  }

  /// `body` is Rust's: it gets the ingest directory, and returns whether it
  /// succeeded. Rust keeps its own error, and returns that in place of the one
  /// thrown here.
  func ingest(body: (RustString) -> Bool) -> CzOutcome {
    let store = store

    return CzOutcome {
      // Doesn't escape: `blocking` returns only once `ingest` is done.
      try withoutActuallyEscaping(body) { body in
        nonisolated(unsafe) let body = body

        return try blocking {
          try await store.ingest { directory in
            guard body(directory.path(percentEncoded: false).intoRustString()) else {
              throw BridgeError.bodyFailed
            }
          }
        }
      }
    }
  }
}

/// A `Content?`: Rust asks `isSome` before anything else.
final class CzContent: Sendable {
  let content: (any Content)?

  init(_ content: (any Content)?) {
    self.content = content
  }

  func isSome() -> Bool {
    content != nil
  }

  func path() -> String {
    content?.path.path(percentEncoded: false) ?? ""
  }

  func digest() -> CzOutcome {
    CzOutcome { try content?.digest().digestString ?? "" }
  }

  func size() -> CzOutcome {
    CzOutcome { try content?.size() ?? 0 }
  }

  func data() -> CzOutcome {
    CzOutcome { try content?.data() as Data? }
  }

  func dataRange(offset: UInt64, length: UInt) -> CzOutcome {
    CzOutcome { try content?.data(offset: offset, length: Int(length)) as Data? }
  }
}
