//===----------------------------------------------------------------------===//
// `LinuxContainer` and `LinuxProcess`, from Containerization.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationOS
import Foundation

final class CzLinuxContainer: Sendable {
  let container: LinuxContainer

  init(_ container: LinuxContainer) {
    self.container = container
  }

  func id() -> String {
    container.id
  }

  func rootfs() -> CzOutcome {
    CzOutcome.holding(container.rootfs)
  }

  func writableLayer() -> CzOutcome {
    CzOutcome.holding(container.writableLayer)
  }

  func config(seed: RustLinuxContainerConfiguration, receive: RustConfigure) {
    fill(seed, from: container.config)
    // `receive` is from Rust's `platform::filled`, which only keeps the value
    // and so never fails.
    _ = receive.call(seed)
  }

  func vmCpus() -> UInt32 {
    UInt32(container.vm.cpus)
  }

  func vmMemoryInBytes() -> UInt64 {
    container.vm.memoryInBytes
  }

  func create() -> CzOutcome {
    .awaiting { try await self.container.create() }
  }

  func start() -> CzOutcome {
    .awaiting { try await self.container.start() }
  }

  func stop() -> CzOutcome {
    .awaiting { try await self.container.stop() }
  }

  func kill(signal: Int32) -> CzOutcome {
    .awaiting { try await self.container.kill(Signal(rawValue: signal)) }
  }

  func wait(timeoutInSeconds: Int64?) -> CzOutcome {
    .awaiting { try await self.container.wait(timeoutInSeconds: timeoutInSeconds) }
  }

  func resize(width: UInt16, height: UInt16) -> CzOutcome {
    .awaiting { try await self.container.resize(to: Terminal.Size(width: width, height: height)) }
  }

  func exec(id: RustStr, configuration: RustLinuxProcessConfiguration) -> CzOutcome {
    let id = id.toString()
    let container = container

    return CzOutcome {
      let configuration = try LinuxProcessConfiguration(configuration)

      return CzLinuxProcess(
        try blocking { try await container.exec(id, configuration: configuration) },
        output: Output(configuration)
      )
    }
  }

  /// `configuration` is Rust's closure, and `seed` Rust's to fill, as in the
  /// manager's `create`.
  func execWith(id: RustStr, seed: RustLinuxProcessConfiguration, configuration: RustConfigureProcess) -> CzOutcome {
    let id = id.toString()
    let output = Output()
    let configure = configured(seed, by: configuration, keeping: output)

    return .awaiting { CzLinuxProcess(try await self.container.exec(id, configuration: configure), output: output) }
  }

  /// The dialed handle doesn't close its descriptor, so Rust takes it.
  func dialVsock(port: UInt32) -> CzOutcome {
    .awaiting { try await self.container.dialVsock(port: port).fileDescriptor }
  }

  func closeStdin() -> CzOutcome {
    .awaiting { try await self.container.closeStdin() }
  }

  func statistics(categories: Int64) -> CzOutcome {
    let categories = StatCategory(rawValue: Int(categories))

    return .awaiting { try await self.container.statistics(categories: categories) }
  }

  func filesystemOperation(operation: FilesystemOperationKind, path: RustStr) -> CzOutcome {
    let path = path.toString()
    let operation = FilesystemOperation(operation)

    return .awaiting { try await self.container.filesystemOperation(operation: operation, path: path) }
  }

  func copyIn(source: RustStr, destination: RustStr, mode: UInt32, createParents: Bool, chunkSize: UInt) -> CzOutcome {
    let container = container
    let source = URL(filePath: source.toString())
    let destination = URL(filePath: destination.toString())

    return CzOutcome {
      let chunkSize = try int(chunkSize, "chunk size")
      return try blocking {
        try await container.copyIn(
          from: source,
          to: destination,
          mode: mode,
          createParents: createParents,
          chunkSize: chunkSize
        )
      }
    }
  }

