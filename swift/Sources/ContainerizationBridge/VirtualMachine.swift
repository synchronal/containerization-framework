//===----------------------------------------------------------------------===//
// `VZVirtualMachineManager`, `VZVirtualMachineInstance` and `VsockListener`,
// from Containerization.
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
    .awaiting { try await self.instance.start() }
  }

  func stop() -> CzOutcome {
    .awaiting { try await self.instance.stop() }
  }

  func pause() -> CzOutcome {
    .awaiting { try await self.instance.pause() }
  }

  func resume() -> CzOutcome {
    .awaiting { try await self.instance.resume() }
  }

  /// The dialed handle doesn't close its descriptor, so Rust takes it.
  func dial(port: UInt32) -> CzOutcome {
    .awaiting { try await self.instance.dial(port).fileDescriptor }
  }

  func listen(port: UInt32) -> CzOutcome {
    CzOutcome { CzVsockListener(try instance.listen(port)) }
  }

  func hotplug(block: RustMount, id: RustStr) -> CzOutcome {
    let block = Containerization.Mount(block)
    let id = id.toString()

    return .awaiting { try await self.instance.hotplug(block, id: id) }
  }

  func registerMounts(id: RustStr, rootfs: RustAttachedFilesystem, additionalMounts: RustVec<RustMount>) -> CzOutcome {
    let id = id.toString()
    let rootfs = AttachedFilesystem(rootfs)
    let additionalMounts = additionalMounts.map { Containerization.Mount($0) }

    return CzOutcome { try instance.registerMounts(id: id, rootfs: rootfs, additionalMounts: additionalMounts) }
  }

  func releaseHotplug(id: RustStr) -> CzOutcome {
    let id = id.toString()

    return .awaiting { try await self.instance.releaseHotplug(id: id) }
  }

  func hotplugVirtioFS(mounts: RustVec<RustMount>, id: RustStr) -> CzOutcome {
    let mounts = mounts.map { Containerization.Mount($0) }
    let id = id.toString()

    return .awaiting { try await self.instance.hotplugVirtioFS(mounts, id: id) }
  }

  func releaseVirtioFS(id: RustStr) -> CzOutcome {
    let id = id.toString()

    return .awaiting { try await self.instance.releaseVirtioFS(id: id) }
  }
}

func installRosetta() -> CzOutcome {
  .awaiting { try await VZVirtualMachineInstance.Configuration.installRosetta() }
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

extension CzOutcome {
  func virtualMachineManager() -> CzVirtualMachineManager { taken() }
  func virtualMachineInstance() -> CzVirtualMachineInstance { taken() }
  func vsockListener() -> CzVsockListener { taken() }
}
