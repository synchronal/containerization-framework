//===----------------------------------------------------------------------===//
// ContainerizationOS: `Terminal`, `CapabilityName`, `CapabilitySet`,
// `KeychainQuery`, `Sysctl` and `File`.
//
// `CapabilityName` and `CapabilitySet` cross as their `description`s.
//===----------------------------------------------------------------------===//

import ContainerizationOS
import Foundation

/// For a unit test: `CapabilityName.allCases`' descriptions.
func capabilityNameDescriptions() -> RustVec<RustString> {
  rustStrings(CapabilityName.allCases.map(\.description))
}

/// For a unit test: `CapabilityName.allCases`' `capValue`s.
func capabilityNameCapValues() -> RustVec<UInt32> {
  let vec = RustVec<UInt32>()
  for name in CapabilityName.allCases {
    vec.push(value: name.capValue)
  }
  return vec
}

/// For a unit test: each `CapabilitySet`'s description, in the order of Rust's
/// `ALL`.
func capabilitySetDescriptions() -> RustVec<RustString> {
  rustStrings(
    [ContainerizationOS.CapabilitySet.bounding, .effective, .inheritable, .permitted, .ambient].map(\.description))
}

func parseCapabilityName(rawValue: RustStr) -> CzOutcome {
  let rawValue = rawValue.toString()

  return CzOutcome { try CapabilityName(rawValue: rawValue).description }
}

func parseCapabilitySet(rawValue: RustStr) -> CzOutcome {
  let rawValue = rawValue.toString()

  // Qualified: the bridge has a `CapabilitySet` of its own.
  return CzOutcome { try ContainerizationOS.CapabilitySet(rawValue: rawValue).description }
}

func newTerminal(descriptor: Int32, setInitState: Bool) -> CzOutcome {
  CzOutcome { CzTerminal(try Terminal(descriptor: descriptor, setInitState: setInitState)) }
}

func currentTerminal() -> CzOutcome {
  CzOutcome { CzTerminal(try Terminal.current) }
}

func createTerminal(hasInitialSize: Bool, width: UInt16, height: UInt16) -> CzOutcome {
  let initialSize = hasInitialSize ? Terminal.Size(width: width, height: height) : nil

  return CzOutcome { try Terminal.create(initialSize: initialSize) }
}

/// A `Terminal`.
final class CzTerminal: Sendable {
  let terminal: Terminal

  init(_ terminal: Terminal) {
    self.terminal = terminal
  }

  /// A copy of the terminal, for a call to take.
  func duplicate() -> CzTerminal {
    CzTerminal(terminal)
  }

  func handle() -> Int32 {
    terminal.handle.fileDescriptor
  }

  func write(data: RustVec<UInt8>) -> CzOutcome {
    let data = Data(data)

    return CzOutcome { try terminal.write(data) }
  }

  func size() -> CzOutcome {
    CzOutcome { try terminal.size }
  }

  func resizeFrom(pty: CzTerminal) -> CzOutcome {
    CzOutcome { try terminal.resize(from: pty.terminal) }
  }

  func resizeSize(width: UInt16, height: UInt16) -> CzOutcome {
    CzOutcome { try terminal.resize(size: Terminal.Size(width: width, height: height)) }
  }

  func resize(width: UInt16, height: UInt16) -> CzOutcome {
    CzOutcome { try terminal.resize(width: width, height: height) }
  }

  func setraw() -> CzOutcome {
    CzOutcome { try terminal.setraw() }
  }

  func enableEcho() -> CzOutcome {
    CzOutcome { try terminal.enableEcho() }
  }

  func disableEcho() -> CzOutcome {
    CzOutcome { try terminal.disableEcho() }
  }

  func close() -> CzOutcome {
    CzOutcome { try terminal.close() }
  }

  func reset() -> CzOutcome {
    CzOutcome { try terminal.reset() }
  }

  func tryReset() {
    terminal.tryReset()
  }
}

func keychainQuerySave(
  securityDomain: RustStr,
  accessGroup: RustString?,
  hostname: RustStr,
  username: RustStr,
  password: RustStr
) -> CzOutcome {
  let securityDomain = securityDomain.toString()
  let accessGroup = accessGroup?.toString()
  let hostname = hostname.toString()
  let username = username.toString()
  let password = password.toString()

  return CzOutcome {
    try KeychainQuery().save(
      securityDomain: securityDomain,
      accessGroup: accessGroup,
      hostname: hostname,
      username: username,
      password: password
    )
  }
}

