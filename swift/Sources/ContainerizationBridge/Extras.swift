//===----------------------------------------------------------------------===//
// ContainerizationExtras' `ProgressEvent` and `ProxyUtils`.
//===----------------------------------------------------------------------===//

import ContainerizationExtras
import Foundation

extension ProgressEvent {
  init(_ kind: ProgressKind, _ value: Int64) {
    self =
      switch kind {
      case .Items: .addItems(Int(value))
      case .TotalItems: .addTotalItems(Int(value))
      case .Size: .addSize(value)
      case .TotalSize: .addTotalSize(value)
      }
  }
}

func progressEventEvent(kind: ProgressKind, value: Int64) -> CzOutcome {
  let event = ProgressEvent(kind, value)

  return CzOutcome { event.event }
}

/// The outcome holds the URL's `absoluteString`, or `Absent`.
func proxyFromEnvironment(
  scheme: RustString?,
  host: RustStr,
  hasEnv: Bool,
  envKeys: RustVec<RustString>,
  envValues: RustVec<RustString>
) -> CzOutcome {
  let scheme = scheme?.toString()
  let host = host.toString()
  let env = hasEnv ? dictionary(envKeys, envValues) : nil

  return CzOutcome {
    let proxy =
      if let env {
        ProxyUtils.proxyFromEnvironment(scheme: scheme, host: host, env: env)
      } else {
        ProxyUtils.proxyFromEnvironment(scheme: scheme, host: host)
      }

    return absent(proxy?.absoluteString)
  }
}
