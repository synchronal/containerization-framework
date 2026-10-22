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
  /// closed (EIO) or a reset socket. Swift cannot catch that exception, which
  /// would abort the process, and with it the VM and every other process in
  /// it.
  func stream() -> AsyncStream<Data> {
    let descriptor = handle.fileDescriptor
    let buffer = ReadBuffer()

    return .init { continuation in
      continuation.onTermination = { [weak self] _ in
        self?.handle.readabilityHandler = nil
      }

      handle.readabilityHandler = { handle in
        let count = buffer.bytes.withUnsafeMutableBytes { read(descriptor, $0.baseAddress, $0.count) }
        let error = errno

        // The handler fires again while the descriptor is still readable.
        if count < 0, error == EAGAIN || error == EINTR {
          return
        }

        // Zero is end of file, and below zero a read that failed. Either way
        // Containerization closes the guest's stdin once the stream finishes.
        guard count > 0 else {
          handle.readabilityHandler = nil
          continuation.finish()
          return
        }

        continuation.yield(Data(buffer.bytes[..<count]))
      }
    }
  }
}

/// The buffer a `FileReader` reuses for every read of its stream.
///
/// Unchecked: only the readability handler touches it, on one queue at a
/// time, and it yields a copy of what it read.
private final class ReadBuffer: @unchecked Sendable {
  var bytes = [UInt8](repeating: 0, count: 64 * 1024)
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

/// The `(inout Value) throws -> Void` closure a Swift API takes, made from
/// Rust's closure. It fills `seed` with the value Swift seeded, lets Rust's
/// closure change it through `call`, and takes back what `make` builds from
/// it. A `seed` that Rust already filled takes no `fill`.
func configuring<Seed, Value>(
  _ seed: Seed,
  fill: @escaping (Seed, Value) -> Void = { _, _ in },
  call: @escaping (Seed) -> Bool,
  make: @escaping (Seed) throws -> Value
) -> @Sendable (inout Value) throws -> Void {
  // Not `Sendable`, but only the closure below uses them, and each Swift API
  // that takes it calls it once, while `blocking` waits.
  nonisolated(unsafe) let seed = seed
  nonisolated(unsafe) let fill = fill
  nonisolated(unsafe) let call = call
  nonisolated(unsafe) let make = make

  return { value in
    fill(seed, value)
    if call(seed) { throw BridgeError.bodyFailed }
    value = try make(seed)
  }
}

/// The closure a container's or a pod's `exec` takes. It is `configuring`'s,
/// keeping the writers of the configuration it makes in `output`.
func configured(
  _ seed: RustLinuxProcessConfiguration,
  by configuration: RustConfigureProcess,
  keeping output: Output
) -> @Sendable (inout LinuxProcessConfiguration) throws -> Void {
  let configure = configuring(
    seed,
    fill: { fill($0, from: $1) },
    call: { configuration.call($0) },
    make: { try LinuxProcessConfiguration($0) }
  )

  return { config in
    try configure(&config)
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
