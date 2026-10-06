//===----------------------------------------------------------------------===//
// Containerization is async throughout; the bridge is not.
//===----------------------------------------------------------------------===//

import Foundation

/// Runs an async body in a detached task and blocks until it finishes.
///
/// Rust calls in on its own threads, never on Swift's cooperative pool, so
/// blocking here cannot deadlock the executor the body runs on.
///
/// The task releases `body` before it signals, so nothing holds it after this
/// returns and a caller may pass a closure from `withoutActuallyEscaping`.
func blocking<T>(_ body: @escaping @Sendable () async throws -> T) throws -> T {
  let semaphore = DispatchSemaphore(value: 0)
  nonisolated(unsafe) var outcome: Result<T, any Error>?
  nonisolated(unsafe) var pending: (@Sendable () async throws -> T)? = body

  Task.detached {
    if let body = pending.take() {
      do {
        outcome = .success(try await body())
      } catch {
        outcome = .failure(error)
      }
    }
    semaphore.signal()
  }

  semaphore.wait()

  guard let outcome else {
    throw BridgeError.noOutcome
  }

  return try outcome.get()
}