  func copyOut(source: RustStr, destination: RustStr, createParents: Bool, chunkSize: UInt) -> CzOutcome {
    let container = container
    let source = URL(filePath: source.toString())
    let destination = URL(filePath: destination.toString())

    return CzOutcome {
      let chunkSize = try int(chunkSize, "chunk size")
      return try blocking {
        try await container.copyOut(
          from: source,
          to: destination,
          createParents: createParents,
          chunkSize: chunkSize
        )
      }
    }
  }

  /// What `withVirtualMachineInstance` hands its closure, for Rust's closure
  /// to use.
  func virtualMachineInstance() -> CzOutcome {
    .awaiting { CzVirtualMachineInstance(try await self.container.withVirtualMachineInstance { $0 }) }
  }
}

/// `writableLayer` stands for `nil` when `hasWritableLayer` is false.
func linuxContainer(
  id: RustStr,
  rootfs: RustMount,
  hasWritableLayer: Bool,
  writableLayer: RustMount,
  vmm: CzVirtualMachineManager,
  vmCpus: UInt32,
  vmMemoryInBytes: UInt64,
  configuration: RustLinuxContainerConfiguration
) -> CzOutcome {
  let id = id.toString()
  let rootfs = Containerization.Mount(rootfs)
  let writableLayer = hasWritableLayer ? Containerization.Mount(writableLayer) : nil

  return CzOutcome {
    CzLinuxContainer(
      try LinuxContainer(
        id,
        rootfs: rootfs,
        writableLayer: writableLayer,
        vmm: vmm.manager,
        vm: VMResources(vmCpus, vmMemoryInBytes),
        configuration: try LinuxContainer.Configuration(configuration)
      )
    )
  }
}

// MARK: For unit tests

func linuxContainerDefaultMounts() -> CzOutcome {
  CzOutcome { [LinuxContainer.defaultMounts(), LinuxContainer.defaultOCIMounts()] }
}

func linuxContainerDefaultCopyChunkSize() -> UInt {
  UInt(LinuxContainer.defaultCopyChunkSize)
}

extension CzOutcome {
  func linuxContainer() -> CzLinuxContainer { taken() }
  func linuxProcess() -> CzLinuxProcess { taken() }
}

// MARK: ContainerStatistics

extension CzOutcome {
  /// The `id`, `interface` or `mountPoint` of the statistics held.
  func statisticsName() -> String {
    switch held(Any.self) {
    case let statistics as ContainerStatistics: statistics.id
    case let network as ContainerStatistics.NetworkStatistics: network.interface
    case let filesystem as ContainerStatistics.FilesystemStatistics: filesystem.mountPoint
    case let other: cast(other)
    }
  }

  /// The `UInt64` fields of the statistics held, in the order Swift declares
  /// them.
  func statisticsNumbers() -> RustVec<UInt64> {
    let numbers: [UInt64] =
      switch held(Any.self) {
      case let process as ContainerStatistics.ProcessStatistics:
        [process.current, process.limit]
      case let memory as ContainerStatistics.MemoryStatistics:
        [
          memory.usageBytes, memory.limitBytes, memory.swapUsageBytes, memory.swapLimitBytes, memory.cacheBytes,
          memory.kernelStackBytes, memory.slabBytes, memory.pageFaults, memory.majorPageFaults,
          memory.inactiveFile, memory.anon, memory.workingsetRefaultAnon, memory.workingsetRefaultFile,
          memory.pgstealKswapd, memory.pgstealDirect, memory.pgstealKhugepaged,
        ]
      case let cpu as ContainerStatistics.CPUStatistics:
        [
          cpu.usageUsec, cpu.userUsec, cpu.systemUsec, cpu.throttlingPeriods, cpu.throttledPeriods,
          cpu.throttledTimeUsec,
        ]
      case let device as ContainerStatistics.BlockIODevice:
        [
          device.major, device.minor, device.readBytes, device.writeBytes, device.readOperations,
          device.writeOperations,
        ]
      case let network as ContainerStatistics.NetworkStatistics:
        [
          network.receivedPackets, network.transmittedPackets, network.receivedBytes, network.transmittedBytes,
          network.receivedDropped, network.transmittedDropped,
        ]
      case let counters as ContainerStatistics.NetworkProtocolCounters:
        [counters.receivedPackets, counters.transmittedPackets]
      case let events as ContainerStatistics.MemoryEventStatistics:
        [events.low, events.high, events.max, events.oom, events.oomKill]
      case let filesystem as ContainerStatistics.FilesystemStatistics:
        [filesystem.blockSize, filesystem.blocks, filesystem.freeBlocks, filesystem.inodes, filesystem.freeInodes]
      case let other: cast(other)
      }

    return rustVec(numbers)
  }

