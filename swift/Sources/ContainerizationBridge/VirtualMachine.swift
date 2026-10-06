//===----------------------------------------------------------------------===//
// `VZVirtualMachineManager`, `VZVirtualMachineInstance` and `VsockListener`,
// from Containerization, and the `LinuxContainer` inits that take a manager.
//===----------------------------------------------------------------------===//

import Containerization
import Foundation

final class CzVirtualMachineManager: Sendable {
  let manager: VZVirtualMachineManager

  init(_ manager: VZVirtualMachineManager) {
    self.manager = manager
  }

  func duplicate() -> CzVirtualMachineManager {
    CzVirtualMachineManager(manager)
  }

  func create(config: RustVmConfiguration) -> CzOutcome {
    let manager = manager

    return CzOutcome {
      CzVirtualMachineInstance(
        try manager.create(config: StandardVMConfig(configuration: try VMConfiguration(config)))
      )
    }
  }
}

func virtualMachineManager(
  kernel: RustKernel,
  initialFilesystem: RustMount,
  rosetta: Bool,
  nestedVirtualization: Bool
) -> CzOutcome {
  let initialFilesystem = Containerization.Mount(initialFilesystem)

  return CzOutcome {
    CzVirtualMachineManager(
      VZVirtualMachineManager(
        kernel: try Kernel(kernel),
        initialFilesystem: initialFilesystem,
        rosetta: rosetta,
        nestedVirtualization: nestedVirtualization
      )
    )
  }
}

final class CzVirtualMachineInstance: Sendable {
  let instance: VZVirtualMachineInstance

  /// On macOS every instance is a `VZVirtualMachineInstance`.
  init(_ instance: any VirtualMachineInstance) {
    guard let instance = instance as? VZVirtualMachineInstance else {
      preconditionFailure("a \(type(of: instance)) isn't a VZVirtualMachineInstance")
    }
    self.instance = instance
  }

  func state() -> InstanceState {
    switch instance.state {
    case .starting: .Starting
    case .running: .Running
    case .stopped: .Stopped
    case .stopping: .Stopping
    case .unknown: .Unknown
    }
  }

  func mounts() -> CzOutcome {
    CzOutcome.holding(instance.mounts)
  }

  func virtiofsLayout() -> VirtiofsLayoutKind {
    switch instance.virtiofsLayout {
    case .unified: .Unified
    case .perTag: .PerTag
    }
  }

  func start() -> CzOutcome {
    let instance = instance

    return CzOutcome { try blocking { try await instance.start() } }
  }

  func stop() -> CzOutcome {
    let instance = instance

    return CzOutcome { try blocking { try await instance.stop() } }
  }

  func pause() -> CzOutcome {
    let instance = instance

    return CzOutcome { try blocking { try await instance.pause() } }
  }

  func resume() -> CzOutcome {
    let instance = instance

    return CzOutcome { try blocking { try await instance.resume() } }
  }

  /// The dialed handle doesn't close its descriptor, so Rust takes it.
  func dial(port: UInt32) -> CzOutcome {
    let instance = instance

    return CzOutcome { try blocking { try await instance.dial(port) }.fileDescriptor }
  }

  func listen(port: UInt32) -> CzOutcome {
    CzOutcome { CzVsockListener(try instance.listen(port)) }
  }

  func hotplug(block: RustMount, id: RustStr) -> CzOutcome {
    let instance = instance
    let block = Containerization.Mount(block)
    let id = id.toString()

    return CzOutcome { try blocking { try await instance.hotplug(block, id: id) } }
  }

  func registerMounts(id: RustStr, rootfs: RustAttachedFilesystem, additionalMounts: RustVec<RustMount>) -> CzOutcome {
    let id = id.toString()
    let rootfs = AttachedFilesystem(rootfs)
    let additionalMounts = additionalMounts.map { Containerization.Mount($0) }

    return CzOutcome { try instance.registerMounts(id: id, rootfs: rootfs, additionalMounts: additionalMounts) }
  }

  func releaseHotplug(id: RustStr) -> CzOutcome {
    let instance = instance
    let id = id.toString()

    return CzOutcome { try blocking { try await instance.releaseHotplug(id: id) } }
  }

  func hotplugVirtioFS(mounts: RustVec<RustMount>, id: RustStr) -> CzOutcome {
    let instance = instance
    let mounts = mounts.map { Containerization.Mount($0) }
    let id = id.toString()

    return CzOutcome { try blocking { try await instance.hotplugVirtioFS(mounts, id: id) } }
  }

  func releaseVirtioFS(id: RustStr) -> CzOutcome {
    let instance = instance
    let id = id.toString()

    return CzOutcome { try blocking { try await instance.releaseVirtioFS(id: id) } }
  }
}

func installRosetta() -> CzOutcome {
  CzOutcome { try blocking { try await VZVirtualMachineInstance.Configuration.installRosetta() } }
}

/// Unchecked: `next` advances `iterator` only while it holds `advancing`.
/// `finish` doesn't take it, so it ends a `next` that is waiting.
final class CzVsockListener: @unchecked Sendable {
  let listener: VsockListener
  private let advancing = NSLock()
  private var iterator: AsyncStream<FileHandle>.AsyncIterator

  init(_ listener: VsockListener) {
    self.listener = listener
    self.iterator = listener.makeAsyncIterator()
  }

  func port() -> UInt32 {
    listener.port
  }

  /// The connection's handle doesn't close its descriptor, so Rust takes it.
  func next() -> CzOutcome {
    advancing.lock()
    defer { advancing.unlock() }

    return CzOutcome {
      absent(
        try blocking {
          var iterator = self.iterator
          defer { self.iterator = iterator }

          return await iterator.next()?.fileDescriptor
        }
      )
    }
  }

  func finish() -> CzOutcome {
    CzOutcome { try listener.finish() }
  }
}

// MARK: LinuxContainer

extension CzLinuxContainer {
  /// What `withVirtualMachineInstance` hands its closure, for Rust's to use.
  func virtualMachineInstance() -> CzOutcome {
    let container = container

    return CzOutcome {
      CzVirtualMachineInstance(try blocking { try await container.withVirtualMachineInstance { $0 } })
    }
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
        vm: VMResources(cpus: Int(vmCpus), memoryInBytes: vmMemoryInBytes),
        configuration: try LinuxContainer.Configuration(configuration)
      )
    )
  }
}
