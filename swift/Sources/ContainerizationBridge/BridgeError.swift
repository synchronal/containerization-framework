//===----------------------------------------------------------------------===//
// What the bridge itself fails with. Containerization's own errors pass through
// unchanged; `bridged` hands either to Rust as a message.
//===----------------------------------------------------------------------===//

enum BridgeError: Error, CustomStringConvertible {
  /// Cannot happen: the semaphore is only signalled after `outcome` is set.
  case noOutcome
  /// A value the bridge could not parse: a malformed wire string, or a
  /// setting that never made sense.
  case malformed(String, String)

  var description: String {
    switch self {
    case .noOutcome:
      return "the bridged task signalled completion without an outcome"
    case .malformed(let what, let value):
      return "malformed \(what): \(value.debugDescription)"
    }
  }
}
