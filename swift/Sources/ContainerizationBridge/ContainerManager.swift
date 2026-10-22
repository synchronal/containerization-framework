//===----------------------------------------------------------------------===//
// `ContainerManager`, from Containerization.
//===----------------------------------------------------------------------===//

import Containerization
import Foundation

// The inits that take a store are methods on it, because swift-bridge can't
// pass a borrowed Swift type as an argument.
extension CzImageStore {
  func containerManager(
    kernel: RustKernel,
    initfs: RustMount,
    network: CzNetwork,
    rosetta: Bool,
    nestedVirtualization: Bool
  ) -> CzOutcome {
    let initfs = Containerization.Mount(initfs)
    let network = network.network
    let store = store

    return CzOutcome {
      CzContainerManager(
        try ContainerManager(
          kernel: try Kernel(kernel),
          initfs: initfs,
          imageStore: store,
          network: network,
          rosetta: rosetta,
          nestedVirtualization: nestedVirtualization
        )
      )
    }
  }

  func containerManager(
    kernel: RustKernel,
    initfsReference: RustStr,
    network: CzNetwork,
    rosetta: Bool,
    nestedVirtualization: Bool
  ) -> CzOutcome {
    let reference = initfsReference.toString()
    let network = network.network
    let store = store

    return CzOutcome {
      let kernel = try Kernel(kernel)

      return CzContainerManager(
        try blocking {
          try await ContainerManager(
            kernel: kernel,
            initfsReference: reference,
            imageStore: store,
            network: network,
            rosetta: rosetta,
            nestedVirtualization: nestedVirtualization
          )
        }
      )
    }
  }
}

/// When `root` is `nil`, the manager uses `ImageStore.default`.
func containerManagerAtRoot(
  kernel: RustKernel,
  initfs: RustMount,
  root: RustString?,
  network: CzNetwork,
  rosetta: Bool,
  nestedVirtualization: Bool
) -> CzOutcome {
  let initfs = Containerization.Mount(initfs)
  let root = root.map { URL(filePath: $0.toString()) }
  let network = network.network

  return CzOutcome {
    CzContainerManager(
      try ContainerManager(
        kernel: try Kernel(kernel),
        initfs: initfs,
        root: root,
        network: network,
        rosetta: rosetta,
        nestedVirtualization: nestedVirtualization
      )
    )
  }
}

func containerManagerAtRoot(
  kernel: RustKernel,
  initfsReference: RustStr,
  root: RustString?,
  network: CzNetwork,
  rosetta: Bool,
  nestedVirtualization: Bool
) -> CzOutcome {
  let reference = initfsReference.toString()
  let root = root.map { URL(filePath: $0.toString()) }
  let network = network.network

  return CzOutcome {
    let kernel = try Kernel(kernel)

    return CzContainerManager(
      try blocking {
        try await ContainerManager(
          kernel: kernel,
          initfsReference: reference,
          root: root,
          network: network,
          rosetta: rosetta,
          nestedVirtualization: nestedVirtualization
        )
      }
    )
  }
}

func containerManager(vmm: CzVirtualMachineManager, network: CzNetwork) -> CzOutcome {
  let vmm = vmm.manager
  let network = network.network

  return CzOutcome { CzContainerManager(try ContainerManager(vmm: vmm, network: network)) }
}

/// The manager's configuration closure. It fills `seed` with what the manager
/// seeded, lets Rust's closure change it, and takes it back.
private func configuredContainer(
  _ seed: RustLinuxContainerConfiguration,
  by configuration: RustConfigure
) -> @Sendable (inout LinuxContainer.Configuration) throws -> Void {
  // Not `Sendable`, but only this closure uses them, and the manager calls it
  // once, while `blocking` waits.
  nonisolated(unsafe) let seed = seed
  nonisolated(unsafe) let configuration = configuration

  return { config in
    fill(seed, from: config)
    if configuration.call(seed) { throw BridgeError.bodyFailed }
    config = try LinuxContainer.Configuration(seed)
  }
}

/// Unchecked: `create`, `releaseNetwork` and `delete` mutate `manager`, and
/// Rust's `&mut self` lets only one run at a time.
final class CzContainerManager: @unchecked Sendable {
  var manager: ContainerManager

