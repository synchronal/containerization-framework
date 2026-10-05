//===----------------------------------------------------------------------===//
// How a throwing Swift call reaches Rust.
//
// swift-bridge 0.1.59's `Result<_, String>` from Swift miscompiles with most
// argument and value types, so a throwing method returns a `CzOutcome`
// instead: the value it made, or the message of what it threw. Rust asks
// `error`, then takes the value it expects, once.
//
// Function names and labels are the `swift_name`s and `label`s in
// `src/bridge/mod.rs`, or else Rust's own; swift-bridge's generated shims call
// with them, so they are part of the contract.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationOCI
import Foundation

/// Unchecked: Rust reads an outcome once, on the thread it was returned to.
final class CzOutcome: @unchecked Sendable {
  private let failure: String?
  private let value: Any?

  /// Runs `body`, keeping what it returns or the message of what it throws.
  init(_ body: () throws -> Any) {
    do {
      value = try body()
      failure = nil
    } catch {
      value = nil
      failure = "\(error)"
    }
  }

  func error() -> String? {
    failure
  }

  private func taken<T>() -> T {
    guard let value = value as? T else {
      preconditionFailure("Rust took a \(T.self) from an outcome holding \(String(describing: value))")
    }
    return value
  }

  func localContentStore() -> CzLocalContentStore { taken() }
  func content() -> CzContent { taken() }
  func imageStore() -> CzImageStore { taken() }
  func image() -> CzImage { taken() }
  func images() -> CzImages { taken() }
  func initImage() -> CzInitImage { taken() }
  func containerManager() -> CzContainerManager { taken() }
  func linuxContainer() -> CzLinuxContainer { taken() }
  func linuxProcess() -> CzLinuxProcess { taken() }
  func contentWriter() -> CzContentWriter { taken() }
  func ext4Reader() -> CzExt4Reader { taken() }

  func writtenSize() -> Int64 {
    (taken() as Written).size
  }

  func writtenDigest() -> String {
    (taken() as Written).digest
  }

  // A `Mount`, field by field: Rust builds its own.

  func mountType() -> String {
    (taken() as Containerization.Mount).type
  }

  func mountSource() -> String {
    (taken() as Containerization.Mount).source
  }

  func mountDestination() -> String {
    (taken() as Containerization.Mount).destination
  }

  func mountOptions() -> RustVec<RustString> {
    rustStrings((taken() as Containerization.Mount).options)
  }

  func mountRuntimeKind() -> RuntimeKind {
    runtimeKind((taken() as Containerization.Mount).runtimeOptions).0
  }

  func mountRuntimeOptions() -> RustVec<RustString> {
    rustStrings(runtimeKind((taken() as Containerization.Mount).runtimeOptions).1)
  }

  // A `Platform`, field by field: Rust builds its own.

  func platformArchitecture() -> String {
    (taken() as Platform).architecture
  }

  func platformOs() -> String {
    (taken() as Platform).os
  }

  func platformOsVersion() -> String? {
    (taken() as Platform).osVersion
  }

  func platformHasOsFeatures() -> Bool {
    (taken() as Platform).osFeatures != nil
  }

  func platformOsFeatures() -> RustVec<RustString> {
    rustStrings((taken() as Platform).osFeatures ?? [])
  }

  func platformVariant() -> String? {
    (taken() as Platform).variant
  }

  func exitCode() -> Int32 {
    (taken() as ExitStatus).exitCode
  }

  /// Seconds since 1970.
  func exitedAt() -> Double {
    (taken() as ExitStatus).exitedAt.timeIntervalSince1970
  }

  /// A `UInt64`, or the bytes a delete freed.
  func number() -> UInt64 {
    if let deleted = value as? ([String], UInt64) {
      return deleted.1
    }
    return taken()
  }

  func text() -> String { taken() }

  /// A `[String]`, or the digests a delete removed.
  func strings() -> RustVec<RustString> {
    if let deleted = value as? ([String], UInt64) {
      return rustStrings(deleted.0)
    }
    return rustStrings(taken() as [String])
  }

  /// For a `Data?`.
  func hasBytes() -> Bool {
    (taken() as Data?) != nil
  }

  func bytes() -> RustVec<UInt8> {
    let vec = RustVec<UInt8>()
    for byte in (taken() as Data?) ?? Data() {
      vec.push(value: byte)
    }
    return vec
  }
}

func rust(_ string: String) -> RustString {
  string.intoRustString()
}

func rust(_ string: String?) -> RustString? {
  string.map(rust)
}

/// Strings as Rust's `Vec<String>`.
func rustStrings(_ values: some Sequence<String>) -> RustVec<RustString> {
  let vec = RustVec<RustString>()
  for value in values {
    vec.push(value: value.intoRustString())
  }
  return vec
}

/// Rust's `Vec<String>` as strings.
func strings(_ vec: RustVec<RustString>) -> [String] {
  vec.map { $0.as_str().toString() }
}
