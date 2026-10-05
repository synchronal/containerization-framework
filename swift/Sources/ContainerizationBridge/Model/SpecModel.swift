//===----------------------------------------------------------------------===//
// The OCI runtime spec's types: Rust's read into ContainerizationOCI's, and
// ContainerizationOCI's read back by Rust field by field. An enum crosses as
// its `rawValue`.
//===----------------------------------------------------------------------===//

import ContainerizationOCI
import Foundation

// MARK: Rust to Swift

/// The case Rust named by its `rawValue`. A unit test checks that Rust's raw
/// values are Swift's.
func swiftCase<T: RawRepresentable>(_ rawValue: RustStr) -> T where T.RawValue == String {
  let rawValue = rawValue.toString()
  guard let value = T(rawValue: rawValue) else {
    preconditionFailure("Rust gave \(T.self) the raw value \(rawValue)")
  }
  return value
}

extension Spec {
  init(_ spec: RustSpecRef) {
    self.init(
      version: spec.version().toString(),
      hooks: spec.hasHooks() ? Hooks(spec.hooks()) : nil,
      process: spec.hasProcess() ? ContainerizationOCI.Process(spec.process()) : nil,
      hostname: spec.hostname().toString(),
      domainname: spec.domainname().toString(),
      mounts: list(spec.mountsLen()) { ContainerizationOCI.Mount(spec.mountsAt($0)) },
      annotations: spec.hasAnnotations()
        ? dictionary(spec.annotationsLen(), spec.annotationKeyAt, spec.annotationValueAt) : nil,
      root: spec.hasRoot() ? Root(path: spec.rootPath().toString(), readonly: spec.rootReadonly()) : nil,
      linux: spec.hasLinux() ? Linux(spec.linux()) : nil
    )
  }
}

extension ContainerizationOCI.Process {
  init(_ process: RustProcessRef) {
    self.init(
      args: strings(process.argsLen(), process.argsAt),
      cwd: process.cwd().toString(),
      env: strings(process.envLen(), process.envAt),
      consoleSize: process.hasConsoleSize()
        ? Box(height: process.consoleHeight(), width: process.consoleWidth()) : nil,
      selinuxLabel: process.selinuxLabel().toString(),
      noNewPrivileges: process.noNewPrivileges(),
      commandLine: process.commandLine().toString(),
      oomScoreAdj: process.oomScoreAdj(),
      capabilities: process.hasCapabilities() ? ContainerizationOCI.LinuxCapabilities(process.capabilities()) : nil,
      apparmorProfile: process.apparmorProfile().toString(),
      user: ContainerizationOCI.User(process.user()),
      rlimits: list(process.rlimitsLen()) {
        POSIXRlimit(
          type: process.rlimitTypeAt($0).toString(),
          hard: process.rlimitHardAt($0),
          soft: process.rlimitSoftAt($0)
        )
      },
      terminal: process.terminal()
    )
  }
}

extension ContainerizationOCI.LinuxCapabilities {
  init(_ capabilities: RustOciLinuxCapabilitiesRef) {
    func set(_ set: CapabilitySet) -> [String]? {
      capabilities.hasSet(set) ? strings(capabilities.setLen(set)) { capabilities.setAt(set, $0) } : nil
    }

    self.init(
      bounding: set(.Bounding),
      effective: set(.Effective),
      inheritable: set(.Inheritable),
      permitted: set(.Permitted),
      ambient: set(.Ambient)
    )
  }
}

extension ContainerizationOCI.Mount {
  init(_ mount: RustOciMountRef) {
    self.init(
      type: mount.mountType().toString(),
      source: mount.source().toString(),
      destination: mount.destination().toString(),
      options: strings(mount.optionsLen(), mount.optionsAt),
      uidMappings: mount.hasUidMappings()
        ? list(mount.uidMappingsLen()) { LinuxIDMapping(mount.uidMappingsAt($0)) } : nil,
      gidMappings: mount.hasGidMappings()
        ? list(mount.gidMappingsLen()) { LinuxIDMapping(mount.gidMappingsAt($0)) } : nil
    )
  }
}