func keychainQueryDelete(securityDomain: RustStr, accessGroup: RustString?, hostname: RustStr) -> CzOutcome {
  let securityDomain = securityDomain.toString()
  let accessGroup = accessGroup?.toString()
  let hostname = hostname.toString()

  return CzOutcome {
    try KeychainQuery().delete(securityDomain: securityDomain, accessGroup: accessGroup, hostname: hostname)
  }
}

func keychainQueryGet(securityDomain: RustStr, accessGroup: RustString?, hostname: RustStr) -> CzOutcome {
  let securityDomain = securityDomain.toString()
  let accessGroup = accessGroup?.toString()
  let hostname = hostname.toString()

  return CzOutcome {
    absent(try KeychainQuery().get(securityDomain: securityDomain, accessGroup: accessGroup, hostname: hostname))
  }
}

func keychainQueryList(securityDomain: RustStr, accessGroup: RustString?) -> CzOutcome {
  let securityDomain = securityDomain.toString()
  let accessGroup = accessGroup?.toString()

  return CzOutcome { try KeychainQuery().list(securityDomain: securityDomain, accessGroup: accessGroup) }
}

func keychainQueryExists(securityDomain: RustStr, accessGroup: RustString?, hostname: RustStr) -> CzOutcome {
  let securityDomain = securityDomain.toString()
  let accessGroup = accessGroup?.toString()
  let hostname = hostname.toString()

  return CzOutcome {
    try KeychainQuery().exists(securityDomain: securityDomain, accessGroup: accessGroup, hostname: hostname)
  }
}

func sysctlByName(name: RustStr) -> CzOutcome {
  let name = name.toString()

  return CzOutcome { try Sysctl.byName(name) }
}

func fileInfoAt(path: RustStr) -> CzOutcome {
  let path = path.toString()

  return CzOutcome { CzFileInfo(try File.info(path)) }
}

/// A `FileInfo`.
final class CzFileInfo: Sendable {
  let info: FileInfo

  init(_ info: FileInfo) {
    self.info = info
  }

  func mode() -> UInt16 { info.mode }
  func uid() -> Int64 { Int64(info.uid) }
  func gid() -> Int64 { Int64(info.gid) }
  func dev() -> Int64 { Int64(info.dev) }
  func ino() -> Int64 { Int64(info.ino) }
  func size() -> Int64 { Int64(info.size) }
  func path() -> String { info.path }
  func isDirectory() -> Bool { info.isDirectory }
  func isPipe() -> Bool { info.isPipe }
  func isSocket() -> Bool { info.isSocket }
  func isLink() -> Bool { info.isLink }
  func isRegularFile() -> Bool { info.isRegularFile }
  func isBlock() -> Bool { info.isBlock }
  func isChar() -> Bool { info.isChar }
}

extension CzOutcome {
  func terminal() -> CzTerminal { taken() }
  func fileInfo() -> CzFileInfo { taken() }
  func integer() -> Int64 { taken() }

  func parentTerminal() -> CzTerminal {
    CzTerminal((taken() as (parent: Terminal, child: Terminal)).parent)
  }

  func childTerminal() -> CzTerminal {
    CzTerminal((taken() as (parent: Terminal, child: Terminal)).child)
  }

  func terminalSizeWidth() -> UInt16 {
    (taken() as Terminal.Size).width
  }

  func terminalSizeHeight() -> UInt16 {
    (taken() as Terminal.Size).height
  }

  // A `KeychainQueryResult`, field by field: Rust builds its own.

  func keychainQueryResultUsername() -> String {
    (taken() as KeychainQueryResult).username
  }

  func keychainQueryResultPassword() -> String {
    (taken() as KeychainQueryResult).password
  }

  func keychainQueryResultModifiedDate() -> Double {
    (taken() as KeychainQueryResult).modifiedDate.timeIntervalSince1970
  }

  func keychainQueryResultCreatedDate() -> Double {
    (taken() as KeychainQueryResult).createdDate.timeIntervalSince1970
  }
}
