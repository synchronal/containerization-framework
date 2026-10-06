//===----------------------------------------------------------------------===//
// Rust's value types read into Containerization's, and Containerization's
// filled back into Rust's.
//
// Rust to Swift reads the opaque Rust values (`RustMount`, ...) once, on the
// calling thread, into `Sendable` types the async work can carry. Swift to Rust
// fills a value Rust hands over: the configuration `ContainerManager.create`
// seeds for Rust to change.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationExtras
import ContainerizationOCI
import ContainerizationOS
import Foundation
// For `FilePermissions` (relayed socket mode).
import SystemPackage

// MARK: Rust to Swift

extension Containerization.Mount {
  init(_ mount: RustMountRef) {
    let options = strings(mount.runtimeOptionsLen(), mount.runtimeOptionsAt)
    let runtimeOptions: RuntimeOptions =
      switch mount.runtimeKind() {
      case .Virtioblk: .virtioblk(options)
      case .Virtiofs: .virtiofs(options)
      case .Shared: .shared
      case .Generic: .any(options)
      }

    self.init(
      type: mount.mountType().toString(),
      source: mount.source().toString(),
      destination: mount.destination().toString(),
      options: strings(mount.optionsLen(), mount.optionsAt),
      runtimeOptions: runtimeOptions
    )
  }
}

extension UnixSocketConfiguration {
  init(_ socket: RustUnixSocketConfigurationRef) throws {
    // `CModeT` is 16-bit: refuse a mode that doesn't fit, not truncate it.
    let permissions = try socket.permissions().map { mode in
      guard let mode = CModeT(exactly: mode) else {
        throw BridgeError.malformed("socket permissions", String(mode, radix: 8))
      }
      return FilePermissions(rawValue: mode)
    }

    let direction: Direction =
      switch socket.direction() {
      case .Into: .into
      case .OutOf: .outOf
      }

    self.init(
      source: URL(filePath: socket.source().toString()),
      destination: URL(filePath: socket.destination().toString()),
      permissions: permissions,
      direction: direction
    )
  }
}

extension NATInterface {
  init(_ interface: RustNatInterfaceRef) throws {
    self.init(
      ipv4Address: try CIDRv4(bridged: interface.ipv4AddressValue(), prefix: interface.ipv4Prefix()),
      ipv4Gateway: interface.ipv4Gateway().map { IPv4Address($0) },
      ipv6Address: interface.hasIpv6Address()
        ? try CIDRv6(bridged: interface.ipv6Address(), prefix: interface.ipv6Prefix()) : nil,
      ipv6Gateway: interface.hasIpv6Gateway() ? IPv6Address(interface.ipv6Gateway()) : nil,
      macAddress: interface.macAddress().map { MACAddress($0) },
      mtu: interface.mtu()
    )
  }
}

extension DNS {
  init(_ dns: RustDnsRef) {
    self.init(
      nameservers: strings(dns.nameserversLen(), dns.nameserversAt),
      domain: dns.domain()?.toString(),
      searchDomains: strings(dns.searchDomainsLen(), dns.searchDomainsAt),
      options: strings(dns.optionsLen(), dns.optionsAt)
    )
  }
}

extension Hosts.Entry {
  init(_ entry: RustHostsEntryRef) {
    self.init(
      ipAddress: entry.ipAddress().toString(),
      hostnames: strings(entry.hostnamesLen(), entry.hostnamesAt),
      comment: entry.comment()?.toString()
    )
  }
}

extension Hosts {
  init(_ hosts: RustHostsRef) {
    self.init(
      entries: list(hosts.entriesLen(), hosts.entriesAt).map(Hosts.Entry.init),
      comment: hosts.comment()?.toString()
    )
  }
}

extension ContainerizationOCI.User {
  init(_ user: RustUserRef) {
    self.init(
      uid: user.uid(),
      gid: user.gid(),
      umask: user.umask(),
      additionalGids: list(user.additionalGidsLen(), user.additionalGidsAt),
      username: user.username().toString()
    )
  }
}