  init(_ manager: ContainerManager) {
    self.manager = manager
  }

  func imageStore() -> CzImageStore {
    CzImageStore(manager.imageStore)
  }

  /// `create(_:reference:...)`, with `seed` and `configuration` as in
  /// `create`.
  func createFromReference(
    id: RustStr,
    reference: RustStr,
    options: RustCreateOptions,
    progress: RustProgressHandler,
    seed: RustLinuxContainerConfiguration,
    configuration: RustConfigure
  ) -> CzOutcome {
    let id = id.toString()
    let reference = reference.toString()
    let rootfsSizeInBytes = options.rootfsSizeInBytes()
    let writableLayerSizeInBytes = options.writableLayerSizeInBytes()
    let readOnly = options.readOnly()
    let networking = options.networking()
    let vm = VMResources(cpus: Int(options.vmCpus()), memoryInBytes: options.vmMemoryInBytes())
    let progress = progressHandler(progress)
    let configure = configuredContainer(seed, by: configuration)

    return .awaiting {
      CzLinuxContainer(
        try await self.manager.create(
          id,
          reference: reference,
          rootfsSizeInBytes: rootfsSizeInBytes,
          writableLayerSizeInBytes: writableLayerSizeInBytes,
          readOnly: readOnly,
          networking: networking,
          vm: vm,
          progress: progress,
          configuration: configure
        )
      )
    }
  }

  /// `configuration` is Rust's closure. It runs inside the manager's closure,
  /// so on a thread of Swift's cooperative pool.
  ///
  /// `seed` is Rust's to fill with what the manager seeded, for `configuration`
  /// to change: Swift never makes a Rust value.
  func create(
    id: RustStr,
    image: CzImage,
    options: RustCreateOptions,
    progress: RustProgressHandler,
    seed: RustLinuxContainerConfiguration,
    configuration: RustConfigure
  ) -> CzOutcome {
    let id = id.toString()
    let image = image.image
    let rootfsSizeInBytes = options.rootfsSizeInBytes()
    let writableLayerSizeInBytes = options.writableLayerSizeInBytes()
    let readOnly = options.readOnly()
    let networking = options.networking()
    let vm = VMResources(cpus: Int(options.vmCpus()), memoryInBytes: options.vmMemoryInBytes())
    let progress = progressHandler(progress)
    let configure = configuredContainer(seed, by: configuration)

    return .awaiting {
      CzLinuxContainer(
        try await self.manager.create(
          id,
          image: image,
          rootfsSizeInBytes: rootfsSizeInBytes,
          writableLayerSizeInBytes: writableLayerSizeInBytes,
          readOnly: readOnly,
          networking: networking,
          vm: vm,
          progress: progress,
          configuration: configure
        )
      )
    }
  }

  /// `create(_:image:rootfs:writableLayer:networking:vm:configuration:)`, with
  /// `seed` and `configuration` as in `create`.
  func createWithRootfs(
    id: RustStr,
    image: CzImage,
    rootfs: RustMount,
    options: RustRootfsCreateOptions,
    seed: RustLinuxContainerConfiguration,
    configuration: RustConfigure
  ) -> CzOutcome {
    let id = id.toString()
    let image = image.image
    let rootfs = Containerization.Mount(rootfs)
    let writableLayer = options.hasWritableLayer() ? Containerization.Mount(options.writableLayer()) : nil
    let networking = options.networking()
    let vm = VMResources(cpus: Int(options.vmCpus()), memoryInBytes: options.vmMemoryInBytes())
    let configure = configuredContainer(seed, by: configuration)

    return .awaiting {
      CzLinuxContainer(
        try await self.manager.create(
          id,
          image: image,
          rootfs: rootfs,
          writableLayer: writableLayer,
          networking: networking,
          vm: vm,
          configuration: configure
        )
      )
    }
  }

  func releaseNetwork(id: RustStr) -> CzOutcome {
    let id = id.toString()

    return CzOutcome { try self.manager.releaseNetwork(id) }
  }

  func delete(id: RustStr) -> CzOutcome {
    let id = id.toString()

    return CzOutcome { try self.manager.delete(id) }
  }
}

extension CzOutcome {
  func containerManager() -> CzContainerManager { taken() }
}
