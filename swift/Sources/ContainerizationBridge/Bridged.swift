//===----------------------------------------------------------------------===//
// How a throwing Swift call reaches Rust.
//
// swift-bridge 0.1.59's `Result<_, String>` from Swift miscompiles with most
// argument and value types, so a throwing method returns a `CzOutcome`
// instead: the value it made, or the message of what it threw. Rust asks
// `error`, then takes the value it expects, once.
//
// This file holds the outcome and the getters that read a shape rather than
// a type. Each type's getters are in an `extension CzOutcome` in its own file.
//
// Function names and labels are the `swift_name`s and `label`s in
// `src/bridge/mod.rs`, or else Rust's own; swift-bridge's generated shims call
// with them, so they are part of the contract.
//===----------------------------------------------------------------------===//

import ContainerizationError
import ContainerizationOS
import Foundation

/// Unchecked: Rust reads an outcome once, on the thread it was returned to.
/// Public because swift-bridge's glue for a Rust method that takes one is.
public final class CzOutcome: @unchecked Sendable {
  private let failure: String?
  private let failureCode: String?
  private let fdOpsFailure: FileDescriptorOps.Error?
  /// Internal so that a getter that reads more than one shape can switch on
  /// it from its type's file.
  let value: Any?

  /// Runs `body`, keeping what it returns or the message of what it throws,
  /// the code of a thrown `ContainerizationError`, and a thrown
  /// `FileDescriptorOps.Error`.
  init(_ body: () throws -> Any) {
    do {
      value = try body()
      failure = nil
      failureCode = nil
      fdOpsFailure = nil
    } catch {
      value = nil
      failure = "\(error)"
      failureCode = (error as? ContainerizationError)?.code.description
      fdOpsFailure = error as? FileDescriptorOps.Error
    }
  }

  /// An outcome holding part of another's value, or `Absent` for `nil`, for
  /// Rust to read with the same getters. (Not an `init`: a trailing closure
  /// could bind to its `Any?`.)
  static func holding(_ part: Any?) -> CzOutcome {
    CzOutcome { absent(part) }
  }

  /// An outcome holding what an async `body` returns, or the message of what
  /// it throws, waiting for it through `blocking`. A wrapper's `body` may
  /// capture `self`, because each wrapper is a `Sendable` final class.
  static func awaiting<T>(_ body: @escaping @Sendable () async throws -> T) -> CzOutcome {
    CzOutcome { try blocking(body) }
  }

  func error() -> String? {
    failure
  }

  /// A thrown `ContainerizationError`'s `code.description`.
  func errorCode() -> String? {
    failureCode
  }

  /// A thrown `FileDescriptorOps.Error`, or `Absent`.
  func thrownFdOpsError() -> CzOutcome {
    CzOutcome.holding(fdOpsFailure)
  }

  func taken<T>() -> T {
    cast(value)
  }

  /// The value, if it is a `T`, for a getter that reads more than one shape.
  func held<T>(_: T.Type) -> T? {
    value as? T
  }

  func cast<T>(_ held: Any?) -> T {
    guard let held = held as? T else {
      preconditionFailure("Rust took a \(T.self) from an outcome holding \(String(describing: held))")
    }
    return held
  }

  // MARK: Optionals, lists and maps

  /// Whether an optional result is there, rather than `Absent`.
  func isSome() -> Bool {
    !(value is Absent)
  }

  /// The length of a held array.
  func len() -> UInt {
    UInt((taken() as [Any]).count)
  }

  /// The `index`th element of a held array, as an outcome of its own.
  func at(index: UInt) -> CzOutcome {
    CzOutcome.holding((taken() as [Any])[Int(index)])
  }

  /// A held `[String: String]`'s keys, and its values in the same order.
  func mapKeys() -> RustVec<RustString> {
    rustStrings((taken() as [String: String]).keys)
  }

  func mapValues() -> RustVec<RustString> {
    rustStrings((taken() as [String: String]).values)
  }

  /// A held `[String: T]`'s keys, sorted.
  func entryKeys() -> RustVec<RustString> {
    rustStrings(sortedEntries().map(\.key))
  }

  /// A held `[String: T]`'s values, in the order of `entryKeys`.
  func entryValues() -> CzOutcome {
    CzOutcome.holding(sortedEntries().map(\.value))
  }

  private func sortedEntries() -> [(key: String, value: Any)] {
    (taken() as [String: Any]).sorted { $0.key < $1.key }
  }

  // MARK: Primitives

  func boolean() -> Bool { taken() }

  func int32() -> Int32 { taken() }

  func integer() -> Int64 { taken() }

  /// A `UInt64`, or the bytes freed from a delete's unlabeled
  /// `([String], UInt64)`, which Rust reads with `strings` and this. The
  /// labeled result of `cleanUpOrphanedBlobs` is rebuilt unlabeled to be
  /// that exact shape.
  func number() -> UInt64 {
    if let deleted = value as? ([String], UInt64) {
      return deleted.1
    }
    return taken()
  }

  func wideHigh() -> UInt64 {
    UInt64((taken() as UInt128) >> 64)
  }

  func wideLow() -> UInt64 {
    UInt64(truncatingIfNeeded: taken() as UInt128)
  }

  func text() -> String { taken() }

  /// A `[String]`, or the digests removed from a delete's unlabeled
  /// `([String], UInt64)`, as `number` reads it.
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

  /// A `Data` or `Data?`, as content and registry reads return, or a
  /// `[UInt8]`, as an address's `bytes` returns. A `nil` reads as empty, so
  /// Rust asks `hasBytes` first where it can be one.
  func bytes() -> RustVec<UInt8> {
    if let bytes = value as? [UInt8] {
      return rustVec(bytes)
    }
    return rustVec((taken() as Data?) ?? Data())
  }
}
