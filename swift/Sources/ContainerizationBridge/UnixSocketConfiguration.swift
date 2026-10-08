//===----------------------------------------------------------------------===//
// `UnixSocketConfiguration`, from Containerization. It crosses whole, not
// field by field, because Swift gives each one a private `id` when it is made
// and stops a relay by that `id`.
//===----------------------------------------------------------------------===//

import Containerization
import Foundation
// For `FilePermissions`.
import SystemPackage

/// Unchecked: the setters mutate `configuration`, and Rust's `&mut self` lets
/// only one run at a time. Public because swift-bridge's glue for a Rust
/// method that takes one is.
public final class CzUnixSocketConfiguration: @unchecked Sendable {
  var configuration: UnixSocketConfiguration

  init(_ configuration: UnixSocketConfiguration) {
    self.configuration = configuration
  }

  /// A copy, which keeps the `id`, as a copy of Swift's struct does.
  func duplicate() -> CzUnixSocketConfiguration {
    CzUnixSocketConfiguration(configuration)
  }

  func id() -> String {
    configuration.id
  }

  func source() -> String {
    configuration.source.path(percentEncoded: false)
  }

  func destination() -> String {
    configuration.destination.path(percentEncoded: false)
  }

  func permissions() -> UInt16? {
    configuration.permissions.map(\.rawValue)
  }

  func direction() -> SocketDirection {
    switch configuration.direction {
    case .into: .Into
    case .outOf: .OutOf
    }
  }

  func setSource(source: RustStr) {
    configuration.source = URL(filePath: source.toString())
  }

  func setDestination(destination: RustStr) {
    configuration.destination = URL(filePath: destination.toString())
  }

  func setPermissions(permissions: UInt16?) {
    configuration.permissions = permissions.map(FilePermissions.init(rawValue:))
  }

  func setDirection(direction: SocketDirection) {
    configuration.direction =
      switch direction {
      case .Into: .into
      case .OutOf: .outOf
      }
  }
}

/// `UnixSocketConfiguration(source:destination:)`, with its other arguments at
/// their defaults. It can't fail, but returns an outcome so that Rust's
/// stand-in elsewhere can.
func unixSocketConfiguration(source: RustStr, destination: RustStr) -> CzOutcome {
  let source = URL(filePath: source.toString())
  let destination = URL(filePath: destination.toString())

  return CzOutcome {
    CzUnixSocketConfiguration(UnixSocketConfiguration(source: source, destination: destination))
  }
}
