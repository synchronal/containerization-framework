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
    _ = receive.call(seed)
  }

  func vmCpus() -> UInt32 {
    UInt32(container.vm.cpus)
  }

  func vmMemoryInBytes() -> UInt64 {
    container.vm.memoryInBytes
  }

  func create() -> CzOutcome {
    let container = container

    return CzOutcome { try blocking { try await container.create() } }
  }

  func start() -> CzOutcome {
    let container = container

    return CzOutcome { try blocking { try await container.start() } }
  }

  func stop() -> CzOutcome {
    let container = container

    return CzOutcome { try blocking { try await container.stop() } }
  }

  func kill(signal: Int32) -> CzOutcome {
    let container = container

    return CzOutcome { try blocking { try await container.kill(Signal(rawValue: signal)) } }
  }

  func wait(timeoutInSeconds: Int64?) -> CzOutcome {
    let container = container

    return CzOutcome { try blocking { try await container.wait(timeoutInSeconds: timeoutInSeconds) } }
  }

  func resize(width: UInt16, height: UInt16) -> CzOutcome {
    let container = container

    return CzOutcome { try blocking { try await container.resize(to: Terminal.Size(width: width, height: height)) } }
  }

  func exec(id: RustStr, configuration: RustLinuxProcessConfiguration) -> CzOutcome {
    let id = id.toString()
    let container = container

    return CzOutcome {
      let configuration = try LinuxProcessConfiguration(configuration)

      return CzLinuxProcess(try blocking { try await container.exec(id, configuration: configuration) })
    }
  }

  /// `configuration` is Rust's closure, and `seed` Rust's to fill, as in the
  /// manager's `create`.
  func execWith(id: RustStr, seed: RustLinuxProcessConfiguration, configuration: RustConfigureProcess) -> CzOutcome {
    let id = id.toString()
    let container = container
    let configure = configured(seed, by: configuration)

    return CzOutcome {
      CzLinuxProcess(try blocking { try await container.exec(id, configuration: configure) })
    }
  }

  /// The dialed handle doesn't close its descriptor, so Rust takes it.
  func dialVsock(port: UInt32) -> CzOutcome {
    let container = container

    return CzOutcome { try blocking { try await container.dialVsock(port: port) }.fileDescriptor }
  }

  func closeStdin() -> CzOutcome {
    let container = container

    return CzOutcome { try blocking { try await container.closeStdin() } }
  }

  func statistics(categories: Int64) -> CzOutcome {
    let container = container
    let categories = StatCategory(rawValue: Int(categories))

    return CzOutcome { try blocking { try await container.statistics(categories: categories) } }
  }

  func filesystemOperation(operation: FilesystemOperationKind, path: RustStr) -> CzOutcome {
    let container = container
    let path = path.toString()
    let operation: FilesystemOperation =
      switch operation {
      case .Freeze: .freeze
      case .Thaw: .thaw
      case .Trim: .trim
      }

    return CzOutcome { try blocking { try await container.filesystemOperation(operation: operation, path: path) } }
  }

  func copyIn(source: RustStr, destination: RustStr, mode: UInt32, createParents: Bool, chunkSize: UInt) -> CzOutcome {
    let container = container
    let source = URL(filePath: source.toString())
    let destination = URL(filePath: destination.toString())

    return CzOutcome {
      try blocking {
        try await container.copyIn(
          from: source,
          to: destination,
          mode: mode,
          createParents: createParents,
          chunkSize: Int(chunkSize)
        )
      }
    }
  }

  func copyOut(source: RustStr, destination: RustStr, createParents: Bool, chunkSize: UInt) -> CzOutcome {
    let container = container
    let source = URL(filePath: source.toString())
    let destination = URL(filePath: destination.toString())

    return CzOutcome {
      try blocking {
        try await container.copyOut(
          from: source,
          to: destination,
          createParents: createParents,
          chunkSize: Int(chunkSize)
        )
      }
    }
  }
}

/// The closure an `exec` takes: fills `seed` with the configuration Swift
/// seeded, lets Rust's closure change it, and takes it back.
func configured(
  _ seed: RustLinuxProcessConfiguration,
  by configuration: RustConfigureProcess
) -> @Sendable (inout LinuxProcessConfiguration) throws -> Void {
  // Rust's: an `exec` calls this once, while `blocking` waits.
  nonisolated(unsafe) let seed = seed
  nonisolated(unsafe) let configuration = configuration

  return { config in
    fill(seed, from: config)
    if configuration.call(seed) { throw BridgeError.bodyFailed }
    config = try LinuxProcessConfiguration(seed)
  }
}

// MARK: For unit tests

func linuxContainerDefaultMounts() -> CzOutcome {
  CzOutcome { [LinuxContainer.defaultMounts(), LinuxContainer.defaultOCIMounts()] }
}

func linuxContainerDefaultCopyChunkSize() -> UInt {
  UInt(LinuxContainer.defaultCopyChunkSize)
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
          network.receivedErrors, network.transmittedErrors,
        ]
      case let events as ContainerStatistics.MemoryEventStatistics:
        [events.low, events.high, events.max, events.oom, events.oomKill]
      case let filesystem as ContainerStatistics.FilesystemStatistics:
        [filesystem.blockSize, filesystem.blocks, filesystem.freeBlocks, filesystem.inodes, filesystem.freeInodes]
      case let other: cast(other)
      }

    let vec = RustVec<UInt64>()
    for number in numbers {
      vec.push(value: number)
    }
    return vec
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
}

final class CzLinuxProcess: Sendable {
  let process: LinuxProcess

  init(_ process: LinuxProcess) {
    self.process = process
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
    let process = process

    return CzOutcome { try blocking { try await process.start() } }
  }

  func kill(signal: Int32) -> CzOutcome {
    let process = process

    return CzOutcome { try blocking { try await process.kill(Signal(rawValue: signal)) } }
  }

  func resize(width: UInt16, height: UInt16) -> CzOutcome {
    let process = process

    return CzOutcome { try blocking { try await process.resize(to: Terminal.Size(width: width, height: height)) } }
  }

  func closeStdin() -> CzOutcome {
    let process = process

    return CzOutcome { try blocking { try await process.closeStdin() } }
  }

  func wait(timeoutInSeconds: Int64?) -> CzOutcome {
    let process = process

    return CzOutcome { try blocking { try await process.wait(timeoutInSeconds: timeoutInSeconds) } }
  }

  func delete() -> CzOutcome {
    let process = process

    return CzOutcome { try blocking { try await process.delete() } }
  }
}
