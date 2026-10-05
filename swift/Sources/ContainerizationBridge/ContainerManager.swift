//===----------------------------------------------------------------------===//
// `ContainerManager`, from Containerization, with no `Network`.
//===----------------------------------------------------------------------===//

import Containerization
import Foundation

// Made from the store they take, which swift-bridge can't pass as an argument.
extension CzImageStore {
  func containerManager(
    kernel: RustKernel,
    initfs: RustMount,
    rosetta: Bool,
    nestedVirtualization: Bool
  ) -> CzOutcome {
    let initfs = Containerization.Mount(initfs)
    let store = store

    return CzOutcome {
      CzContainerManager(
        try ContainerManager(
          kernel: try Kernel(kernel),
          initfs: initfs,
          imageStore: store,
          rosetta: rosetta,
          nestedVirtualization: nestedVirtualization
        )
      )
    }
  }

  func containerManager(
    kernel: RustKernel,
    initfsReference: RustStr,
    rosetta: Bool,
    nestedVirtualization: Bool
  ) -> CzOutcome {
    let reference = initfsReference.toString()
    let store = store

    return CzOutcome {
      let kernel = try Kernel(kernel)

      return CzContainerManager(
        try blocking {
          try await ContainerManager(
            kernel: kernel,
            initfsReference: reference,
            imageStore: store,
            rosetta: rosetta,
            nestedVirtualization: nestedVirtualization
          )
        }
      )
    }
  }
}

/// Unchecked: `create` and `delete` mutate `manager`, and Rust's `&mut self`
/// lets only one run at a time.
final class CzContainerManager: @unchecked Sendable {
  var manager: ContainerManager

  init(_ manager: ContainerManager) {
    self.manager = manager
  }

  /// `configuration` is Rust's: it gets the seeded configuration and returns it,
  /// changed. It runs inside the manager's closure, so on Swift's thread.
  ///
  /// `seed` is Rust's to fill with what the manager seeded: Swift never makes a
  /// Rust value.
  func create(
    id: RustStr,
    image: CzImage,
    options: RustCreateOptions,
    seed: RustLinuxContainerConfiguration,
    configuration: (RustLinuxContainerConfiguration) -> RustLinuxContainerConfiguration
  ) -> CzOutcome {
    let id = id.toString()
    let image = image.image
    let rootfsSizeInBytes = options.rootfsSizeInBytes()
    let writableLayerSizeInBytes = options.writableLayerSizeInBytes()
    let readOnly = options.readOnly()
    let networking = options.networking()
    let vm = VMResources(cpus: Int(options.vmCpus()), memoryInBytes: options.vmMemoryInBytes())

    return CzOutcome {
      // Doesn't escape: `blocking` returns only once the manager is done.
      try withoutActuallyEscaping(configuration) { configuration in
        nonisolated(unsafe) let configuration = configuration
        nonisolated(unsafe) let seed = seed

        return CzLinuxContainer(
          try blocking {
            try await self.manager.create(
              id,
              image: image,
              rootfsSizeInBytes: rootfsSizeInBytes,
              writableLayerSizeInBytes: writableLayerSizeInBytes,
              readOnly: readOnly,
              networking: networking,
              vm: vm
            ) { config in
              try fill(seed, from: config)
              config = try LinuxContainer.Configuration(configuration(seed))
            }
          }
        )
      }
    }
  }

  func delete(id: RustStr) -> CzOutcome {
    let id = id.toString()

    return CzOutcome { try self.manager.delete(id) }
  }
}