extension Containerization.LinuxCapabilities {
  init(_ capabilities: RustLinuxCapabilitiesRef) throws {
    func names(_ set: CapabilitySet) throws -> [CapabilityName] {
      try list(capabilities.setLen(set)) { try CapabilityName(rawValue: capabilities.setAt(set, $0).toString()) }
    }

    self.init(
      bounding: try names(.Bounding),
      effective: try names(.Effective),
      inheritable: try names(.Inheritable),
      permitted: try names(.Permitted),
      ambient: try names(.Ambient)
    )
  }
}

/// `LinuxRLimit.Kind` and the bridge's `RlimitKind`, case for case.
var rlimitKinds: [(LinuxRLimit.Kind, RlimitKind)] {
  [
    (.addressSpace, .AddressSpace),
    (.coreFileSize, .CoreFileSize),
    (.cpuTime, .CpuTime),
    (.dataSize, .DataSize),
    (.fileSize, .FileSize),
    (.locks, .Locks),
    (.lockedMemory, .LockedMemory),
    (.messageQueue, .MessageQueue),
    (.nice, .Nice),
    (.openFiles, .OpenFiles),
    (.numberOfProcesses, .NumberOfProcesses),
    (.residentSetSize, .ResidentSetSize),
    (.realtimePriority, .RealtimePriority),
    (.realtimeTimeout, .RealtimeTimeout),
    (.signalsPending, .SignalsPending),
    (.stackSize, .StackSize),
  ]
}

extension LinuxProcessConfiguration {
  init(_ process: RustLinuxProcessConfigurationRef) throws {
    self.init(
      arguments: strings(process.argumentsLen(), process.argumentsAt),
      environmentVariables: strings(process.environmentVariablesLen(), process.environmentVariablesAt),
      workingDirectory: process.workingDirectory().toString(),
      user: ContainerizationOCI.User(process.user()),
      rlimits: list(process.rlimitsLen()) { index in
        let kind = process.rlimitKindAt(index)

        return LinuxRLimit(
          kind: rlimitKinds.first { $0.1 == kind }!.0,
          hard: process.rlimitHardAt(index),
          soft: process.rlimitSoftAt(index)
        )
      },
      noNewPrivileges: process.noNewPrivileges(),
      capabilities: try LinuxCapabilities(process.capabilities()),
      terminal: process.terminal(),
      stdin: try process.stdin().map { FileReader(try duplicate($0)) },
      stdout: try process.stdout().map { FileWriter(try duplicate($0)) },
      stderr: try process.stderr().map { FileWriter(try duplicate($0)) }
    )
  }
}

extension BootLog {
  static func from(_ bootLog: RustBootLogRef) throws -> BootLog {
    switch bootLog.kind() {
    case .File: .file(path: URL(filePath: bootLog.path().toString()), append: bootLog.append())
    case .FileHandle: .fileHandle(try duplicate(bootLog.fileHandle()))
    }
  }
}

extension LinuxContainer.Configuration {
  init(_ configuration: RustLinuxContainerConfigurationRef) throws {
    self.init()

    process = try LinuxProcessConfiguration(configuration.process())
    cpus = Int(configuration.cpus())
    memoryInBytes = configuration.memoryInBytes()
    hostname = configuration.hostname()?.toString()
    sysctl = dictionary(configuration.sysctlLen(), configuration.sysctlKeyAt, configuration.sysctlValueAt)
    interfaces = try lentInterfaces(
      configuration.interfacesLen(),
      configuration.interfaceKindAt,
      configuration.natInterfaceAt,
      configuration.vmnetInterfaceAt
    )
    sockets = try list(configuration.socketsLen()) { try UnixSocketConfiguration(configuration.socketsAt($0)) }
    mounts = list(configuration.mountsLen()) { Containerization.Mount(configuration.mountsAt($0)) }
    maskedPaths = strings(configuration.maskedPathsLen(), configuration.maskedPathsAt)
    readonlyPaths = strings(configuration.readonlyPathsLen(), configuration.readonlyPathsAt)
    dns = configuration.hasDns() ? DNS(configuration.dns()) : nil
    hosts = configuration.hasHosts() ? Hosts(configuration.hosts()) : nil
    virtualization = configuration.virtualization()
    bootLog = configuration.hasBootLog() ? try BootLog.from(configuration.bootLog()) : nil
    ociRuntimePath = configuration.ociRuntimePath()?.toString()
    seccompProfile = lentSeccompProfile(configuration.seccompMode(), configuration.seccompProfile)
    useInit = configuration.useInit()
  }
}

