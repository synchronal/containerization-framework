//===----------------------------------------------------------------------===//
// `Vminitd`, from Containerization: the client of the agent running in the
// guest, which `VZVirtualMachineInstance.dialAgent()` returns.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationExtras
import ContainerizationOCI
import ContainerizationOS
import Foundation

final class CzVminitd: Sendable {
  let agent: Vminitd

  init(_ agent: Vminitd) {
    self.agent = agent
  }

  /// Another handle on the same agent connection, as a copy of Swift's
  /// struct.
  func duplicate() -> CzVminitd {
    CzVminitd(agent)
  }

  func standardSetup() -> CzOutcome {
    .awaiting { try await self.agent.standardSetup() }
  }

  func close() -> CzOutcome {
    .awaiting { try await self.agent.close() }
  }

  func filesystemOperation(operation: FilesystemOperationKind, path: RustStr, containerID: RustString?) -> CzOutcome {
    let path = path.toString()
    let containerID = containerID?.toString()
    let operation = FilesystemOperation(operation)

    return .awaiting {
      try await self.agent.filesystemOperation(operation: operation, path: path, containerID: containerID)
    }
  }

  func getenv(key: RustStr) -> CzOutcome {
    let key = key.toString()

    return .awaiting { try await self.agent.getenv(key: key) }
  }

  func setenv(key: RustStr, value: RustStr) -> CzOutcome {
    let key = key.toString()
    let value = value.toString()

    return .awaiting { try await self.agent.setenv(key: key, value: value) }
  }

  func mount(mount: RustOciMount) -> CzOutcome {
    let mount = ContainerizationOCI.Mount(mount)

    return .awaiting { try await self.agent.mount(mount) }
  }

  func umount(path: RustStr, flags: Int32) -> CzOutcome {
    let path = path.toString()

    return .awaiting { try await self.agent.umount(path: path, flags: flags) }
  }

  func mkdir(path: RustStr, all: Bool, perms: UInt32) -> CzOutcome {
    let path = path.toString()

    return .awaiting { try await self.agent.mkdir(path: path, all: all, perms: perms) }
  }

  func kill(pid: Int32, signal: Int32) -> CzOutcome {
    .awaiting { try await self.agent.kill(pid: pid, signal: signal) }
  }

  func sync() -> CzOutcome {
    .awaiting { try await self.agent.sync() }
  }

  /// `options` stands for `nil` when `hasOptions` is false.
  func createProcess(
    id: RustStr,
    containerID: RustString?,
    stdinPort: UInt32?,
    stdoutPort: UInt32?,
    stderrPort: UInt32?,
    ociRuntimePath: RustString?,
    configuration: RustSpec,
    hasOptions: Bool,
    options: RustVec<UInt8>
  ) -> CzOutcome {
    let id = id.toString()
    let containerID = containerID?.toString()
    let ociRuntimePath = ociRuntimePath?.toString()
    let configuration = ContainerizationOCI.Spec(configuration)
    let options = hasOptions ? Data(options) : nil

    return .awaiting {
      try await self.agent.createProcess(
        id: id,
        containerID: containerID,
        stdinPort: stdinPort,
        stdoutPort: stdoutPort,
        stderrPort: stderrPort,
        ociRuntimePath: ociRuntimePath,
        configuration: configuration,
        options: options
      )
    }
  }

  func startProcess(id: RustStr, containerID: RustString?) -> CzOutcome {
    let id = id.toString()
    let containerID = containerID?.toString()

    return .awaiting { try await self.agent.startProcess(id: id, containerID: containerID) }
  }

  func signalProcess(id: RustStr, containerID: RustString?, signal: Int32) -> CzOutcome {
    let id = id.toString()
    let containerID = containerID?.toString()

    return .awaiting { try await self.agent.signalProcess(id: id, containerID: containerID, signal: signal) }
  }

  func resizeProcess(id: RustStr, containerID: RustString?, columns: UInt32, rows: UInt32) -> CzOutcome {
    let id = id.toString()
    let containerID = containerID?.toString()

    return .awaiting {
      try await self.agent.resizeProcess(id: id, containerID: containerID, columns: columns, rows: rows)
    }
  }

  func waitProcess(id: RustStr, containerID: RustString?, timeoutInSeconds: Int64?) -> CzOutcome {
    let id = id.toString()
    let containerID = containerID?.toString()

    return .awaiting {
      try await self.agent.waitProcess(id: id, containerID: containerID, timeoutInSeconds: timeoutInSeconds)
    }
  }

