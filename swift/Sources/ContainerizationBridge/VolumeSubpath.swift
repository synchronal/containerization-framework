//===----------------------------------------------------------------------===//
// ContainerizationOS: `VolumeSubpath`.
//
// `open(beneath:subpath:)` exists only on Linux, so it isn't bound.
//===----------------------------------------------------------------------===//

import ContainerizationOS

func volumeSubpathIsSubpathMount(options: RustVec<RustString>) -> CzOutcome {
  let options = strings(options)

  return CzOutcome { try VolumeSubpath.isSubpathMount(options: options) }
}

func volumeSubpathExtract(options: RustVec<RustString>) -> CzOutcome {
  let options = strings(options)

  return CzOutcome { try VolumeSubpath.extract(from: options) }
}

/// For a unit test: `VolumeSubpath.toMountOption(_:)`.
func volumeSubpathToMountOption(subpath: RustStr) -> String {
  VolumeSubpath.toMountOption(subpath.toString())
}

extension CzOutcome {
  // A held `extract(from:)` result, field by field.

  func volumeSubpathExtractedSubpath() -> String {
    (taken() as (subpath: String, options: [String])).subpath
  }

  func volumeSubpathExtractedOptions() -> RustVec<RustString> {
    rustStrings((taken() as (subpath: String, options: [String])).options)
  }

  // A held `VolumeSubpath.Error`: its case, and what the case carries.

  func heldVolumeSubpathErrorKind() -> VolumeSubpathErrorKind {
    switch taken() as VolumeSubpath.Error {
    case .invalid: .Invalid
    case .missing: .Missing
    case .notBind: .NotBind
    case .resolve: .Resolve
    }
  }

  /// An `invalid` or `resolve` error's subpath.
  func heldVolumeSubpathErrorSubpath() -> String {
    switch taken() as VolumeSubpath.Error {
    case .invalid(let subpath, _), .resolve(let subpath, _): subpath
    default: preconditionFailure("Rust asks for the subpath only of an invalid or unresolved subpath")
    }
  }

  /// An `invalid` error's reason.
  func heldVolumeSubpathErrorReason() -> String {
    guard case .invalid(_, let reason) = taken() as VolumeSubpath.Error else {
      preconditionFailure("Rust asks for the reason only of an invalid subpath")
    }
    return reason
  }

  /// A `missing` error's options.
  func heldVolumeSubpathErrorOptions() -> RustVec<RustString> {
    guard case .missing(let options) = taken() as VolumeSubpath.Error else {
      preconditionFailure("Rust asks for the options only of a missing subpath")
    }
    return rustStrings(options)
  }

  /// A `resolve` error's `errno`.
  func heldVolumeSubpathErrorNumber() -> Int32 {
    guard case .resolve(_, let number) = taken() as VolumeSubpath.Error else {
      preconditionFailure("Rust asks for the number only of an unresolved subpath")
    }
    return number
  }
}