extension LinuxIDMapping {
  init(_ mapping: RustLinuxIDMappingRef) {
    self.init(containerID: mapping.containerID(), hostID: mapping.hostID(), size: mapping.size())
  }
}

extension Hook {
  init(_ hook: RustHookRef) {
    self.init(
      path: hook.path().toString(),
      args: strings(hook.argsLen(), hook.argsAt),
      env: strings(hook.envLen(), hook.envAt),
      timeout: hook.timeout()
    )
  }
}

extension Hooks {
  init(_ hooks: RustHooksRef) {
    func of(_ kind: HookKind) -> [Hook] {
      list(hooks.hooksLen(kind)) { Hook(hooks.hooksAt(kind, $0)) }
    }

    self.init(
      prestart: of(.Prestart),
      createRuntime: of(.CreateRuntime),
      createContainer: of(.CreateContainer),
      startContainer: of(.StartContainer),
      poststart: of(.Poststart),
      poststop: of(.Poststop)
    )
  }
}

extension Linux {
  init(_ linux: RustLinuxRef) {
    self.init(
      uidMappings: list(linux.uidMappingsLen()) { LinuxIDMapping(linux.uidMappingsAt($0)) },
      gidMappings: list(linux.gidMappingsLen()) { LinuxIDMapping(linux.gidMappingsAt($0)) },
      sysctl: linux.hasSysctl() ? dictionary(linux.sysctlLen(), linux.sysctlKeyAt, linux.sysctlValueAt) : nil,
      resources: linux.hasResources() ? LinuxResources(linux.resources()) : nil,
      cgroupsPath: linux.cgroupsPath().toString(),
      namespaces: list(linux.namespacesLen()) {
        LinuxNamespace(type: swiftCase(linux.namespaceTypeAt($0)), path: linux.namespacePathAt($0).toString())
      },
      devices: list(linux.devicesLen()) { LinuxDevice(linux.devicesAt($0)) },
      seccomp: linux.hasSeccomp() ? LinuxSeccomp(linux.seccomp()) : nil,
      rootfsPropagation: linux.rootfsPropagation().toString(),
      maskedPaths: strings(linux.maskedPathsLen(), linux.maskedPathsAt),
      readonlyPaths: strings(linux.readonlyPathsLen(), linux.readonlyPathsAt),
      mountLabel: linux.mountLabel().toString(),
      personality: linux.hasPersonality()
        ? LinuxPersonality(
          domain: swiftCase(linux.personalityDomain()),
          flags: strings(linux.personalityFlagsLen(), linux.personalityFlagsAt)
        ) : nil
    )
  }
}

extension LinuxResources {
  init(_ resources: RustLinuxResourcesRef) {
    self.init(
      devices: list(resources.devicesLen()) { LinuxDeviceCgroup(resources.devicesAt($0)) },
      memory: resources.hasMemory() ? LinuxMemory(resources.memory()) : nil,
      cpu: resources.hasCpu() ? LinuxCPU(resources.cpu()) : nil,
      pids: resources.pidsLimit().map { LinuxPids(limit: $0) },
      blockIO: resources.hasBlockIO() ? LinuxBlockIO(resources.blockIO()) : nil,
      hugepageLimits: list(resources.hugepageLimitsLen()) {
        LinuxHugepageLimit(
          pagesize: resources.hugepageLimitPagesizeAt($0).toString(),
          limit: resources.hugepageLimitLimitAt($0)
        )
      },
      network: resources.hasNetwork()
        ? LinuxNetwork(
          classID: resources.networkClassID(),
          priorities: list(resources.networkPrioritiesLen()) {
            LinuxInterfacePriority(
              name: resources.networkPriorityNameAt($0).toString(),
              priority: resources.networkPriorityAt($0)
            )
          }
        ) : nil,
      rdma: resources.hasRdma()
        ? Dictionary(
          uniqueKeysWithValues: list(resources.rdmaLen()) {
            (
              resources.rdmaKeyAt($0).toString(),
              LinuxRdma(hcsHandles: resources.rdmaHcsHandlesAt($0), hcaObjects: resources.rdmaHcaObjectsAt($0))
            )
          }
        ) : nil
    )
    // The initializer can't make `unified` nil.
    unified =
      resources.hasUnified()
      ? dictionary(resources.unifiedLen(), resources.unifiedKeyAt, resources.unifiedValueAt) : nil
  }
}

