//===----------------------------------------------------------------------===//
// `ContainerManager`, from Containerization.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationExtras
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

/// The manager's configuration closure, as `configuring` makes it.
private func configuredContainer(
  _ seed: RustLinuxContainerConfiguration,
  by configuration: RustConfigure
) -> @Sendable (inout LinuxContainer.Configuration) throws -> Void {
  configuring(
    seed,
    fill: { fill($0, from: $1) },
    call: { configuration.call($0) },
    make: { try LinuxContainer.Configuration($0) }
  )
}

/// What `create` and `createFromReference` read from Rust before they await
/// the manager.
private struct Creation: Sendable {
  let rootfsSizeInBytes: UInt64
  let writableLayerSizeInBytes: UInt64?
  let readOnly: Bool
  let networking: Bool
  let vm: VMResources
  let progress: ProgressHandler?
  let configure: @Sendable (inout LinuxContainer.Configuration) throws -> Void

  init(
    _ options: RustCreateOptions,
    _ progress: RustProgressHandler,
    _ seed: RustLinuxContainerConfiguration,
    _ configuration: RustConfigure
  ) {
    rootfsSizeInBytes = options.rootfsSizeInBytes()
    writableLayerSizeInBytes = options.writableLayerSizeInBytes()
    readOnly = options.readOnly()
    networking = options.networking()
    vm = VMResources(options.vmCpus(), options.vmMemoryInBytes())
    self.progress = progressHandler(progress)
    configure = configuredContainer(seed, by: configuration)
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
    let creation = Creation(options, progress, seed, configuration)

    return .awaiting {
      CzLinuxContainer(
        try await self.manager.create(
          id,
          reference: reference,
          rootfsSizeInBytes: creation.rootfsSizeInBytes,
          writableLayerSizeInBytes: creation.writableLayerSizeInBytes,
          readOnly: creation.readOnly,
          networking: creation.networking,
          vm: creation.vm,
          progress: creation.progress,
          configuration: creation.configure
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
    let creation = Creation(options, progress, seed, configuration)

    return .awaiting {
      CzLinuxContainer(
        try await self.manager.create(
          id,
          image: image,
          rootfsSizeInBytes: creation.rootfsSizeInBytes,
          writableLayerSizeInBytes: creation.writableLayerSizeInBytes,
          readOnly: creation.readOnly,
          networking: creation.networking,
          vm: creation.vm,
          progress: creation.progress,
          configuration: creation.configure
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
    let vm = VMResources(options.vmCpus(), options.vmMemoryInBytes())
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