  /// The `UInt64?` fields of the statistics held, in the order Swift declares
  /// them, each a `UInt64` or `Absent`.
  func statisticsOptionalNumbers() -> CzOutcome {
    let numbers: [UInt64?] =
      switch held(Any.self) {
      case let network as ContainerStatistics.NetworkStatistics:
        [network.receivedErrors, network.transmittedErrors]
      case let counters as ContainerStatistics.NetworkProtocolCounters:
        [
          counters.receivedBytes, counters.transmittedBytes, counters.receivedDrops, counters.transmittedDrops,
          counters.receivedErrors, counters.transmittedErrors,
        ]
      case let other: cast(other)
      }

    return CzOutcome.holding(numbers.map { absent($0) })
  }

  /// The part of the held `ContainerStatistics` that `category` reports.
  func statisticsCategory(category: Int64) -> CzOutcome {
    let statistics: ContainerStatistics = taken()

    return switch StatCategory(rawValue: Int(category)) {
    case .process: CzOutcome.holding(statistics.process)
    case .memory: CzOutcome.holding(statistics.memory)
    case .cpu: CzOutcome.holding(statistics.cpu)
    case .blockIO: CzOutcome.holding(statistics.blockIO?.devices)
    case .network: CzOutcome.holding(statistics.networks)
    case .memoryEvents: CzOutcome.holding(statistics.memoryEvents)
    case .filesystem: CzOutcome.holding(statistics.filesystem)
    default: preconditionFailure("Rust asks for one category at a time, not \(category)")
    }
  }

  /// The held `ContainerStatistics`' `networkProtocols`, as its `tcp`, `udp`,
  /// `icmp`, `ip`, `udp6`, `icmp6` and `ip6` counters. The last three may be
  /// `Absent`.
  func statisticsNetworkProtocols() -> CzOutcome {
    let statistics: ContainerStatistics = taken()

    return CzOutcome.holding(
      statistics.networkProtocols.map {
        [$0.tcp, $0.udp, $0.icmp, $0.ip, absent($0.udp6), absent($0.icmp6), absent($0.ip6)]
      }
    )
  }
}

final class CzLinuxProcess: Sendable {
  let process: LinuxProcess
  let output: Output

  init(_ process: LinuxProcess, output: Output) {
    self.process = process
    self.output = output
  }

  func id() -> String {
    process.id
  }

  func owningContainer() -> String? {
    process.owningContainer
  }

  func pid() -> Int32 {
    process.pid
  }

  func start() -> CzOutcome {
    .awaiting { try await self.process.start() }
  }

  func kill(signal: Int32) -> CzOutcome {
    .awaiting { try await self.process.kill(Signal(rawValue: signal)) }
  }

  func resize(width: UInt16, height: UInt16) -> CzOutcome {
    .awaiting { try await self.process.resize(to: Terminal.Size(width: width, height: height)) }
  }

  func closeStdin() -> CzOutcome {
    .awaiting { try await self.process.closeStdin() }
  }

  func wait(timeoutInSeconds: Int64?) -> CzOutcome {
    .awaiting { try await self.process.wait(timeoutInSeconds: timeoutInSeconds) }
  }

  func delete() -> CzOutcome {
    let process = process
    let output = output

    return CzOutcome {
      try blocking { try await process.delete() }
      output.close()
      return ()
    }
  }
}