/// A seccomp profile Rust lends as its mode, and the custom profile when the
/// mode is `Profile`.
func lentSeccompProfile(
  _ mode: SeccompMode,
  _ profile: () -> RustLinuxSeccompRef
) -> LinuxContainer.Configuration.SeccompProfile {
  switch mode {
  case .Unconfined: .unconfined
  case .Default: .default
  case .Profile: .profile(LinuxSeccomp(profile()))
  }
}

extension LinuxPod.PodVolume {
  init(_ volume: RustPodVolumeRef) throws {
    let location = volume.location().toString()
    let source: Source =
      switch volume.sourceKind() {
      case .Nbd:
        .nbd(
          url: try URL(string: location) ?? { throw BridgeError.malformed("volume URL", location) }(),
          timeout: volume.timeout(),
          readOnly: volume.readOnly()
        )
      case .DiskImage: .diskImage(path: URL(filePath: location), readOnly: volume.readOnly())
      case .Tmpfs: .tmpfs(sizeBytes: volume.sizeBytes())
      }

    self.init(name: volume.name().toString(), source: source, format: volume.format().toString())
  }
}

extension LinuxPod.Configuration {
  init(_ configuration: RustPodConfigurationRef) throws {
    self.init()

    interfaces = try lentInterfaces(
      configuration.interfacesLen(),
      configuration.interfaceKindAt,
      configuration.natInterfaceAt,
      configuration.vmnetInterfaceAt
    )
    virtualization = configuration.virtualization()
    bootLog = configuration.hasBootLog() ? try BootLog.from(configuration.bootLog()) : nil
    shareProcessNamespace = configuration.shareProcessNamespace()
    hostname = configuration.hostname()?.toString()
    dns = configuration.hasDns() ? DNS(configuration.dns()) : nil
    hosts = configuration.hasHosts() ? Hosts(configuration.hosts()) : nil
    volumes = try list(configuration.volumesLen()) { try LinuxPod.PodVolume(configuration.volumesAt($0)) }
    ociRuntimePath = configuration.ociRuntimePath()?.toString()
    seccompProfile = lentSeccompProfile(configuration.seccompMode(), configuration.seccompProfile)
  }
}

extension LinuxPod.ContainerConfiguration {
  init(_ configuration: RustPodContainerConfigurationRef) throws {
    self.init()

    process = try LinuxProcessConfiguration(configuration.process())
    cpus = Int(configuration.cpus())
    memoryInBytes = configuration.memoryInBytes()
    hostname = configuration.hostname()?.toString()
    sysctl = dictionary(configuration.sysctlLen(), configuration.sysctlKeyAt, configuration.sysctlValueAt)
    mounts = list(configuration.mountsLen()) { Containerization.Mount(configuration.mountsAt($0)) }
    maskedPaths = strings(configuration.maskedPathsLen(), configuration.maskedPathsAt)
    readonlyPaths = strings(configuration.readonlyPathsLen(), configuration.readonlyPathsAt)
    sockets = try list(configuration.socketsLen()) { try UnixSocketConfiguration(configuration.socketsAt($0)) }
    dns = configuration.hasDns() ? DNS(configuration.dns()) : nil
    hosts = configuration.hasHosts() ? Hosts(configuration.hosts()) : nil
    seccompProfile =
      configuration.hasSeccompProfile()
      ? lentSeccompProfile(configuration.seccompMode(), configuration.seccompProfile) : nil
    useInit = configuration.useInit()
  }
}

