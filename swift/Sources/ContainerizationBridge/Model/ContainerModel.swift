//===----------------------------------------------------------------------===//
// Rust's container values read into Containerization's, and Containerization's
// filled back into Rust's: mounts, users, capabilities, rlimits, process
// configurations, and the container's and pod's configurations. The Rust side
// is `bridge::containers` and `bridge::accessors::container`.
//
// Rust to Swift reads the opaque Rust values (`RustMount`, ...) once, on the
// calling thread, into `Sendable` types the async work can carry. Swift to Rust
// fills a value Rust hands over, such as the configuration
// `ContainerManager.create` seeds for Rust to change.
//
// The network values are in `NetworkModel.swift`, and the VM's in
// `VMModel.swift`.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationExtras
import ContainerizationOCI
import ContainerizationOS
import Foundation

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

extension FilesystemOperation {
  /// The operation the bridge's `FilesystemOperationKind` names.
  init(_ kind: FilesystemOperationKind) {
    self =
      switch kind {
      case .Freeze: .freeze
      case .Thaw: .thaw
      case .Trim: .trim
      }
  }
}

/// `LinuxRLimit.Kind` and the bridge's `RlimitKind`, case for case.
///
/// The generated `RlimitKind` isn't `Sendable`, though it has no payloads, and
/// nothing mutates this constant.
nonisolated(unsafe) let rlimitKinds: [(LinuxRLimit.Kind, RlimitKind)] = [
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

extension LinuxProcessConfiguration {
  init(_ process: RustLinuxProcessConfigurationRef) throws {
    let stdout = try process.stdout().map { FileWriter(try duplicate($0)) }
    let stderr =
      process.stderr() == process.stdout() ? stdout : try process.stderr().map { FileWriter(try duplicate($0)) }

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
      stdout: stdout,
      stderr: stderr
    )
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
    sockets = list(configuration.socketsLen()) { configuration.socketsAt($0).configuration }
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
    unified = dictionary(configuration.unifiedLen(), configuration.unifiedKeyAt, configuration.unifiedValueAt)
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
    let location = volume.location()?.toString()
    let source: Source =
      switch volume.sourceKind() {
      case .Nbd:
        .nbd(
          url: try location.flatMap { URL(string: $0) }
            ?? { throw BridgeError.malformed("volume URL", volume.lossyLocation().toString()) }(),
          timeout: volume.timeout(),
          readOnly: volume.readOnly()
        )
      case .DiskImage:
        .diskImage(path: try lentPath(location, "volume", volume.lossyLocation), readOnly: volume.readOnly())
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
    sockets = list(configuration.socketsLen()) { configuration.socketsAt($0).configuration }
    dns = configuration.hasDns() ? DNS(configuration.dns()) : nil
    hosts = configuration.hasHosts() ? Hosts(configuration.hosts()) : nil
    seccompProfile =
      configuration.hasSeccompProfile()
      ? lentSeccompProfile(configuration.seccompMode(), configuration.seccompProfile) : nil
    useInit = configuration.useInit()
    unified = dictionary(configuration.unifiedLen(), configuration.unifiedKeyAt, configuration.unifiedValueAt)
  }
}

// MARK: Swift to Rust

/// `process` into Rust's, all but stdio: Swift's streams don't cross back.
func fill(_ built: RustLinuxProcessConfigurationRefMut, from process: LinuxProcessConfiguration) {
  let user = process.user
  let capabilities = process.capabilities

  built.setArguments(rustStrings(process.arguments))
  built.setEnvironmentVariables(rustStrings(process.environmentVariables))
  built.setWorkingDirectory(rust(process.workingDirectory))
  built.setUser(user.uid, user.gid, user.umask, rustVec(user.additionalGids), rust(user.username))
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
  built.clearUnified()
  built.clearSockets()
  built.clearMounts()

  for (key, value) in configuration.sysctl.sorted(by: { $0.key < $1.key }) {
    built.insertSysctl(rust(key), rust(value))
  }

  for (key, value) in configuration.unified.sorted(by: { $0.key < $1.key }) {
    built.insertUnified(rust(key), rust(value))
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
    built.pushSocket(CzUnixSocketConfiguration(socket))
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
    case .fileHandle(let handle): built.setBootLogFileHandle(lentDescriptor(handle))
    }
  }

  switch seccompProfile {
  case .unconfined: built.setSeccompProfile(.Unconfined, CzOutcome.holding(nil))
  case .default: built.setSeccompProfile(.Default, CzOutcome.holding(nil))
  case .profile(let profile): built.setSeccompProfile(.Profile, CzOutcome.holding(profile))
  }
}

// A `Mount` and an `ExitStatus`, field by field: Rust builds its own.
extension CzOutcome {
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

  func exitCode() -> Int32 {
    (taken() as ExitStatus).exitCode
  }

  /// Seconds since 1970.
  func exitedAt() -> Double {
    (taken() as ExitStatus).exitedAt.timeIntervalSince1970
  }
}
