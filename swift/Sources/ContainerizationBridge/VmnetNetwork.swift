//===----------------------------------------------------------------------===//
// `VmnetNetwork` and its `Interface`, from Containerization, and the
// `Network?` the manager's inits take.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationExtras
import vmnet

/// A `Network?`.
final class CzNetwork: Sendable {
  let network: (any Network)?

  init(_ network: (any Network)?) {
    self.network = network
  }
}

func noNetwork() -> CzNetwork {
  CzNetwork(nil)
}

/// Unchecked: the interface methods mutate `network`, and Rust's `&mut self`
/// lets only one run at a time.
final class CzVmnetNetwork: @unchecked Sendable {
  var network: VmnetNetwork

  init(_ network: VmnetNetwork) {
    self.network = network
  }

  /// A copy, as Swift passes it to the manager.
  func asNetwork() -> CzNetwork {
    CzNetwork(network)
  }

  func subnet() -> CzOutcome {
    CzOutcome.holding(network.subnet)
  }

  func prefixV6() -> CzOutcome {
    CzOutcome.holding(network.prefixV6)
  }

  func ipv4Gateway() -> CzOutcome {
    CzOutcome.holding(network.ipv4Gateway)
  }

  func ipv6Gateway() -> CzOutcome {
    CzOutcome.holding(network.ipv6Gateway)
  }

  func createInterface(id: RustStr) -> CzOutcome {
    let id = id.toString()

    return CzOutcome { absent(try vmnetInterface(network.createInterface(id))) }
  }

  func createInterface(id: RustStr, mtu: UInt32) -> CzOutcome {
    let id = id.toString()

    return CzOutcome { absent(try vmnetInterface(network.createInterface(id, mtu: mtu))) }
  }

  func createInterfaceWithoutGateway(id: RustStr) -> CzOutcome {
    let id = id.toString()

    return CzOutcome { absent(try vmnetInterface(network.createInterfaceWithoutGateway(id))) }
  }

  func releaseInterface(id: RustStr) -> CzOutcome {
    let id = id.toString()

    return CzOutcome { try network.releaseInterface(id) }
  }
}

/// What `VmnetNetwork`'s `create` methods return, which is always one of its
/// own interfaces.
private func vmnetInterface(_ interface: (any Interface)?) -> CzVmnetInterface? {
  interface.map { interface in
    guard let interface = interface as? VmnetNetwork.Interface else {
      preconditionFailure("VmnetNetwork made a \(type(of: interface))")
    }
    return CzVmnetInterface(interface)
  }
}

/// Public because swift-bridge's glue for a Rust method that takes one is.
public final class CzVmnetInterface: Sendable {
  let interface: VmnetNetwork.Interface

  init(_ interface: VmnetNetwork.Interface) {
    self.interface = interface
  }

  func duplicate() -> CzVmnetInterface {
    CzVmnetInterface(interface)
  }

  func ipv4Address() -> CzOutcome {
    CzOutcome.holding(interface.ipv4Address)
  }

  func ipv4Gateway() -> CzOutcome {
    CzOutcome.holding(interface.ipv4Gateway)
  }

  func ipv6Address() -> CzOutcome {
    CzOutcome.holding(interface.ipv6Address)
  }

  func ipv6Gateway() -> CzOutcome {
    CzOutcome.holding(interface.ipv6Gateway)
  }

  func macAddress() -> CzOutcome {
    CzOutcome.holding(interface.macAddress)
  }

  func mtu() -> UInt32 {
    interface.mtu
  }
}

func vmnetNetwork(
  mode: VmnetMode,
  subnetAddress: UInt32?,
  subnetPrefix: UInt8,
  hasPrefixV6: Bool,
  prefixV6Address: RustIPv6Address,
  prefixV6Length: UInt8
) -> CzOutcome {
  let mode: operating_modes_t =
    switch mode {
    case .Shared: .VMNET_SHARED_MODE
    case .Host: .VMNET_HOST_MODE
    case .Bridged: .VMNET_BRIDGED_MODE
    }

  return CzOutcome {
    let subnet = try subnetAddress.map { try CIDRv4(bridged: $0, prefix: subnetPrefix) }
    let prefixV6 = hasPrefixV6 ? try CIDRv6(bridged: prefixV6Address, prefix: prefixV6Length) : nil

    return CzVmnetNetwork(try VmnetNetwork(mode: mode, subnet: subnet, prefixV6: prefixV6))
  }
}