extension LinuxMemory {
  init(_ memory: RustLinuxMemoryRef) {
    self.init(
      limit: memory.limit(),
      reservation: memory.reservation(),
      swap: memory.swap(),
      kernel: memory.kernel(),
      kernelTCP: memory.kernelTCP(),
      swappiness: memory.swappiness(),
      disableOOMKiller: memory.disableOOMKiller(),
      useHierarchy: memory.useHierarchy(),
      checkBeforeUpdate: memory.checkBeforeUpdate()
    )
  }
}

extension LinuxCPU {
  init(_ cpu: RustLinuxCPURef) {
    self.init(
      shares: cpu.shares(),
      quota: cpu.quota(),
      burst: cpu.burst(),
      period: cpu.period(),
      realtimeRuntime: cpu.realtimeRuntime(),
      realtimePeriod: cpu.realtimePeriod(),
      cpus: cpu.cpus().toString(),
      mems: cpu.mems().toString(),
      idle: cpu.idle()
    )
  }
}

extension LinuxBlockIO {
  init(_ blockIO: RustLinuxBlockIORef) {
    func throttle(_ kind: ThrottleKind) -> [LinuxThrottleDevice] {
      list(blockIO.throttleLen(kind)) {
        LinuxThrottleDevice(
          major: blockIO.throttleMajorAt(kind, $0),
          minor: blockIO.throttleMinorAt(kind, $0),
          rate: blockIO.throttleRateAt(kind, $0)
        )
      }
    }

    self.init(
      weight: blockIO.weight(),
      leafWeight: blockIO.leafWeight(),
      weightDevice: list(blockIO.weightDeviceLen()) {
        LinuxWeightDevice(
          major: blockIO.weightDeviceMajorAt($0),
          minor: blockIO.weightDeviceMinorAt($0),
          weight: blockIO.weightDeviceWeightAt($0),
          leafWeight: blockIO.weightDeviceLeafWeightAt($0)
        )
      },
      throttleReadBpsDevice: throttle(.ReadBps),
      throttleWriteBpsDevice: throttle(.WriteBps),
      throttleReadIOPSDevice: throttle(.ReadIops),
      throttleWriteIOPSDevice: throttle(.WriteIops)
    )
  }
}

extension LinuxDevice {
  init(_ device: RustLinuxDeviceRef) {
    self.init(
      path: device.path().toString(),
      type: device.deviceType().toString(),
      major: device.major(),
      minor: device.minor(),
      fileMode: device.fileMode(),
      uid: device.uid(),
      gid: device.gid()
    )
  }
}

extension LinuxDeviceCgroup {
  init(_ device: RustLinuxDeviceCgroupRef) {
    self.init(
      allow: device.allow(),
      type: device.deviceType().toString(),
      major: device.major(),
      minor: device.minor(),
      access: device.access()?.toString()
    )
  }
}

extension LinuxSeccomp {
  init(_ seccomp: RustLinuxSeccompRef) {
    self.init(
      defaultAction: swiftCase(seccomp.defaultAction()),
      defaultErrnoRet: seccomp.defaultErrnoRet(),
      architectures: list(seccomp.architecturesLen()) { swiftCase(seccomp.architecturesAt($0)) },
      flags: list(seccomp.flagsLen()) { swiftCase(seccomp.flagsAt($0)) },
      listenerPath: seccomp.listenerPath().toString(),
      listenerMetadata: seccomp.listenerMetadata().toString(),
      syscalls: list(seccomp.syscallsLen()) { LinuxSyscall(seccomp.syscallsAt($0)) }
    )
  }
}

