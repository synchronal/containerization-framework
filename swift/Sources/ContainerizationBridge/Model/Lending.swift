//===----------------------------------------------------------------------===//
// How Rust lends the model: collections by length and index, since
// swift-bridge can't return them whole. The Rust side is `bridge::accessors`.
//===----------------------------------------------------------------------===//

/// A list Rust lends by index: `xLen`, `xAt`.
func list<Element>(_ count: UInt, _ element: (UInt) throws -> Element) rethrows -> [Element] {
  try (0..<count).map(element)
}

/// A list of strings Rust lends by index.
func strings(_ count: UInt, _ element: (UInt) -> RustStr) -> [String] {
  list(count) { element($0).toString() }
}

/// A map Rust lends by index: `xLen`, `xKeyAt`, `xValueAt`.
func dictionary(_ count: UInt, _ key: (UInt) -> RustStr, _ value: (UInt) -> RustStr) -> [String: String] {
  Dictionary(uniqueKeysWithValues: list(count) { (key($0).toString(), value($0).toString()) })
}
