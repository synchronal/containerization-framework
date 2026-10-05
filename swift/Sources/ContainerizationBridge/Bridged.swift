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
import ContainerizationArchive
import ContainerizationError
import ContainerizationExtras
import ContainerizationOCI
import Foundation

/// Unchecked: Rust reads an outcome once, on the thread it was returned to.
/// Public because swift-bridge's glue for a Rust method that takes one is.
public final class CzOutcome: @unchecked Sendable {
  private let failure: String?
  private let failureCode: String?
  private let value: Any?

  /// Runs `body`, keeping what it returns or the message of what it throws,
  /// and the code of a thrown `ContainerizationError`.
  init(_ body: () throws -> Any) {
    do {
      value = try body()
      failure = nil
      failureCode = nil
    } catch {
      value = nil
      failure = "\(error)"
      failureCode = (error as? ContainerizationError)?.code.description
    }
  }

  /// An outcome holding part of another's value, or `Absent` for `nil`, for
  /// Rust to read with the same getters. (Not an `init`: a trailing closure
  /// could bind to its `Any?`.)
  static func holding(_ part: Any?) -> CzOutcome {
    CzOutcome { absent(part) }
  }

  func error() -> String? {
    failure
  }

  /// A thrown `ContainerizationError`'s `code.description`.
  func errorCode() -> String? {
    failureCode
  }

  func taken<T>() -> T {
    cast(value)
  }

  func cast<T>(_ held: Any?) -> T {
    guard let held = held as? T else {
      preconditionFailure("Rust took a \(T.self) from an outcome holding \(String(describing: held))")
    }
    return held
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
  func archiveWriter() -> CzArchiveWriter { taken() }
  func archiveReader() -> CzArchiveReader { taken() }

  // An archive entry, alone or with the data or reader it was read with.

  func writeEntry() -> CzWriteEntry {
    switch value {
    case let (entry, _) as (WriteEntry, Data): CzWriteEntry(entry)
    case let (entry, _) as (WriteEntry, ArchiveEntryReader): CzWriteEntry(entry)
    default: taken()
    }
  }

  func entryData() -> RustVec<UInt8> {
    rustBytes((taken() as (WriteEntry, Data)).1)
  }

  func archiveEntryReader() -> CzArchiveEntryReader {
    CzArchiveEntryReader((taken() as (WriteEntry, ArchiveEntryReader)).1)
  }

  /// A held `[String: Data]`'s keys, and the value of one.
  func dataMapKeys() -> RustVec<RustString> {
    rustStrings((taken() as [String: Data]).keys)
  }

  func dataMapValue(key: RustStr) -> RustVec<UInt8> {
    rustBytes((taken() as [String: Data])[key.toString()] ?? Data())
  }

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

  // Addresses, field by field: Rust builds its own.

  func boolean() -> Bool { taken() }

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

  /// Whether the address held, alone or in a CIDR block, is IPv6.
  func isIPv6() -> Bool {
    heldAddress() is IPv6Address
  }

  func ipv4Value() -> UInt32 {
    (cast(heldAddress()) as IPv4Address).value
  }

  func ipv6High() -> UInt64 {
    UInt64((cast(heldAddress()) as IPv6Address).value >> 64)
  }

  func ipv6Low() -> UInt64 {
    UInt64(truncatingIfNeeded: (cast(heldAddress()) as IPv6Address).value)
  }

  func ipv6Zone() -> String? {
    (cast(heldAddress()) as IPv6Address).zone
  }

  func prefixLength() -> UInt8 {
    switch value {
    case let cidr as CIDRv4: cidr.prefix.length
    case let cidr as CIDRv6: cidr.prefix.length
    case let cidr as CIDR: cidr.prefix.length
    default: (taken() as Prefix).length
    }
  }

  func macValue() -> UInt64 {
    (taken() as MACAddress).value
  }

  func wideHigh() -> UInt64 {
    UInt64((taken() as UInt128) >> 64)
  }

  func wideLow() -> UInt64 {
    UInt64(truncatingIfNeeded: taken() as UInt128)
  }

  /// The `IPv4Address` or `IPv6Address` held, alone or in an `IPAddress`, a
  /// CIDR block or a `CIDR`.
  private func heldAddress() -> Any? {
    switch value {
    case let cidr as CIDRv4: cidr.address
    case let cidr as CIDRv6: cidr.address
    case let cidr as CIDR: unwrapped(cidr.address)
    case let address as IPAddress: unwrapped(address)
    default: value
    }
  }

  private func unwrapped(_ address: IPAddress) -> Any {
    switch address {
    case .v4(let address): address
    case .v6(let address): address
    }
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

  /// A `Data?`, or a `[UInt8]`.
  func bytes() -> RustVec<UInt8> {
    rustBytes((value as? [UInt8]).map { Data($0) } ?? (taken() as Data?) ?? Data())
  }
}

/// Bytes as Rust's `Vec<u8>`.
func rustBytes(_ bytes: Data) -> RustVec<UInt8> {
  let vec = RustVec<UInt8>()
  for byte in bytes {
    vec.push(value: byte)
  }
  return vec
}

/// What an outcome holds for a `nil` result, which `Any` can't hold apart from
/// a missing value.
struct Absent {}

/// An optional result, with `nil` as `Absent`.
func absent(_ value: Any?) -> Any {
  value ?? Absent()
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

/// A Rust map, crossing as its keys and its values in the same order.
func dictionary(_ keys: RustVec<RustString>, _ values: RustVec<RustString>) -> [String: String] {
  Dictionary(zip(strings(keys), strings(values)), uniquingKeysWith: { _, last in last })
}
