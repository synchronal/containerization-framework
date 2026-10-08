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

  /// `seed` is Rust's `ContainerConfiguration()`, which is Swift's, for
  /// `configuration` to change: Swift never fills one.
  func addContainer(
    id: RustStr,
    rootfs: RustMount,
    seed: RustPodContainerConfiguration,
    configuration: RustConfigurePodContainer
  ) -> CzOutcome {
    let pod = pod
    let id = id.toString()
    let rootfs = Containerization.Mount(rootfs)
    // Rust's: the pod calls the closure once, while `blocking` waits.
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
    let pod = pod

    return CzOutcome { try blocking { try await pod.create() } }
  }

  func startContainer(id: RustStr) -> CzOutcome {
    let pod = pod
    let id = id.toString()

    return CzOutcome { try blocking { try await pod.startContainer(id) } }
  }

  func stopContainer(id: RustStr) -> CzOutcome {
    let pod = pod
    let id = id.toString()

    return CzOutcome { try blocking { try await pod.stopContainer(id) } }
  }

  func stop() -> CzOutcome {
    let pod = pod

    return CzOutcome { try blocking { try await pod.stop() } }
  }

  func killContainer(id: RustStr, signal: Int32) -> CzOutcome {
    let pod = pod
    let id = id.toString()

    return CzOutcome { try blocking { try await pod.killContainer(id, signal: Signal(rawValue: signal)) } }
  }

  func waitContainer(id: RustStr, timeoutInSeconds: Int64?) -> CzOutcome {
    let pod = pod
    let id = id.toString()

    return CzOutcome { try blocking { try await pod.waitContainer(id, timeoutInSeconds: timeoutInSeconds) } }
  }

  func resizeContainer(id: RustStr, width: UInt16, height: UInt16) -> CzOutcome {
    let pod = pod
    let id = id.toString()

    return CzOutcome {
      try blocking { try await pod.resizeContainer(id, to: Terminal.Size(width: width, height: height)) }
    }
  }

  /// `seed` and `configuration` as in a container's `execWith`.
  func execInContainer(
    id: RustStr,
    processID: RustStr,
    seed: RustLinuxProcessConfiguration,
    configuration: RustConfigureProcess
  ) -> CzOutcome {
    let pod = pod
    let id = id.toString()
    let processID = processID.toString()
    let configure = configured(seed, by: configuration)

    return CzOutcome {
      CzLinuxProcess(
        try blocking { try await pod.execInContainer(id, processID: processID, configuration: configure) }
      )
    }
  }

  /// Swift's doesn't throw, so nor does `blocking` here.
  func listContainers() -> RustVec<RustString> {
    let pod = pod

    return rustStrings((try? blocking { await pod.listContainers() }) ?? [])
  }

  func statistics(hasContainerIDs: Bool, containerIDs: RustVec<RustString>, categories: Int64) -> CzOutcome {
    let pod = pod
    let containerIDs = hasContainerIDs ? strings(containerIDs) : nil
    let categories = StatCategory(rawValue: Int(categories))

    return CzOutcome {
      try blocking { try await pod.statistics(containerIDs: containerIDs, categories: categories) }
    }
  }

  /// The dialed handle doesn't close its descriptor, so Rust takes it.
  func dialVsock(port: UInt32) -> CzOutcome {
    let pod = pod

    return CzOutcome { try blocking { try await pod.dialVsock(port: port) }.fileDescriptor }
  }

  /// What `withVirtualMachineInstance` hands its closure, for Rust's to use.
  func virtualMachineInstance() -> CzOutcome {
    let pod = pod

    return CzOutcome {
      CzVirtualMachineInstance(try blocking { try await pod.withVirtualMachineInstance { $0 } })
    }
  }

  func filesystemOperation(id: RustStr, operation: FilesystemOperationKind, path: RustStr) -> CzOutcome {
    let pod = pod
    let id = id.toString()
    let path = path.toString()
    let operation: FilesystemOperation =
      switch operation {
      case .Freeze: .freeze
      case .Thaw: .thaw
      case .Trim: .trim
      }

    return CzOutcome { try blocking { try await pod.filesystemOperation(id, operation: operation, path: path) } }
  }

  func closeContainerStdin(id: RustStr) -> CzOutcome {
    let pod = pod
    let id = id.toString()

    return CzOutcome { try blocking { try await pod.closeContainerStdin(id) } }
  }

  func relayUnixSocket(id: RustStr, socket: CzUnixSocketConfiguration) -> CzOutcome {
    let pod = pod
    let id = id.toString()
    let socket = socket.configuration

    return CzOutcome { try blocking { try await pod.relayUnixSocket(id, socket: socket) } }
  }
}