/// The interfaces Rust lends by index, each as the conforming type it is.
func lentInterfaces(
  _ count: UInt,
  _ kind: (UInt) -> InterfaceKind,
  _ nat: (UInt) -> RustNatInterfaceRef,
  _ vmnet: (UInt) -> CzVmnetInterface
) throws -> [any Interface] {
  try list(count) { index -> any Interface in
    switch kind(index) {
    case .Nat: try NATInterface(nat(index))
    case .Vmnet: vmnet(index).interface
    }
  }
}

extension VMConfiguration {
  init(_ configuration: RustVmConfigurationRef) throws {
    self.init(
      cpus: Int(configuration.cpus()),
      memoryInBytes: configuration.memoryInBytes(),
      interfaces: try lentInterfaces(
        configuration.interfacesLen(),
        configuration.interfaceKindAt,
        configuration.natInterfaceAt,
        configuration.vmnetInterfaceAt
      ),
      mountsByID: Dictionary(
        uniqueKeysWithValues: list(configuration.mountsByIdLen()) { index in
          (
            configuration.mountsByIdKeyAt(index).toString(),
            list(configuration.mountsByIdMountsLen(index)) {
              Containerization.Mount(configuration.mountsByIdMountAt(index, $0))
            }
          )
        }
      ),
      bootLog: configuration.hasBootLog() ? try BootLog.from(configuration.bootLog()) : nil,
      nestedVirtualization: configuration.nestedVirtualization()
    )
  }
}

extension AttachedFilesystem {
  init(_ filesystem: RustAttachedFilesystemRef) {
    self.init(
      type: filesystem.filesystemType().toString(),
      source: filesystem.source().toString(),
      destination: filesystem.destination().toString(),
      options: strings(filesystem.optionsLen(), filesystem.optionsAt)
    )
  }
}

