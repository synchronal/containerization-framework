//===----------------------------------------------------------------------===//
// `Authentication` and `KeychainHelper`, from ContainerizationOCI, and the
// `RegistryInfo` its `list` returns, from ContainerizationOS.
//===----------------------------------------------------------------------===//

import ContainerizationOCI
import ContainerizationOS
import Foundation

/// An `Authentication?`.
final class CzAuthentication: Sendable {
  let authentication: (any Authentication)?

  init(_ authentication: (any Authentication)?) {
    self.authentication = authentication
  }

  func duplicate() -> CzAuthentication {
    CzAuthentication(authentication)
  }

  func token() -> CzOutcome {
    let authentication = authentication

    return CzOutcome { try blocking { try await authentication?.token() ?? "" } }
  }
}

func basicAuthentication(username: RustStr, password: RustStr) -> CzOutcome {
  let authentication = BasicAuthentication(username: username.toString(), password: password.toString())

  return CzOutcome { CzAuthentication(authentication) }
}

func noAuthentication() -> CzOutcome {
  CzOutcome { CzAuthentication(nil) }
}

func keychainHelperLookup(securityDomain: RustStr, accessGroup: RustString?, hostname: RustStr) -> CzOutcome {
  let helper = KeychainHelper(securityDomain: securityDomain.toString(), accessGroup: accessGroup?.toString())
  let hostname = hostname.toString()

  return CzOutcome { CzAuthentication(try helper.lookup(hostname: hostname)) }
}

func keychainHelperList(securityDomain: RustStr, accessGroup: RustString?) -> CzOutcome {
  let helper = KeychainHelper(securityDomain: securityDomain.toString(), accessGroup: accessGroup?.toString())

  return CzOutcome { try helper.list() }
}

func keychainHelperDelete(securityDomain: RustStr, accessGroup: RustString?, hostname: RustStr) -> CzOutcome {
  let helper = KeychainHelper(securityDomain: securityDomain.toString(), accessGroup: accessGroup?.toString())
  let hostname = hostname.toString()

  return CzOutcome { try helper.delete(hostname: hostname) }
}

func keychainHelperSave(
  securityDomain: RustStr,
  accessGroup: RustString?,
  hostname: RustStr,
  username: RustStr,
  password: RustStr
) -> CzOutcome {
  let helper = KeychainHelper(securityDomain: securityDomain.toString(), accessGroup: accessGroup?.toString())
  let hostname = hostname.toString()
  let username = username.toString()
  let password = password.toString()

  return CzOutcome { try helper.save(hostname: hostname, username: username, password: password) }
}

extension CzOutcome {
  func authentication() -> CzAuthentication { taken() }

  // A `RegistryInfo`, field by field: Rust builds its own.

  func registryInfoHostname() -> String {
    (taken() as RegistryInfo).hostname
  }

  func registryInfoUsername() -> String {
    (taken() as RegistryInfo).username
  }

  func registryInfoModifiedDate() -> Double {
    (taken() as RegistryInfo).modifiedDate.timeIntervalSince1970
  }

  func registryInfoCreatedDate() -> Double {
    (taken() as RegistryInfo).createdDate.timeIntervalSince1970
  }
}