extension LinuxSyscall {
  init(_ syscall: RustLinuxSyscallRef) {
    self.init(
      names: strings(syscall.namesLen(), syscall.namesAt),
      action: swiftCase(syscall.action()),
      errnoRet: syscall.errnoRet(),
      args: list(syscall.argsLen()) {
        LinuxSeccompArg(
          index: syscall.argIndexAt($0),
          value: syscall.argValueAt($0),
          valueTwo: syscall.argValueTwoAt($0),
          op: swiftCase(syscall.argOpAt($0))
        )
      }
    )
  }
}

extension ImageConfig {
  init(_ config: RustImageConfigRef) {
    self.init(
      user: config.user()?.toString(),
      env: config.hasEnv() ? strings(config.envLen(), config.envAt) : nil,
      entrypoint: config.hasEntrypoint() ? strings(config.entrypointLen(), config.entrypointAt) : nil,
      cmd: config.hasCmd() ? strings(config.cmdLen(), config.cmdAt) : nil,
      workingDir: config.workingDir()?.toString(),
      labels: config.hasLabels() ? dictionary(config.labelsLen(), config.labelKeyAt, config.labelValueAt) : nil,
      stopSignal: config.stopSignal()?.toString()
    )
  }
}

// MARK: Swift to Rust

/// Rust's `Vec` of numbers.
func rustVec<T: Vectorizable>(_ values: some Sequence<T>) -> RustVec<T> {
  let vec = RustVec<T>()
  for value in values {
    vec.push(value: value)
  }
  return vec
}

// The OCI runtime spec, field by field, as the image types are.
extension CzOutcome {
  /// A held `[String: T]`'s keys, sorted.
  func entryKeys() -> RustVec<RustString> {
    rustStrings(sortedEntries().map(\.key))
  }

  /// A held `[String: T]`'s values, in the order of `entryKeys`.
  func entryValues() -> CzOutcome {
    CzOutcome.holding(sortedEntries().map(\.value))
  }

  private func sortedEntries() -> [(key: String, value: Any)] {
    (taken() as [String: Any]).sorted { $0.key < $1.key }
  }

  // `Spec`.

  func specVersion() -> String {
    (taken() as Spec).version
  }

  func specHooks() -> CzOutcome {
    CzOutcome.holding((taken() as Spec).hooks)
  }

  func specProcess() -> CzOutcome {
    CzOutcome.holding((taken() as Spec).process)
  }

  func specHostname() -> String {
    (taken() as Spec).hostname
  }

  func specDomainname() -> String {
    (taken() as Spec).domainname
  }

  func specMounts() -> CzOutcome {
    CzOutcome.holding((taken() as Spec).mounts)
  }

  func specAnnotations() -> CzOutcome {
    CzOutcome.holding((taken() as Spec).annotations)
  }

  func specRoot() -> CzOutcome {
    CzOutcome.holding((taken() as Spec).root)
  }

  func specLinux() -> CzOutcome {
    CzOutcome.holding((taken() as Spec).linux)
  }

  // `Process`.

  func processCwd() -> String {
    (taken() as ContainerizationOCI.Process).cwd
  }

  func processEnv() -> RustVec<RustString> {
    rustStrings((taken() as ContainerizationOCI.Process).env)
  }

  func processConsoleSize() -> CzOutcome {
    CzOutcome.holding((taken() as ContainerizationOCI.Process).consoleSize)
  }

  func processSelinuxLabel() -> String {
    (taken() as ContainerizationOCI.Process).selinuxLabel
  }

  func processNoNewPrivileges() -> Bool {
    (taken() as ContainerizationOCI.Process).noNewPrivileges
  }

  func processCommandLine() -> String {
    (taken() as ContainerizationOCI.Process).commandLine
  }

  func processOomScoreAdj() -> Int? {
    (taken() as ContainerizationOCI.Process).oomScoreAdj
  }

