//===----------------------------------------------------------------------===//
// Rust's virtual machine values read into Containerization's: the VM's
// configuration, boot log, kernel, platform and attached filesystems, and a
// `Binfmt.Entry`. The Rust side is `bridge::vm` and
// `bridge::accessors::container::vm`.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationOS
import Foundation

// MARK: Rust to Swift

extension BootLog {
  static func from(_ bootLog: RustBootLogRef) throws -> BootLog {
    switch bootLog.kind() {
    case .File:
      .file(path: try lentPath(bootLog.path()?.toString(), "boot log", bootLog.lossyPath), append: bootLog.append())
    case .FileHandle: .fileHandle(try duplicate(bootLog: bootLog.fileHandle()))
    }
  }
}

extension VMResources {
  /// The resources Rust passes as a vCPU count and a memory size.
  init(_ cpus: UInt32, _ memoryInBytes: UInt64) {
    self.init(cpus: Int(cpus), memoryInBytes: memoryInBytes)
  }
}

extension VMConfiguration {
  init(_ configuration: RustVmConfigurationRef) throws {
    self.init(
      cpus: Int(configuration.cpus()),
      memoryInBytes: configuration.memoryInBytes(),
      interfaces: try lentInterfaces(
        configuration.interfacesLen(),
        configuration.interfaceKindAt,
        configuration.natInterfaceAt,
        configuration.vmnetInterfaceAt
      ),
      mountsByID: Dictionary(
        list(configuration.mountsByIdLen()) { index in
          (
            configuration.mountsByIdKeyAt(index).toString(),
            list(configuration.mountsByIdMountsLen(index)) {
              Containerization.Mount(configuration.mountsByIdMountAt(index, $0))
            }
          )
        },
        uniquingKeysWith: { _, last in last }
      ),
      bootLog: configuration.hasBootLog() ? try BootLog.from(configuration.bootLog()) : nil,
      nestedVirtualization: configuration.nestedVirtualization()
    )
  }
}

extension AttachedFilesystem {
  init(_ filesystem: RustAttachedFilesystemRef) {
    self.init(
      type: filesystem.filesystemType().toString(),
      source: filesystem.source().toString(),
      destination: filesystem.destination().toString(),
      options: strings(filesystem.optionsLen(), filesystem.optionsAt)
    )
  }
}

extension SystemPlatform {
  /// Decoded, since its memberwise init is internal.
  init(_ platform: RustSystemPlatformRef) throws {
    let os: OS =
      switch platform.os() {
      case .Linux: .linux
      case .Darwin: .darwin
      }
    let architecture: Architecture =
      switch platform.architecture() {
      case .Arm64: .arm64
      case .Amd64: .amd64
      }

    self = try JSONDecoder().decode(
      SystemPlatform.self,
      from: Data(#"{"os":"\#(os.rawValue)","architecture":"\#(architecture.rawValue)"}"#.utf8)
    )
  }
}

extension Kernel {
  init(_ kernel: RustKernelRef) throws {
    self.init(
      path: try lentPath(kernel.path()?.toString(), "kernel", kernel.lossyPath),
      platform: try SystemPlatform(kernel.platform()),
      commandline: CommandLine(
        kernelArgs: strings(kernel.kernelArgsLen(), kernel.kernelArgsAt),
        initArgs: strings(kernel.initArgsLen(), kernel.initArgsAt)
      )
    )
  }
}

extension Binfmt.Entry {
  init(_ entry: RustBinfmtEntryRef) {
    self.init(
      name: entry.name().toString(),
      type: entry.entryType().toString(),
      offset: entry.offset().toString(),
      magic: entry.magic().toString(),
      mask: entry.mask().toString(),
      flags: entry.flags().toString()
    )
  }
}

// MARK: Swift to Rust

/// What a `BootLog` writes to. Containerization keeps it internal, so it is
/// read by reflection.
enum BootLogRepresentation {
  case file(URL, Bool)
  case fileHandle(FileHandle)
}

func representation(of bootLog: BootLog) -> BootLogRepresentation {
  let base = Mirror(reflecting: bootLog).children.first { $0.label == "base" }!.value
  let representation = Mirror(reflecting: base).children.first!

  switch representation.value {
  case let (path, append) as (URL, Bool):
    return .file(path, append)
  case let handle as FileHandle:
    return .fileHandle(handle)
  default:
    preconditionFailure("BootLog holds \(representation.label ?? "something") this bridge doesn't know")
  }
}

// An `AttachedFilesystem`, field by field: Rust builds its own.
extension CzOutcome {
  func attachedFilesystemType() -> String {
    (taken() as AttachedFilesystem).type
  }

  func attachedFilesystemSource() -> String {
    (taken() as AttachedFilesystem).source
  }

  func attachedFilesystemDestination() -> String {
    (taken() as AttachedFilesystem).destination
  }

  func attachedFilesystemOptions() -> RustVec<RustString> {
    rustStrings((taken() as AttachedFilesystem).options)
  }
}
