//===----------------------------------------------------------------------===//
// Descriptors as the streams Containerization reads and writes.
//
// `LinuxProcessConfiguration` takes a `ReaderStream` and `Writer`s; Rust passes
// descriptors. Each is duplicated here, so the stream owns its copy and the
// caller keeps theirs.
//===----------------------------------------------------------------------===//

import Containerization
import Foundation
import Synchronization

/// A duplicate of `descriptor`, closed with the handle.
func duplicate(_ descriptor: Int32) throws -> FileHandle {
  let copy = dup(descriptor)

  guard copy >= 0 else {
    throw BridgeError.malformed("descriptor", String(descriptor))
  }

  return FileHandle(fileDescriptor: copy, closeOnDealloc: true)
}

/// A descriptor the guest reads, streamed until it reaches end of file.
///
/// Unchecked: `FileHandle` is not `Sendable`, and only the readability handler
/// touches it, on one queue at a time.
final class FileReader: ReaderStream, @unchecked Sendable {
  private let handle: FileHandle

  init(_ handle: FileHandle) {
    self.handle = handle
  }

  func stream() -> AsyncStream<Data> {
    .init { continuation in
      handle.readabilityHandler = { handle in
        let data = handle.availableData

        // Empty means end of file. Containerization closes the guest's
        // stdin once the stream finishes.
        guard !data.isEmpty else {
          handle.readabilityHandler = nil
          continuation.finish()
          return
        }

        continuation.yield(data)
      }
    }
  }
}

/// A descriptor the guest writes. Locked so stdout and stderr chunks never
/// interleave mid-write when they share one.
final class FileWriter: Writer, Sendable {
  private let handle: Mutex<FileHandle>

  init(_ handle: FileHandle) {
    self.handle = Mutex(handle)
  }

  func write(_ data: Data) throws {
    try handle.withLock { try $0.write(contentsOf: data) }
  }

  func close() throws {
    try handle.withLock { try $0.close() }
  }
}