  func processCapabilities() -> CzOutcome {
    CzOutcome.holding((taken() as ContainerizationOCI.Process).capabilities)
  }

  func processApparmorProfile() -> String {
    (taken() as ContainerizationOCI.Process).apparmorProfile
  }

  func processUser() -> CzOutcome {
    CzOutcome.holding((taken() as ContainerizationOCI.Process).user)
  }

  func processRlimits() -> CzOutcome {
    CzOutcome.holding((taken() as ContainerizationOCI.Process).rlimits)
  }

  func processArgs() -> RustVec<RustString> {
    rustStrings((taken() as ContainerizationOCI.Process).args)
  }

  func processTerminal() -> Bool {
    (taken() as ContainerizationOCI.Process).terminal
  }

  // `Box`, whose fields are internal.

  func boxHeight() -> UInt {
    cast(Mirror(reflecting: taken() as Box).descendant("height"))
  }

  func boxWidth() -> UInt {
    cast(Mirror(reflecting: taken() as Box).descendant("width"))
  }

  // `User`.

  func userUid() -> UInt32 {
    (taken() as ContainerizationOCI.User).uid
  }

  func userGid() -> UInt32 {
    (taken() as ContainerizationOCI.User).gid
  }

  func userUmask() -> UInt32? {
    (taken() as ContainerizationOCI.User).umask
  }

  func userAdditionalGids() -> RustVec<UInt32> {
    rustVec((taken() as ContainerizationOCI.User).additionalGids)
  }

  func userUsername() -> String {
    (taken() as ContainerizationOCI.User).username
  }

  // `LinuxCapabilities`.

  func capabilitiesSet(set: CapabilitySet) -> CzOutcome {
    let capabilities: ContainerizationOCI.LinuxCapabilities = taken()
    let names: [String]? =
      switch set {
      case .Bounding: capabilities.bounding
      case .Effective: capabilities.effective
      case .Inheritable: capabilities.inheritable
      case .Permitted: capabilities.permitted
      case .Ambient: capabilities.ambient
      }

    return CzOutcome.holding(names)
  }

  // `Root`.

  func rootPath() -> String {
    (taken() as Root).path
  }

  func rootReadonly() -> Bool {
    (taken() as Root).readonly
  }

  // `Mount`.

  func ociMountType() -> String {
    (taken() as ContainerizationOCI.Mount).type
  }

  func ociMountSource() -> String {
    (taken() as ContainerizationOCI.Mount).source
  }

  func ociMountDestination() -> String {
    (taken() as ContainerizationOCI.Mount).destination
  }

  func ociMountOptions() -> RustVec<RustString> {
    rustStrings((taken() as ContainerizationOCI.Mount).options)
  }

  func ociMountUidMappings() -> CzOutcome {
    CzOutcome.holding((taken() as ContainerizationOCI.Mount).uidMappings)
  }

  func ociMountGidMappings() -> CzOutcome {
    CzOutcome.holding((taken() as ContainerizationOCI.Mount).gidMappings)
  }

  // `Hook` and `Hooks`.

  func hookPath() -> String {
    (taken() as Hook).path
  }

  func hookArgs() -> RustVec<RustString> {
    rustStrings((taken() as Hook).args)
  }

  func hookEnv() -> RustVec<RustString> {
    rustStrings((taken() as Hook).env)
  }

  func hookTimeout() -> Int? {
    (taken() as Hook).timeout
  }

  func hooksOf(kind: HookKind) -> CzOutcome {
    let hooks: Hooks = taken()
    let list: [Hook] =
      switch kind {
      case .Prestart: hooks.prestart
      case .CreateRuntime: hooks.createRuntime
      case .CreateContainer: hooks.createContainer
      case .StartContainer: hooks.startContainer
      case .Poststart: hooks.poststart
      case .Poststop: hooks.poststop
      }

    return CzOutcome.holding(list)
  }

  // `Linux`.

  func linuxUidMappings() -> CzOutcome {
    CzOutcome.holding((taken() as Linux).uidMappings)
  }

