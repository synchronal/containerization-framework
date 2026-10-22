//===----------------------------------------------------------------------===//
// `LinuxPod`, from Containerization.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationOS
import Foundation

/// `configuration` is what Rust's closure made of `LinuxPod.Configuration()`,
/// which Swift's closure takes.
func linuxPod(
  id: RustStr,
  vmm: CzVirtualMachineManager,
  vmCpus: UInt32,
  vmMemoryInBytes: UInt64,
  configuration: RustPodConfiguration
) -> CzOutcome {
  let id = id.toString()

  return CzOutcome {
    let configuration = try LinuxPod.Configuration(configuration)

    return CzLinuxPod(
      try LinuxPod(id, vmm: vmm.manager, vm: VMResources(cpus: Int(vmCpus), memoryInBytes: vmMemoryInBytes)) {
        $0 = configuration
      }
    )
  }
}

final class CzLinuxPod: Sendable {
  let pod: LinuxPod

  init(_ pod: LinuxPod) {
    self.pod = pod
  }

  func id() -> String {
    pod.id
  }

  func config(seed: RustPodConfiguration, receive: RustConfigurePod) {
    fill(seed, from: pod.config)
    _ = receive.call(seed)
  }

  func vmCpus() -> UInt32 {
    UInt32(pod.vm.cpus)
  }

  func vmMemoryInBytes() -> UInt64 {
    pod.vm.memoryInBytes
  }

  /// `seed` is Rust's `ContainerConfiguration()`, for `configuration` to
  /// change. Swift doesn't fill it, because the pod seeds Swift's default,
  /// which Rust's matches.
  func addContainer(
    id: RustStr,
    rootfs: RustMount,
    seed: RustPodContainerConfiguration,
    configuration: RustConfigurePodContainer
  ) -> CzOutcome {
    let pod = pod
    let id = id.toString()
    let rootfs = Containerization.Mount(rootfs)
    // Not `Sendable`, but only the closure below uses them, and the pod calls
    // it once, while `blocking` waits.
    nonisolated(unsafe) let seed = seed
    nonisolated(unsafe) let configuration = configuration

    return CzOutcome {
      try blocking {
        try await pod.addContainer(id, rootfs: rootfs) { config in
          if configuration.call(seed) { throw BridgeError.bodyFailed }
          config = try LinuxPod.ContainerConfiguration(seed)
        }
      }
    }
  }

  func create() -> CzOutcome {
    .awaiting { try await self.pod.create() }
  }

  func startContainer(id: RustStr) -> CzOutcome {
    let id = id.toString()

    return .awaiting { try await self.pod.startContainer(id) }
  }

  func stopContainer(id: RustStr) -> CzOutcome {
    let id = id.toString()

    return .awaiting { try await self.pod.stopContainer(id) }
  }

  func stop() -> CzOutcome {
    .awaiting { try await self.pod.stop() }
  }

  func killContainer(id: RustStr, signal: Int32) -> CzOutcome {
    let id = id.toString()

    return .awaiting { try await self.pod.killContainer(id, signal: Signal(rawValue: signal)) }
  }

  func waitContainer(id: RustStr, timeoutInSeconds: Int64?) -> CzOutcome {
    let id = id.toString()

    return .awaiting { try await self.pod.waitContainer(id, timeoutInSeconds: timeoutInSeconds) }
  }

  func resizeContainer(id: RustStr, width: UInt16, height: UInt16) -> CzOutcome {
    let id = id.toString()

    return .awaiting { try await self.pod.resizeContainer(id, to: Terminal.Size(width: width, height: height)) }
  }

  /// `seed` and `configuration` as in a container's `execWith`.
  func execInContainer(
    id: RustStr,
    processID: RustStr,
    seed: RustLinuxProcessConfiguration,
    configuration: RustConfigureProcess
  ) -> CzOutcome {
    let id = id.toString()
    let processID = processID.toString()
    let output = Output()
    let configure = configured(seed, by: configuration, keeping: output)

    return .awaiting {
      CzLinuxProcess(
        try await self.pod.execInContainer(id, processID: processID, configuration: configure),
        output: output
      )
    }
  }

  /// Swift's `listContainers` doesn't throw, so `blocking` can't fail here.
  func listContainers() -> RustVec<RustString> {
    let pod = pod

    return rustStrings((try? blocking { await pod.listContainers() }) ?? [])
  }

  func statistics(hasContainerIDs: Bool, containerIDs: RustVec<RustString>, categories: Int64) -> CzOutcome {
    let containerIDs = hasContainerIDs ? strings(containerIDs) : nil
    let categories = StatCategory(rawValue: Int(categories))

    return .awaiting { try await self.pod.statistics(containerIDs: containerIDs, categories: categories) }
  }

  /// The dialed handle doesn't close its descriptor, so Rust takes it.
  func dialVsock(port: UInt32) -> CzOutcome {
    .awaiting { try await self.pod.dialVsock(port: port).fileDescriptor }
  }

  /// What `withVirtualMachineInstance` hands its closure, for Rust's closure
  /// to use.
  func virtualMachineInstance() -> CzOutcome {
    .awaiting { CzVirtualMachineInstance(try await self.pod.withVirtualMachineInstance { $0 }) }
  }

  func filesystemOperation(id: RustStr, operation: FilesystemOperationKind, path: RustStr) -> CzOutcome {
    let id = id.toString()
    let path = path.toString()
    let operation = FilesystemOperation(operation)

    return .awaiting { try await self.pod.filesystemOperation(id, operation: operation, path: path) }
  }

  func closeContainerStdin(id: RustStr) -> CzOutcome {
    let id = id.toString()

    return .awaiting { try await self.pod.closeContainerStdin(id) }
  }

  func relayUnixSocket(id: RustStr, socket: CzUnixSocketConfiguration) -> CzOutcome {
    let id = id.toString()
    let socket = socket.configuration

    return .awaiting { try await self.pod.relayUnixSocket(id, socket: socket) }
  }
}

extension CzOutcome {
  func linuxPod() -> CzLinuxPod { taken() }
}
