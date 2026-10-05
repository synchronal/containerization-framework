//===----------------------------------------------------------------------===//
// ContainerizationOCI's `Reference` and `ParsedDigest`.
//
// `Reference` is a class, so Rust holds a handle on it. A `ParsedDigest`
// crosses as its validated `encoded` string.
//===----------------------------------------------------------------------===//

import ContainerizationOCI
import Foundation

// MARK: Reference

extension CzOutcome {
  func reference() -> CzReference { taken() }
}

final class CzReference {
  let reference: Reference

  init(_ reference: Reference) {
    self.reference = reference
  }

  func domain() -> String? {
    reference.domain
  }

  func resolvedDomain() -> String? {
    reference.resolvedDomain
  }

  func path() -> String {
    reference.path
  }

  func tag() -> String? {
    reference.tag
  }

  func digest() -> String? {
    reference.digest
  }

  func name() -> String {
    reference.name
  }

  func description() -> String {
    reference.description
  }

  func withTag(tag: RustStr) -> CzOutcome {
    let tag = tag.toString()

    return CzOutcome { CzReference(try reference.withTag(tag)) }
  }

  func withDigest(digest: RustStr) -> CzOutcome {
    let digest = digest.toString()

    return CzOutcome { CzReference(try reference.withDigest(digest)) }
  }

  func normalize() {
    reference.normalize()
  }
}

func newReference(path: RustStr, domain: RustString?, tag: RustString?, digest: RustString?) -> CzOutcome {
  let path = path.toString()
  let domain = domain?.toString()
  let tag = tag?.toString()
  let digest = digest?.toString()

  return CzOutcome { CzReference(try Reference(path: path, domain: domain, tag: tag, digest: digest)) }
}

func parseReference(string: RustStr) -> CzOutcome {
  let string = string.toString()

  return CzOutcome { CzReference(try Reference.parse(string)) }
}

func referenceWithName(name: RustStr) -> CzOutcome {
  let name = name.toString()

  return CzOutcome { CzReference(try Reference.withName(name)) }
}

func referenceResolveDomain(domain: RustStr) -> CzOutcome {
  let domain = domain.toString()

  return CzOutcome { Reference.resolveDomain(domain: domain) }
}

// MARK: ParsedDigest

extension ParsedDigest {
  /// The `encoded` Rust holds, which Swift validated.
  init(bridged encoded: String) throws {
    try self.init(parsingPathComponent: encoded)
  }
}

extension CzOutcome {
  func parsedDigestEncoded() -> String {
    (taken() as ParsedDigest).encoded
  }
}

func parseDigest(digest: RustStr) -> CzOutcome {
  let digest = digest.toString()

  return CzOutcome { try ParsedDigest(parsing: digest) }
}

func parseDigestPathComponent(component: RustStr) -> CzOutcome {
  let component = component.toString()

  return CzOutcome { try ParsedDigest(parsingPathComponent: component) }
}

func digestIsValid(digest: RustStr) -> CzOutcome {
  let digest = digest.toString()

  return CzOutcome { ParsedDigest.isValid(digest) }
}

func digestDescription(encoded: RustStr) -> CzOutcome {
  let encoded = encoded.toString()

  return CzOutcome { try ParsedDigest(bridged: encoded).description }
}

func digestPath(encoded: RustStr, root: RustStr) -> CzOutcome {
  let encoded = encoded.toString()
  let root = URL(filePath: root.toString())

  return CzOutcome { try ParsedDigest(bridged: encoded).path(in: root).path(percentEncoded: false) }
}

func digestAlgorithm() -> String {
  ParsedDigest.algorithm
}
