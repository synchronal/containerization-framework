//===----------------------------------------------------------------------===//
// ContainerizationIO's `ReadStream`, and its `dataStream`, whose `next()`
// blocks.
//===----------------------------------------------------------------------===//

import ContainerizationIO
import Foundation

func newReadStream() -> CzOutcome {
  CzOutcome { CzReadStream(ReadStream()) }
}

func readStreamWithURL(url: RustStr, bufferSize: UInt) -> CzOutcome {
  let url = URL(filePath: url.toString())

  return CzOutcome { CzReadStream(try ReadStream(url: url, bufferSize: Int(bufferSize))) }
}

func readStreamWithData(data: RustVec<UInt8>, bufferSize: UInt) -> CzOutcome {
  let data = Data(data)

  return CzOutcome { CzReadStream(ReadStream(data: data, bufferSize: Int(bufferSize))) }
}

/// For a test that compares Rust's copy with it.
func readStreamBufferSize() -> Int {
  ReadStream.bufferSize
}

/// Unchecked: Rust's `ReadStream` isn't `Sync`, so one call runs at a time.
final class CzReadStream: @unchecked Sendable {
  let stream: ReadStream

  init(_ stream: ReadStream) {
    self.stream = stream
  }

  func reset() -> CzOutcome {
    CzOutcome { try stream.reset() }
  }

  func dataStream() -> CzDataStream {
    CzDataStream(stream.dataStream)
  }
}

extension CzOutcome {
  func readStream() -> CzReadStream { taken() }
}

/// Unchecked: Rust's iterator isn't `Sync`, so one call runs at a time.
final class CzDataStream: @unchecked Sendable {
  private var iterator: AsyncStream<Data>.Iterator

  init(_ stream: AsyncStream<Data>) {
    iterator = stream.makeAsyncIterator()
  }

  /// The outcome holds the next chunk, or `Absent` at the end.
  func next() -> CzOutcome {
    // Not `Sendable`, but only the task below uses it until it finishes.
    nonisolated(unsafe) var iterator = iterator
    let chunk: Data? = (try? blocking { await iterator.next() }) ?? nil
    self.iterator = iterator

    return CzOutcome.holding(chunk)
  }
}