  func linuxGidMappings() -> CzOutcome {
    CzOutcome.holding((taken() as Linux).gidMappings)
  }

  func linuxSysctl() -> CzOutcome {
    CzOutcome.holding((taken() as Linux).sysctl)
  }

  func linuxResources() -> CzOutcome {
    CzOutcome.holding((taken() as Linux).resources)
  }

  func linuxCgroupsPath() -> String {
    (taken() as Linux).cgroupsPath
  }

  func linuxNamespaces() -> CzOutcome {
    CzOutcome.holding((taken() as Linux).namespaces)
  }

  func linuxDevices() -> CzOutcome {
    CzOutcome.holding((taken() as Linux).devices)
  }

  func linuxSeccomp() -> CzOutcome {
    CzOutcome.holding((taken() as Linux).seccomp)
  }

  func linuxRootfsPropagation() -> String {
    (taken() as Linux).rootfsPropagation
  }

  func linuxMaskedPaths() -> RustVec<RustString> {
    rustStrings((taken() as Linux).maskedPaths)
  }

  func linuxReadonlyPaths() -> RustVec<RustString> {
    rustStrings((taken() as Linux).readonlyPaths)
  }

  func linuxMountLabel() -> String {
    (taken() as Linux).mountLabel
  }

  func linuxPersonality() -> CzOutcome {
    CzOutcome.holding((taken() as Linux).personality)
  }

  // `LinuxNamespace`, `LinuxIDMapping` and `POSIXRlimit`.

  func namespaceType() -> String {
    (taken() as LinuxNamespace).type.rawValue
  }

  func namespacePath() -> String {
    (taken() as LinuxNamespace).path
  }

  func idMappingContainerID() -> UInt32 {
    (taken() as LinuxIDMapping).containerID
  }

  func idMappingHostID() -> UInt32 {
    (taken() as LinuxIDMapping).hostID
  }

  func idMappingSize() -> UInt32 {
    (taken() as LinuxIDMapping).size
  }

  func rlimitType() -> String {
    (taken() as POSIXRlimit).type
  }

  func rlimitHard() -> UInt64 {
    (taken() as POSIXRlimit).hard
  }

  func rlimitSoft() -> UInt64 {
    (taken() as POSIXRlimit).soft
  }

  // `LinuxResources`, and the types only it holds.

  func resourcesDevices() -> CzOutcome {
    CzOutcome.holding((taken() as LinuxResources).devices)
  }

  func resourcesMemory() -> CzOutcome {
    CzOutcome.holding((taken() as LinuxResources).memory)
  }

  func resourcesCpu() -> CzOutcome {
    CzOutcome.holding((taken() as LinuxResources).cpu)
  }

  func resourcesPids() -> CzOutcome {
    CzOutcome.holding((taken() as LinuxResources).pids)
  }

  func resourcesBlockIO() -> CzOutcome {
    CzOutcome.holding((taken() as LinuxResources).blockIO)
  }

  func resourcesHugepageLimits() -> CzOutcome {
    CzOutcome.holding((taken() as LinuxResources).hugepageLimits)
  }

  func resourcesNetwork() -> CzOutcome {
    CzOutcome.holding((taken() as LinuxResources).network)
  }

  func resourcesRdma() -> CzOutcome {
    CzOutcome.holding((taken() as LinuxResources).rdma)
  }

  func resourcesUnified() -> CzOutcome {
    CzOutcome.holding((taken() as LinuxResources).unified)
  }

  func memoryLimit() -> Int64? {
    (taken() as LinuxMemory).limit
  }

  func memoryReservation() -> Int64? {
    (taken() as LinuxMemory).reservation
  }

  func memorySwap() -> Int64? {
    (taken() as LinuxMemory).swap
  }

  func memoryKernel() -> Int64? {
    (taken() as LinuxMemory).kernel
  }

  func memoryKernelTCP() -> Int64? {
    (taken() as LinuxMemory).kernelTCP
  }

