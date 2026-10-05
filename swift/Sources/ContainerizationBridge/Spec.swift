//===----------------------------------------------------------------------===//
// ContainerizationOCI's runtime spec: the seccomp profiles, `Bundle`, and the
// conversions Containerization's types make to it.
//
// A `Bundle` crosses as its path. Its only stored property is that URL, and
// its initializer from one is private.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationOCI
import Foundation

func processFromImageConfig(config: RustImageConfig) -> CzOutcome {
  let config = ImageConfig(config)

  return CzOutcome { ContainerizationOCI.Process(from: config) }
}

func processDescription(process: RustProcess) -> CzOutcome {
  let process = ContainerizationOCI.Process(process)

  return CzOutcome { process.description }
}

func hookDescription(hook: RustHook) -> CzOutcome {
  let hook = Hook(hook)

  return CzOutcome { hook.description }
}

func decodeLinuxSeccomp(data: RustVec<UInt8>) -> CzOutcome {
  let data = Data(data.map { $0 })

  return CzOutcome { try LinuxSeccomp.decode(from: data) }
}

func defaultSeccompProfile(
  hasCapabilities: Bool,
  capabilities: RustOciLinuxCapabilities,
  arch: RustStr
) -> CzOutcome {
  let capabilities = hasCapabilities ? ContainerizationOCI.LinuxCapabilities(capabilities) : nil
  let arch: Arch = swiftCase(arch)

  return CzOutcome { LinuxSeccomp.defaultProfile(capabilities: capabilities, arch: arch) }
}

func currentArch() -> CzOutcome {
  CzOutcome { absent(Arch.current?.rawValue) }
}

func currentVerifiedArch() -> CzOutcome {
  CzOutcome { try Arch.currentVerified().rawValue }
}

func currentRuntimeSpecVersion() -> CzOutcome {
  CzOutcome { RuntimeSpecVersion.current }
}

func linuxRLimitToOCI(rlimit: RustLinuxRLimit) -> CzOutcome {
  let kind = rlimit.kind()
  let rlimit = LinuxRLimit(kind: rlimitKinds.first { $0.1 == kind }!.0, hard: rlimit.hard(), soft: rlimit.soft())

  return CzOutcome { rlimit.toOCI() }
}

func linuxCapabilitiesToOCI(capabilities: RustLinuxCapabilities) -> CzOutcome {
  CzOutcome { try Containerization.LinuxCapabilities(capabilities).toOCI() }
}

func systemPlatformOCIPlatform(platform: RustSystemPlatform) -> CzOutcome {
  CzOutcome { try SystemPlatform(platform).ociPlatform() }
}

// MARK: Bundle

extension ContainerizationOCI.Bundle {
  /// The bundle at `path`, unchecked, as Swift's private `init(path:)` would
  /// make it.
  init(bridged path: RustStr) {
    self = unsafeBitCast(URL(filePath: path.toString()), to: ContainerizationOCI.Bundle.self)
  }

  var bridgedPath: String {
    path.path(percentEncoded: false)
  }
}

func createBundle(path: RustStr, spec: RustSpec) -> CzOutcome {
  let path = URL(filePath: path.toString())
  let spec = Spec(spec)

  return CzOutcome { try ContainerizationOCI.Bundle.create(path: path, spec: spec).bridgedPath }
}

func createBundleFromData(path: RustStr, spec: RustVec<UInt8>) -> CzOutcome {
  let path = URL(filePath: path.toString())
  let spec = Data(spec.map { $0 })

  return CzOutcome { try ContainerizationOCI.Bundle.create(path: path, spec: spec).bridgedPath }
}

func loadBundle(path: RustStr) -> CzOutcome {
  let path = URL(filePath: path.toString())

  return CzOutcome { try ContainerizationOCI.Bundle.load(path: path).bridgedPath }
}

func bundleConfigPath(path: RustStr) -> CzOutcome {
  let bundle = ContainerizationOCI.Bundle(bridged: path)

  return CzOutcome { bundle.configPath.path(percentEncoded: false) }
}

func bundleRootfsPath(path: RustStr) -> CzOutcome {
  let bundle = ContainerizationOCI.Bundle(bridged: path)

  return CzOutcome { bundle.rootfsPath.path(percentEncoded: false) }
}

func deleteBundle(path: RustStr) -> CzOutcome {
  let bundle = ContainerizationOCI.Bundle(bridged: path)

  return CzOutcome { try bundle.delete() }
}

func bundleLoadConfig(path: RustStr) -> CzOutcome {
  let bundle = ContainerizationOCI.Bundle(bridged: path)

  return CzOutcome { try bundle.loadConfig() }
}

// MARK: For unit tests

/// The raw values of the Swift enum `name`, in the order of Rust's `ALL`.
func rawValues(name: RustStr) -> RustVec<RustString> {
  let name = name.toString()
  let rawValues: [String] =
    switch name {
    case "LinuxNamespaceType":
      [LinuxNamespaceType.pid, .network, .uts, .mount, .ipc, .user, .cgroup].map(\.rawValue)
    case "LinuxPersonalityDomain":
      [LinuxPersonalityDomain.perLinux, .perLinux32].map(\.rawValue)
    case "LinuxSeccompFlag":
      [LinuxSeccompFlag.flagLog, .flagSpecAllow, .flagWaitKillableRecv].map(\.rawValue)
    case "Arch":
      [
        Arch.archX86, .archX86_64, .archX32, .archARM, .archAARCH64, .archMIPS, .archMIPS64, .archMIPS64N32,
        .archMIPSEL, .archMIPSEL64, .archMIPSEL64N32, .archPPC, .archPPC64, .archPPC64LE, .archS390, .archS390X,
        .archPARISC, .archPARISC64, .archRISCV64,
      ].map(\.rawValue)
    case "LinuxSeccompAction":
      [
        LinuxSeccompAction.actKill, .actKillProcess, .actKillThread, .actTrap, .actErrno, .actTrace, .actAllow,
        .actLog, .actNotify,
      ].map(\.rawValue)
    case "LinuxSeccompOperator":
      [
        LinuxSeccompOperator.opNotEqual, .opLessThan, .opLessEqual, .opEqualTo, .opGreaterEqual, .opGreaterThan,
        .opMaskedEqual,
      ].map(\.rawValue)
    case "ContainerState":
      [ContainerState.creating, .created, .running, .stopped].map(\.rawValue)
    default:
      preconditionFailure("no enum named \(name)")
    }

  return rustStrings(rawValues)
}

func seccompFdName() -> String {
  ContainerizationOCI.seccompFdName
}

/// What the initializer of the Swift type `name` makes with no arguments.
func defaultValue(name: RustStr) -> CzOutcome {
  let name = name.toString()
  let value: Any =
    switch name {
    case "Spec": Spec()
    case "Process": ContainerizationOCI.Process()
    case "LinuxCapabilities": ContainerizationOCI.LinuxCapabilities()
    case "Linux": Linux()
    case "LinuxResources": LinuxResources()
    case "LinuxMemory": LinuxMemory()
    case "LinuxCPU": LinuxCPU()
    default: preconditionFailure("no type named \(name)")
    }

  return CzOutcome { value }
}