  func deleteProcess(id: RustStr, containerID: RustString?) -> CzOutcome {
    let id = id.toString()
    let containerID = containerID?.toString()

    return .awaiting { try await self.agent.deleteProcess(id: id, containerID: containerID) }
  }

  func closeProcessStdin(id: RustStr, containerID: RustString?) -> CzOutcome {
    let id = id.toString()
    let containerID = containerID?.toString()

    return .awaiting { try await self.agent.closeProcessStdin(id: id, containerID: containerID) }
  }

  func up(name: RustStr, mtu: UInt32?) -> CzOutcome {
    let name = name.toString()

    return .awaiting { try await self.agent.up(name: name, mtu: mtu) }
  }

  func down(name: RustStr) -> CzOutcome {
    let name = name.toString()

    return .awaiting { try await self.agent.down(name: name) }
  }

  func addressAdd(name: RustStr, address: RustInterfaceAddress) -> CzOutcome {
    let agent = agent
    let name = name.toString()

    return CzOutcome {
      let address = try InterfaceAddress(address)

      return try blocking { try await agent.addressAdd(name: name, address: address) }
    }
  }

  func routeAddLink(name: RustStr, route: RustLinkRoute) -> CzOutcome {
    let name = name.toString()
    let route = LinkRoute(route)

    return .awaiting { try await self.agent.routeAddLink(name: name, route: route) }
  }

  func routeAddDefault(name: RustStr, route: RustDefaultRoute) -> CzOutcome {
    let name = name.toString()
    let route = DefaultRoute(route)

    return .awaiting { try await self.agent.routeAddDefault(name: name, route: route) }
  }

  func configureDNS(config: RustDns, location: RustStr) -> CzOutcome {
    let config = DNS(config)
    let location = location.toString()

    return .awaiting { try await self.agent.configureDNS(config: config, location: location) }
  }

  func configureHosts(config: RustHosts, location: RustStr) -> CzOutcome {
    let config = Hosts(config)
    let location = location.toString()

    return .awaiting { try await self.agent.configureHosts(config: config, location: location) }
  }

  func containerStatistics(containerIDs: RustVec<RustString>, categories: Int64) -> CzOutcome {
    let containerIDs = strings(containerIDs)
    let categories = StatCategory(rawValue: Int(categories))

    return .awaiting { try await self.agent.containerStatistics(containerIDs: containerIDs, categories: categories) }
  }

  func setupEmulator(binaryPath: RustStr, configuration: RustBinfmtEntry) -> CzOutcome {
    let binaryPath = binaryPath.toString()
    // `Binfmt.Entry` isn't `Sendable`, though it holds only `String`s.
    nonisolated(unsafe) let configuration = Binfmt.Entry(configuration)

    return .awaiting { try await self.agent.setupEmulator(binaryPath: binaryPath, configuration: configuration) }
  }

  func setTime(sec: Int64, usec: Int32) -> CzOutcome {
    .awaiting { try await self.agent.setTime(sec: sec, usec: usec) }
  }

  func sysctl(keys: RustVec<RustString>, values: RustVec<RustString>) -> CzOutcome {
    let settings = dictionary(keys, values)

    return .awaiting { try await self.agent.sysctl(settings: settings) }
  }

  func stat(root: RustStr, path: RustStr) -> CzOutcome {
    let root = root.toString()
    let path = path.toString()

    return .awaiting { try await self.agent.stat(root: root, path: path) }
  }

  func enableRosetta() -> CzOutcome {
    .awaiting { try await self.agent.enableRosetta() }
  }

  func relaySocket(port: UInt32, configuration: CzUnixSocketConfiguration) -> CzOutcome {
    let configuration = configuration.configuration

    return .awaiting { try await self.agent.relaySocket(port: port, configuration: configuration) }
  }

  func stopSocketRelay(configuration: CzUnixSocketConfiguration) -> CzOutcome {
    let configuration = configuration.configuration

    return .awaiting { try await self.agent.stopSocketRelay(configuration: configuration) }
  }
}

extension CzVirtualMachineInstance {
  func dialAgent() -> CzOutcome {
    .awaiting { CzVminitd(try await self.instance.dialAgent()) }
  }
}

extension CzOutcome {
  func vminitd() -> CzVminitd { taken() }
}