  func memorySwappiness() -> UInt64? {
    (taken() as LinuxMemory).swappiness
  }

  func memoryDisableOOMKiller() -> Bool? {
    (taken() as LinuxMemory).disableOOMKiller
  }

  func memoryUseHierarchy() -> Bool? {
    (taken() as LinuxMemory).useHierarchy
  }

  func memoryCheckBeforeUpdate() -> Bool? {
    (taken() as LinuxMemory).checkBeforeUpdate
  }

  func cpuShares() -> UInt64? {
    (taken() as LinuxCPU).shares
  }

  func cpuQuota() -> Int64? {
    (taken() as LinuxCPU).quota
  }

  func cpuBurst() -> UInt64? {
    (taken() as LinuxCPU).burst
  }

  func cpuPeriod() -> UInt64? {
    (taken() as LinuxCPU).period
  }

  func cpuRealtimeRuntime() -> Int64? {
    (taken() as LinuxCPU).realtimeRuntime
  }

  func cpuRealtimePeriod() -> Int64? {
    (taken() as LinuxCPU).realtimePeriod
  }

  func cpuCpus() -> String {
    (taken() as LinuxCPU).cpus
  }

  func cpuMems() -> String {
    (taken() as LinuxCPU).mems
  }

  func cpuIdle() -> Int64? {
    (taken() as LinuxCPU).idle
  }

  func pidsLimit() -> Int64 {
    (taken() as LinuxPids).limit
  }

  func blockIOWeight() -> UInt16? {
    (taken() as LinuxBlockIO).weight
  }

  func blockIOLeafWeight() -> UInt16? {
    (taken() as LinuxBlockIO).leafWeight
  }

  func blockIOWeightDevice() -> CzOutcome {
    CzOutcome.holding((taken() as LinuxBlockIO).weightDevice)
  }

  func blockIOThrottle(kind: ThrottleKind) -> CzOutcome {
    let blockIO: LinuxBlockIO = taken()
    let devices: [LinuxThrottleDevice] =
      switch kind {
      case .ReadBps: blockIO.throttleReadBpsDevice
      case .WriteBps: blockIO.throttleWriteBpsDevice
      case .ReadIops: blockIO.throttleReadIOPSDevice
      case .WriteIops: blockIO.throttleWriteIOPSDevice
      }

    return CzOutcome.holding(devices)
  }

  func weightDeviceMajor() -> Int64 {
    (taken() as LinuxWeightDevice).major
  }

  func weightDeviceMinor() -> Int64 {
    (taken() as LinuxWeightDevice).minor
  }

  func weightDeviceWeight() -> UInt16? {
    (taken() as LinuxWeightDevice).weight
  }

  func weightDeviceLeafWeight() -> UInt16? {
    (taken() as LinuxWeightDevice).leafWeight
  }

  func throttleDeviceMajor() -> Int64 {
    (taken() as LinuxThrottleDevice).major
  }

  func throttleDeviceMinor() -> Int64 {
    (taken() as LinuxThrottleDevice).minor
  }

  func throttleDeviceRate() -> UInt64 {
    (taken() as LinuxThrottleDevice).rate
  }

  func hugepageLimitPagesize() -> String {
    (taken() as LinuxHugepageLimit).pagesize
  }

  func hugepageLimitLimit() -> UInt64 {
    (taken() as LinuxHugepageLimit).limit
  }

  func networkClassID() -> UInt32? {
    (taken() as LinuxNetwork).classID
  }

  func networkPriorities() -> CzOutcome {
    CzOutcome.holding((taken() as LinuxNetwork).priorities)
  }

  func interfacePriorityName() -> String {
    (taken() as LinuxInterfacePriority).name
  }

  func interfacePriorityPriority() -> UInt32 {
    (taken() as LinuxInterfacePriority).priority
  }

  func rdmaHcsHandles() -> UInt32? {
    (taken() as LinuxRdma).hcsHandles
  }

