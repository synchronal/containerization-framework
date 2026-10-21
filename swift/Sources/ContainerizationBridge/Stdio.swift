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
/// Unchecked: `FileHandle` is not `Sendable`. Only the readability handler
/// reads from it, on one queue at a time, and the stream's termination only
/// clears that handler.
final class FileReader: ReaderStream, @unchecked Sendable {
  private let handle: FileHandle

  init(_ handle: FileHandle) {
    self.handle = handle
  }

  /// Reading stops when the stream ends, at end of file or when
  /// Containerization cancels its relay on closing stdin or deleting the
  /// process. Otherwise the handler would go on taking the caller's input
  /// until Swift frees it.
  ///
  /// A failed read ends the stream as end of file does. It reads with
  /// `read(2)` rather than `availableData`, which raises an Objective-C
  /// exception when the read fails, as it does on a pty whose terminal has
  /// closed (EIO) or a reset socket. Swift cannot catch that, so it aborted
  /// the process, and with it the VM and every other process in it.
  func stream() -> AsyncStream<Data> {
    let descriptor = handle.fileDescriptor

    return .init { continuation in
      continuation.onTermination = { [weak self] _ in
        self?.handle.readabilityHandler = nil
      }

      handle.readabilityHandler = { handle in
        var buffer = [UInt8](repeating: 0, count: 64 * 1024)
        let count = buffer.withUnsafeMutableBytes { read(descriptor, $0.baseAddress, $0.count) }

        if count < 0, errno == EAGAIN || errno == EINTR {
          return
        }

        // Zero is end of file, and below zero a read that failed. Either way
        // Containerization closes the guest's stdin once the stream finishes.
        guard count > 0 else {
          handle.readabilityHandler = nil
          continuation.finish()
          return
        }

        continuation.yield(Data(buffer[..<count]))
      }
    }
  }
}

/// A descriptor the guest writes. When stdout and stderr share a descriptor,
/// they share one writer, whose lock stops their chunks interleaving mid-write.
final class FileWriter: Writer, Sendable {
  private let handle: Mutex<FileHandle?>

  init(_ handle: FileHandle) {
    self.handle = Mutex(handle)
  }

  func write(_ data: Data) throws {
    try handle.withLock { handle in
      guard let handle else { throw BridgeError.closed }
      try handle.write(contentsOf: data)
    }
  }

  /// Closing a second time does nothing.
  func close() throws {
    try handle.withLock { handle in
      defer { handle = nil }
      try handle?.close()
    }
  }
}

/// The writers the bridge made for an `exec`ed process's stdout and stderr.
/// Containerization never closes a `Writer`, and a caller can't close the
/// bridge's duplicate, so the bridge closes them once Containerization has
/// deleted the process.
///
/// An init process's writers stay open as long as its container or pod,
/// because a stopped one can be created again and reuses its configuration.
final class Output: Sendable {
  private let writers = Mutex<[FileWriter]>([])

  init() {}

  init(_ process: LinuxProcessConfiguration) {
    keep(process)
  }

  func keep(_ process: LinuxProcessConfiguration) {
    let made = [process.stdout, process.stderr].compactMap { $0 as? FileWriter }
    writers.withLock { $0.append(contentsOf: made) }
  }

  func close() {
    for writer in writers.withLock({ $0 }) {
      try? writer.close()
    }
  }
}

/// The closure a container's or a pod's `exec` takes. It fills `seed` with the
/// configuration Swift seeded, lets Rust's closure change it, and takes it
/// back, keeping its writers in `output`.
func configured(
  _ seed: RustLinuxProcessConfiguration,
  by configuration: RustConfigureProcess,
  keeping output: Output
) -> @Sendable (inout LinuxProcessConfiguration) throws -> Void {
  // Not `Sendable`, but only this closure uses them, and an `exec` calls it
  // once, while `blocking` waits.
  nonisolated(unsafe) let seed = seed
  nonisolated(unsafe) let configuration = configuration

  return { config in
    fill(seed, from: config)
    if configuration.call(seed) { throw BridgeError.bodyFailed }
    config = try LinuxProcessConfiguration(seed)
    output.keep(config)
  }
}

/// A duplicate of a boot log descriptor, which remembers the caller's number,
/// so that a configuration read back reports the descriptor they set.
func duplicate(bootLog descriptor: Int32) throws -> FileHandle {
  let handle = try duplicate(descriptor)

  objc_setAssociatedObject(handle, &lentDescriptorKey, NSNumber(value: descriptor), .OBJC_ASSOCIATION_RETAIN)
  return handle
}

/// The caller's number for a boot log's handle.
func lentDescriptor(_ handle: FileHandle) -> Int32 {
  (objc_getAssociatedObject(handle, &lentDescriptorKey) as? NSNumber)?.int32Value ?? handle.fileDescriptor
}

// Only its address is used, as the key.
nonisolated(unsafe) private var lentDescriptorKey: UInt8 = 0
