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

  /// Another handle on the same store, to pass where Rust would lend one.
  func duplicate() -> CzLocalContentStore {
    CzLocalContentStore(store)
  }

  func get(digest: RustStr) -> CzOutcome {
    let digest = digest.toString()

    return .awaiting { CzContent(try await self.store.get(digest: digest)) }
  }

  func deleteDigests(digests: RustVec<RustString>) -> CzOutcome {
    let digests = strings(digests)

    return .awaiting { try await self.store.delete(digests: digests) }
  }

  func deleteKeeping(keeping: RustVec<RustString>) -> CzOutcome {
    let keeping = strings(keeping)

    return .awaiting { try await self.store.delete(keeping: keeping) }
  }

  func newIngestSession() -> CzOutcome {
    .awaiting { try await self.store.newIngestSession() }
  }

  func completeIngestSession(id: RustStr) -> CzOutcome {
    let id = id.toString()

    return .awaiting { try await self.store.completeIngestSession(id) }
  }

  func cancelIngestSession(id: RustStr) -> CzOutcome {
    let id = id.toString()

    return .awaiting { try await self.store.cancelIngestSession(id) }
  }

  func totalAllocatedSize() -> CzOutcome {
    .awaiting { try await self.store.totalAllocatedSize() }
  }

  /// `body` is Rust's: it gets the ingest directory, and returns whether it
  /// threw. Rust keeps its own error, and returns that in place of the one
  /// thrown here. Swift's closure is `async`, so `body` runs off the
  /// cooperative pool and may call back into Swift.
  func ingest(body: (RustString) -> Bool) -> CzOutcome {
    let store = store

    return CzOutcome {
      // Doesn't escape: `blocking` returns only once `ingest` is done.
      try withoutActuallyEscaping(body) { body in
        nonisolated(unsafe) let body = body

        return try blocking {
          try await store.ingest { directory in
            let threw = await offPool {
              body(directory.path(percentEncoded: false).intoRustString())
            }
            if threw {
              throw BridgeError.bodyFailed
            }
          }
        }
      }
    }
  }
}

extension CzOutcome {
  func localContentStore() -> CzLocalContentStore { taken() }
  func content() -> CzContent { taken() }

  // What `newIngestSession` returned.

  func ingestSessionId() -> String {
    (taken() as (id: String, ingestDir: URL)).id
  }

  func ingestSessionDirectory() -> String {
    (taken() as (id: String, ingestDir: URL)).ingestDir.path(percentEncoded: false)
  }
}

func openLocalContent(path: RustStr) -> CzOutcome {
  let path = URL(filePath: path.toString())

  return CzOutcome { CzContent(try LocalContent(path: path)) }
}

func localContentMaxDecodedSize() -> Int {
  LocalContent.maxDecodedSize
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
    CzOutcome { try content?.data(offset: offset, length: try int(length, "length")) as Data? }
  }
}