  func rdmaHcaObjects() -> UInt32? {
    (taken() as LinuxRdma).hcaObjects
  }

  // `LinuxDevice` and `LinuxDeviceCgroup`.

  func devicePath() -> String {
    (taken() as LinuxDevice).path
  }

  func deviceType() -> String {
    (taken() as LinuxDevice).type
  }

  func deviceMajor() -> Int64 {
    (taken() as LinuxDevice).major
  }

  func deviceMinor() -> Int64 {
    (taken() as LinuxDevice).minor
  }

  func deviceFileMode() -> UInt32? {
    (taken() as LinuxDevice).fileMode
  }

  func deviceUid() -> UInt32? {
    (taken() as LinuxDevice).uid
  }

  func deviceGid() -> UInt32? {
    (taken() as LinuxDevice).gid
  }

  func deviceCgroupAllow() -> Bool {
    (taken() as LinuxDeviceCgroup).allow
  }

  func deviceCgroupType() -> String {
    (taken() as LinuxDeviceCgroup).type
  }

  func deviceCgroupMajor() -> Int64? {
    (taken() as LinuxDeviceCgroup).major
  }

  func deviceCgroupMinor() -> Int64? {
    (taken() as LinuxDeviceCgroup).minor
  }

  func deviceCgroupAccess() -> String? {
    (taken() as LinuxDeviceCgroup).access
  }

  // `LinuxPersonality`.

  func personalityDomain() -> String {
    (taken() as LinuxPersonality).domain.rawValue
  }

  func personalityFlags() -> RustVec<RustString> {
    rustStrings((taken() as LinuxPersonality).flags)
  }

  // `LinuxSeccomp`, `LinuxSyscall` and `LinuxSeccompArg`.

  func seccompDefaultAction() -> String {
    (taken() as LinuxSeccomp).defaultAction.rawValue
  }

  func seccompDefaultErrnoRet() -> UInt? {
    (taken() as LinuxSeccomp).defaultErrnoRet
  }

  func seccompArchitectures() -> RustVec<RustString> {
    rustStrings((taken() as LinuxSeccomp).architectures.map(\.rawValue))
  }

  func seccompFlags() -> RustVec<RustString> {
    rustStrings((taken() as LinuxSeccomp).flags.map(\.rawValue))
  }

  func seccompListenerPath() -> String {
    (taken() as LinuxSeccomp).listenerPath
  }

  func seccompListenerMetadata() -> String {
    (taken() as LinuxSeccomp).listenerMetadata
  }

  func seccompSyscalls() -> CzOutcome {
    CzOutcome.holding((taken() as LinuxSeccomp).syscalls)
  }

  func syscallNames() -> RustVec<RustString> {
    rustStrings((taken() as LinuxSyscall).names)
  }

  func syscallAction() -> String {
    (taken() as LinuxSyscall).action.rawValue
  }

  func syscallErrnoRet() -> UInt? {
    (taken() as LinuxSyscall).errnoRet
  }

  func syscallArgs() -> CzOutcome {
    CzOutcome.holding((taken() as LinuxSyscall).args)
  }

  func seccompArgIndex() -> UInt {
    (taken() as LinuxSeccompArg).index
  }

  func seccompArgValue() -> UInt64 {
    (taken() as LinuxSeccompArg).value
  }

  func seccompArgValueTwo() -> UInt64 {
    (taken() as LinuxSeccompArg).valueTwo
  }

  func seccompArgOp() -> String {
    (taken() as LinuxSeccompArg).op.rawValue
  }

  // `RuntimeSpecVersion`.

  func runtimeSpecVersionMajor() -> Int {
    (taken() as RuntimeSpecVersion).major
  }

  func runtimeSpecVersionMinor() -> Int {
    (taken() as RuntimeSpecVersion).minor
  }

  func runtimeSpecVersionPatch() -> Int {
    (taken() as RuntimeSpecVersion).patch
  }

  func runtimeSpecVersionDev() -> String {
    (taken() as RuntimeSpecVersion).dev
  }
}