extension SystemPlatform {
  /// Decoded, since its memberwise init is internal.
  init(_ platform: RustSystemPlatformRef) throws {
    let os: OS =
      switch platform.os() {
      case .Linux: .linux
      case .Darwin: .darwin
      }
    let architecture: Architecture =
      switch platform.architecture() {
      case .Arm64: .arm64
      case .Amd64: .amd64
      }

    self = try JSONDecoder().decode(
      SystemPlatform.self,
      from: Data(#"{"os":"\#(os.rawValue)","architecture":"\#(architecture.rawValue)"}"#.utf8)
    )
  }
}

extension Kernel {
  init(_ kernel: RustKernelRef) throws {
    self.init(
      path: URL(filePath: kernel.path().toString()),
      platform: try SystemPlatform(kernel.platform()),
      commandline: CommandLine(
        kernelArgs: strings(kernel.kernelArgsLen(), kernel.kernelArgsAt),
        initArgs: strings(kernel.initArgsLen(), kernel.initArgsAt)
      )
    )
  }
}

// MARK: Swift to Rust

/// A mount's `RuntimeOptions`, as the bridge's kind and the options it carries.
func runtimeKind(_ options: Containerization.Mount.RuntimeOptions) -> (RuntimeKind, [String]) {
  switch options {
  case .virtioblk(let options): (.Virtioblk, options)
  case .virtiofs(let options): (.Virtiofs, options)
  case .shared: (.Shared, [])
  case .any(let options): (.Generic, options)
  }
}

/// `process` into Rust's, all but stdio: Swift's streams don't cross back.
func fill(_ built: RustLinuxProcessConfigurationRefMut, from process: LinuxProcessConfiguration) {
  let user = process.user
  let gids = RustVec<UInt32>()
  for gid in user.additionalGids {
    gids.push(value: gid)
  }

  let capabilities = process.capabilities

  built.setArguments(rustStrings(process.arguments))
  built.setEnvironmentVariables(rustStrings(process.environmentVariables))
  built.setWorkingDirectory(rust(process.workingDirectory))
  built.setUser(user.uid, user.gid, user.umask, gids, rust(user.username))
  built.setNoNewPrivileges(process.noNewPrivileges)
  built.setCapabilities(
    rustStrings(capabilities.bounding.map(\.description)),
    rustStrings(capabilities.effective.map(\.description)),
    rustStrings(capabilities.inheritable.map(\.description)),
    rustStrings(capabilities.permitted.map(\.description)),
    rustStrings(capabilities.ambient.map(\.description))
  )
  built.setTerminal(process.terminal)
  built.clearRlimits()

  for rlimit in process.rlimits {
    built.pushRlimit(rlimitKinds.first { $0.0 == rlimit.kind }!.1, rlimit.hard, rlimit.soft)
  }
}

/// The setters a container's and a pod's configuration share, as
/// swift-bridge generates them, so one `fill` serves both.
protocol FilledSandbox {
  func clearInterfaces()
  func pushInterface(
    _ ipv4Address: UInt32, _ ipv4Prefix: UInt8, _ ipv4Gateway: UInt32?, _ macAddress: UInt64?, _ mtu: UInt32)
  func setInterfaceIpv6Address<S: IntoRustString>(_ high: UInt64, _ low: UInt64, _ zone: S?, _ prefix: UInt8)
  func setInterfaceIpv6Gateway<S: IntoRustString>(_ high: UInt64, _ low: UInt64, _ zone: S?)
  func pushVmnetInterface(_ interface: CzVmnetInterface)
  func clearDns()
  func setDns<S: IntoRustString>(
    _ nameservers: RustVec<S>, _ domain: S?, _ searchDomains: RustVec<S>, _ options: RustVec<S>)
  func clearHosts()
  func setHosts<S: IntoRustString>(_ comment: S?)
  func pushHostsEntry<S: IntoRustString>(_ ipAddress: S, _ hostnames: RustVec<S>, _ comment: S?)
  func clearBootLog()
  func setBootLogFile<S: IntoRustString>(_ path: S, _ append: Bool)
  func setBootLogFileHandle(_ descriptor: Int32)
  func setSeccompProfile(_ mode: SeccompMode, _ profile: CzOutcome)
}

extension RustLinuxContainerConfigurationRefMut: FilledSandbox {}
extension RustPodConfigurationRefMut: FilledSandbox {}

/// `configuration` into Rust's, replacing whatever it held.
func fill(_ built: RustLinuxContainerConfigurationRefMut, from configuration: LinuxContainer.Configuration) {
  fill(built.processMut(), from: configuration.process)
  built.setCpus(UInt32(configuration.cpus))
  built.setMemoryInBytes(configuration.memoryInBytes)
  built.setHostname(rust(configuration.hostname))
  built.setMaskedPaths(rustStrings(configuration.maskedPaths))
  built.setReadonlyPaths(rustStrings(configuration.readonlyPaths))
  built.setVirtualization(configuration.virtualization)
  built.setOciRuntimePath(rust(configuration.ociRuntimePath))
  built.setUseInit(configuration.useInit)
  built.clearSysctl()
  built.clearSockets()
  built.clearMounts()

  for (key, value) in configuration.sysctl.sorted(by: { $0.key < $1.key }) {
    built.insertSysctl(rust(key), rust(value))
  }

  fill(
    built,
    interfaces: configuration.interfaces,
    dns: configuration.dns,
    hosts: configuration.hosts,
    bootLog: configuration.bootLog,
    seccompProfile: configuration.seccompProfile
  )

  for socket in configuration.sockets {
    built.pushSocket(
      rust(socket.source.path(percentEncoded: false)),
      rust(socket.destination.path(percentEncoded: false)),
      socket.permissions.map { UInt32($0.rawValue) },
      socket.direction == .into ? .Into : .OutOf
    )
  }

  for mount in configuration.mounts {
    let (kind, options) = runtimeKind(mount.runtimeOptions)

    built.pushMount(
      rust(mount.type),
      rust(mount.source),
      rust(mount.destination),
      rustStrings(mount.options),
      kind,
      rustStrings(options)
    )
  }
}

/// `configuration` into Rust's, replacing whatever it held.
func fill(_ built: RustPodConfigurationRefMut, from configuration: LinuxPod.Configuration) {
  built.setVirtualization(configuration.virtualization)
  built.setShareProcessNamespace(configuration.shareProcessNamespace)
  built.setHostname(rust(configuration.hostname))
  built.setOciRuntimePath(rust(configuration.ociRuntimePath))
  built.clearVolumes()

  fill(
    built,
    interfaces: configuration.interfaces,
    dns: configuration.dns,
    hosts: configuration.hosts,
    bootLog: configuration.bootLog,
    seccompProfile: configuration.seccompProfile
  )

  for volume in configuration.volumes {
    let name = rust(volume.name)
    let format = rust(volume.format)

    switch volume.source {
    case .nbd(let url, let timeout, let readOnly):
      built.pushVolume(name, format, .Nbd, rust(url.absoluteString), timeout, readOnly, nil)
    case .diskImage(let path, let readOnly):
      built.pushVolume(name, format, .DiskImage, rust(path.path(percentEncoded: false)), nil, readOnly, nil)
    case .tmpfs(let sizeBytes):
      built.pushVolume(name, format, .Tmpfs, rust(""), nil, false, sizeBytes)
    }
  }
}

/// The parts a container's and a pod's configuration share, into Rust's,
/// replacing whatever it held.
private func fill(
  _ built: some FilledSandbox,
  interfaces: [any Interface],
  dns: DNS?,
  hosts: Hosts?,
  bootLog: BootLog?,
  seccompProfile: LinuxContainer.Configuration.SeccompProfile
) {
  built.clearInterfaces()
  built.clearDns()
  built.clearHosts()
  built.clearBootLog()

  // Any other conforming type crosses as a `NATInterface` of its fields.
  for interface in interfaces {
    if let interface = interface as? VmnetNetwork.Interface {
      built.pushVmnetInterface(CzVmnetInterface(interface))
      continue
    }

    built.pushInterface(
      interface.ipv4Address.address.value,
      interface.ipv4Address.prefix.length,
      interface.ipv4Gateway?.value,
      interface.macAddress?.value,
      interface.mtu
    )

    if let address = interface.ipv6Address {
      built.setInterfaceIpv6Address(
        UInt64(address.address.value >> 64),
        UInt64(truncatingIfNeeded: address.address.value),
        rust(address.address.zone),
        address.prefix.length
      )
    }

    if let gateway = interface.ipv6Gateway {
      built.setInterfaceIpv6Gateway(
        UInt64(gateway.value >> 64),
        UInt64(truncatingIfNeeded: gateway.value),
        rust(gateway.zone)
      )
    }
  }

  if let dns {
    built.setDns(
      rustStrings(dns.nameservers),
      rust(dns.domain),
      rustStrings(dns.searchDomains),
      rustStrings(dns.options)
    )
  }

  if let hosts {
    built.setHosts(rust(hosts.comment))

    for entry in hosts.entries {
      built.pushHostsEntry(rust(entry.ipAddress), rustStrings(entry.hostnames), rust(entry.comment))
    }
  }

  if let bootLog {
    switch representation(of: bootLog) {
    case .file(let path, let append): built.setBootLogFile(rust(path.path(percentEncoded: false)), append)
    case .fileHandle(let handle): built.setBootLogFileHandle(handle.fileDescriptor)
    }
  }

  switch seccompProfile {
  case .unconfined: built.setSeccompProfile(.Unconfined, CzOutcome.holding(nil))
  case .default: built.setSeccompProfile(.Default, CzOutcome.holding(nil))
  case .profile(let profile): built.setSeccompProfile(.Profile, CzOutcome.holding(profile))
  }
}

/// What a `BootLog` writes to. Containerization keeps it internal, so it is
/// read by reflection.
private enum BootLogRepresentation {
  case file(URL, Bool)
  case fileHandle(FileHandle)
}

private func representation(of bootLog: BootLog) -> BootLogRepresentation {
  let base = Mirror(reflecting: bootLog).children.first { $0.label == "base" }!.value
  let representation = Mirror(reflecting: base).children.first!

  switch representation.value {
  case let (path, append) as (URL, Bool):
    return .file(path, append)
  case let handle as FileHandle:
    return .fileHandle(handle)
  default:
    preconditionFailure("BootLog holds \(representation.label ?? "something") this bridge doesn't know")
  }
}
