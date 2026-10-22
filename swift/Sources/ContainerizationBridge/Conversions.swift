//===----------------------------------------------------------------------===//
// Values converted between Rust's types and Swift's.
//
// Rust lends collections by length and index, since swift-bridge can't return
// them whole; the Rust side is `bridge::accessors`. Swift hands back strings
// and bytes as Rust's `Vec`s, and an optional result as `Absent` for `nil`.
//===----------------------------------------------------------------------===//

import Containerization
import Foundation

// MARK: Rust to Swift

/// A list Rust lends by index: `xLen`, `xAt`.
func list<Element>(_ count: UInt, _ element: (UInt) throws -> Element) rethrows -> [Element] {
  try (0..<count).map(element)
}

/// A list of strings Rust lends by index.
func strings(_ count: UInt, _ element: (UInt) -> RustStr) -> [String] {
  list(count) { element($0).toString() }
}

/// Rust's `Vec<String>` as strings.
func strings(_ vec: RustVec<RustString>) -> [String] {
  vec.map { $0.as_str().toString() }
}

/// A map Rust lends by index: `xLen`, `xKeyAt`, `xValueAt`. Keys that differ
/// in Rust can be equal in Swift, which compares strings by canonical
/// equivalence, and then the last one wins.
func dictionary(_ count: UInt, _ key: (UInt) -> RustStr, _ value: (UInt) -> RustStr) -> [String: String] {
  Dictionary(list(count) { (key($0).toString(), value($0).toString()) }, uniquingKeysWith: { _, last in last })
}

/// A Rust map, crossing as its keys and its values in the same order.
func dictionary(_ keys: RustVec<RustString>, _ values: RustVec<RustString>) -> [String: String] {
  Dictionary(zip(strings(keys), strings(values)), uniquingKeysWith: { _, last in last })
}

/// A Rust `usize` as Swift's `Int`, which can't hold one above `Int.max`.
func int(_ value: UInt, _ what: String) throws -> Int {
  guard let int = Int(exactly: value) else { throw BridgeError.tooLarge(what, value) }
  return int
}

/// The case Rust named by its `rawValue`. A unit test checks that Rust's raw
/// values are Swift's.
func swiftCase<T: RawRepresentable>(_ rawValue: RustStr) -> T where T.RawValue == String {
  let rawValue = rawValue.toString()
  guard let value = T(rawValue: rawValue) else {
    preconditionFailure("Rust gave \(T.self) the raw value \(rawValue)")
  }
  return value
}

/// The case a Rust caller named by its `rawValue`. Unlike `swiftCase`, which
/// reads a raw value from Rust's own enum, this reads a string any caller may
/// pass, so it throws, naming `what`, when no case has it.
func parsedCase<T: RawRepresentable>(_ rawValue: String, _ what: String) throws -> T where T.RawValue == String {
  guard let value = T(rawValue: rawValue) else {
    throw BridgeError.malformed(what, rawValue)
  }
  return value
}

/// The file URL of a path Rust lends. Rust lends `nil` for a path that isn't
/// UTF-8, which a `String` can't hold, and this throws, naming `what` and the
/// path as `lossy` gives it.
func lentPath(_ path: String?, _ what: String, _ lossy: () -> RustString) throws -> URL {
  guard let path else { throw BridgeError.malformed("\(what) path", lossy().toString()) }
  return URL(filePath: path)
}

// MARK: Swift to Rust

/// Bytes or numbers as Rust's `Vec`.
func rustVec<T: Vectorizable>(_ values: some Sequence<T>) -> RustVec<T> {
  let vec = RustVec<T>()
  for value in values {
    vec.push(value: value)
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

/// A mount's `RuntimeOptions`, as the bridge's kind and the options it carries.
func runtimeKind(_ options: Containerization.Mount.RuntimeOptions) -> (RuntimeKind, [String]) {
  switch options {
  case .virtioblk(let options): (.Virtioblk, options)
  case .virtiofs(let options): (.Virtiofs, options)
  case .shared: (.Shared, [])
  case .any(let options): (.Generic, options)
  }
}
