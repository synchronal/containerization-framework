//===----------------------------------------------------------------------===//
// The address types, from ContainerizationExtras: Rust's read into Swift's,
// and what Swift computes from them.
//
// IPv4 and MAC addresses cross as their values, and a prefix as its length.
// IPv6 and IP addresses cross as opaque Rust types. Rust builds a result back
// field by field, through `CzOutcome`'s getters.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationExtras
import Foundation

// MARK: Rust to Swift

extension IPv6Address {
  init(_ address: RustIPv6AddressRef) {
    self.init(UInt128(address.valueHigh()) << 64 | UInt128(address.valueLow()), zone: address.zone()?.toString())
  }
}

extension IPAddress {
  init(_ address: RustIpAddressRef) {
    self = address.holdsV6() ? .v6(IPv6Address(address.v6())) : .v4(IPv4Address(address.v4Value()))
  }
}

extension Prefix {
  /// A length Rust holds, which Swift made.
  init(bridged length: UInt8) throws {
    guard let prefix = Prefix(length: length) else {
      throw BridgeError.malformed("prefix length", String(length))
    }
    self = prefix
  }
}

extension CIDRv4 {
  init(bridged address: UInt32, prefix: UInt8) throws {
    try self.init(IPv4Address(address), prefix: Prefix(bridged: prefix))
  }
}

extension CIDRv6 {
  init(bridged address: RustIPv6AddressRef, prefix: UInt8) throws {
    try self.init(IPv6Address(address), prefix: Prefix(bridged: prefix))
  }
}

extension CIDR {
  /// The case Rust holds, made directly as Rust's was.
  init(bridged address: RustIpAddressRef, prefix: UInt8) throws {
    let prefix = try Prefix(bridged: prefix)
    self =
      switch IPAddress(address) {
      case .v4(let address): .v4(address, prefix)
      case .v6(let address): .v6(address, prefix)
      }
  }
}

// MARK: IPv4Address

func ipv4AddressFromBytes(bytes: RustVec<UInt8>) -> CzOutcome {
  let bytes = bytes.map { $0 }

  return CzOutcome { try IPv4Address(bytes) }
}

func parseIPv4Address(string: RustStr) -> CzOutcome {
  let string = string.toString()

  return CzOutcome { try IPv4Address(string) }
}

func ipv4AddressBytes(value: UInt32) -> CzOutcome {
  CzOutcome { IPv4Address(value).bytes }
}

func ipv4AddressDescription(value: UInt32) -> CzOutcome {
  CzOutcome { IPv4Address(value).description }
}

func ipv4AddressIsUnspecified(value: UInt32) -> CzOutcome {
  CzOutcome { IPv4Address(value).isUnspecified }
}

func ipv4AddressIsLoopback(value: UInt32) -> CzOutcome {
  CzOutcome { IPv4Address(value).isLoopback }
}

func ipv4AddressIsMulticast(value: UInt32) -> CzOutcome {
  CzOutcome { IPv4Address(value).isMulticast }
}

func ipv4AddressIsLinkLocal(value: UInt32) -> CzOutcome {
  CzOutcome { IPv4Address(value).isLinkLocal }
}

func ipv4AddressIsBroadcast(value: UInt32) -> CzOutcome {
  CzOutcome { IPv4Address(value).isBroadcast }
}

func ipv4AddressLessThan(lhs: UInt32, rhs: UInt32) -> CzOutcome {
  CzOutcome { IPv4Address(lhs) < IPv4Address(rhs) }
}

// MARK: IPv6Address

func parseIPv6Address(address: RustStr) -> CzOutcome {
  let address = address.toString()

  return CzOutcome { try IPv6Address(address) }
}

func ipv6AddressFromBytes(bytes: RustVec<UInt8>, zone: RustString?) -> CzOutcome {
  let bytes = bytes.map { $0 }
  let zone = zone?.toString()

  return CzOutcome { try IPv6Address(bytes, zone: zone) }
}

func ipv6AddressUnspecified() -> CzOutcome {
  CzOutcome { IPv6Address.unspecified }
}

func ipv6AddressLoopback() -> CzOutcome {
  CzOutcome { IPv6Address.loopback }
}

func ipv6AddressBytes(address: RustIPv6Address) -> CzOutcome {
  let address = IPv6Address(address)

  return CzOutcome { address.bytes }
}

func ipv6AddressDescription(address: RustIPv6Address) -> CzOutcome {
  let address = IPv6Address(address)

  return CzOutcome { address.description }
}

func ipv6AddressIsUnspecified(address: RustIPv6Address) -> CzOutcome {
  let address = IPv6Address(address)

  return CzOutcome { address.isUnspecified }
}

func ipv6AddressIsLoopback(address: RustIPv6Address) -> CzOutcome {
  let address = IPv6Address(address)

  return CzOutcome { address.isLoopback }
}

func ipv6AddressIsMulticast(address: RustIPv6Address) -> CzOutcome {
  let address = IPv6Address(address)

  return CzOutcome { address.isMulticast }
}

func ipv6AddressIsLinkLocal(address: RustIPv6Address) -> CzOutcome {
  let address = IPv6Address(address)

  return CzOutcome { address.isLinkLocal }
}

func ipv6AddressIsUniqueLocal(address: RustIPv6Address) -> CzOutcome {
  let address = IPv6Address(address)

  return CzOutcome { address.isUniqueLocal }
}

func ipv6AddressIsGlobalUnicast(address: RustIPv6Address) -> CzOutcome {
  let address = IPv6Address(address)

  return CzOutcome { address.isGlobalUnicast }
}

func ipv6AddressIsDocumentation(address: RustIPv6Address) -> CzOutcome {
  let address = IPv6Address(address)

  return CzOutcome { address.isDocumentation }
}

func ipv6AddressLessThan(lhs: RustIPv6Address, rhs: RustIPv6Address) -> CzOutcome {
  let lhs = IPv6Address(lhs)
  let rhs = IPv6Address(rhs)

  return CzOutcome { lhs < rhs }
}

// MARK: IPAddress

func parseIPAddress(string: RustStr) -> CzOutcome {
  let string = string.toString()

  return CzOutcome { try IPAddress(string) }
}

func ipAddressDescription(address: RustIpAddress) -> CzOutcome {
  let address = IPAddress(address)

  return CzOutcome { address.description }
}

func ipAddressIsV4(address: RustIpAddress) -> CzOutcome {
  let address = IPAddress(address)

  return CzOutcome { address.isV4 }
}

func ipAddressIsV6(address: RustIpAddress) -> CzOutcome {
  let address = IPAddress(address)

  return CzOutcome { address.isV6 }
}

func ipAddressIPv4(address: RustIpAddress) -> CzOutcome {
  let address = IPAddress(address)

  return CzOutcome { absent(address.ipv4) }
}

func ipAddressIPv6(address: RustIpAddress) -> CzOutcome {
  let address = IPAddress(address)

  return CzOutcome { absent(address.ipv6) }
}

func ipAddressIsLoopback(address: RustIpAddress) -> CzOutcome {
  let address = IPAddress(address)

  return CzOutcome { address.isLoopback }
}

func ipAddressIsMulticast(address: RustIpAddress) -> CzOutcome {
  let address = IPAddress(address)

  return CzOutcome { address.isMulticast }
}

func ipAddressIsUnspecified(address: RustIpAddress) -> CzOutcome {
  let address = IPAddress(address)

  return CzOutcome { address.isUnspecified }
}

// MARK: Prefix

func prefixWithLength(length: UInt8) -> CzOutcome {
  CzOutcome { absent(Prefix(length: length)) }
}

func prefixIPv4(length: UInt8) -> CzOutcome {
  CzOutcome { absent(Prefix.ipv4(length)) }
}

func prefixIPv6(length: UInt8) -> CzOutcome {
  CzOutcome { absent(Prefix.ipv6(length)) }
}

func prefixDescription(length: UInt8) -> CzOutcome {
  CzOutcome { try Prefix(bridged: length).description }
}

/// As a `UInt64`, for `number`.
func prefixSuffixMask32(length: UInt8) -> CzOutcome {
  CzOutcome { UInt64(try Prefix(bridged: length).suffixMask32) }
}

/// As a `UInt64`, for `number`.
func prefixPrefixMask32(length: UInt8) -> CzOutcome {
  CzOutcome { UInt64(try Prefix(bridged: length).prefixMask32) }
}

func prefixSuffixMask128(length: UInt8) -> CzOutcome {
  CzOutcome { try Prefix(bridged: length).suffixMask128 }
}

func prefixPrefixMask128(length: UInt8) -> CzOutcome {
  CzOutcome { try Prefix(bridged: length).prefixMask128 }
}

// MARK: CIDRv4

func parseCIDRv4(cidr: RustStr) -> CzOutcome {
  let cidr = cidr.toString()

  return CzOutcome { try CIDRv4(cidr) }
}

func cidrV4(address: UInt32, prefix: UInt8) -> CzOutcome {
  CzOutcome { try CIDRv4(bridged: address, prefix: prefix) }
}

func cidrV4FromRange(lower: UInt32, upper: UInt32) -> CzOutcome {
  CzOutcome { try CIDRv4(lower: IPv4Address(lower), upper: IPv4Address(upper)) }
}

func cidrV4Lower(address: UInt32, prefix: UInt8) -> CzOutcome {
  CzOutcome { try CIDRv4(bridged: address, prefix: prefix).lower }
}

func cidrV4Upper(address: UInt32, prefix: UInt8) -> CzOutcome {
  CzOutcome { try CIDRv4(bridged: address, prefix: prefix).upper }
}

func cidrV4Gateway(address: UInt32, prefix: UInt8) -> CzOutcome {
  CzOutcome { try CIDRv4(bridged: address, prefix: prefix).gateway }
}

func cidrV4Contains(address: UInt32, prefix: UInt8, ip: UInt32) -> CzOutcome {
  CzOutcome { try CIDRv4(bridged: address, prefix: prefix).contains(IPv4Address(ip)) }
}

func cidrV4Description(address: UInt32, prefix: UInt8) -> CzOutcome {
  CzOutcome { try CIDRv4(bridged: address, prefix: prefix).description }
}

// MARK: CIDRv6

func parseCIDRv6(cidr: RustStr) -> CzOutcome {
  let cidr = cidr.toString()

  return CzOutcome { try CIDRv6(cidr) }
}

func cidrV6(address: RustIPv6Address, prefix: UInt8) -> CzOutcome {
  CzOutcome { try CIDRv6(bridged: address, prefix: prefix) }
}

func cidrV6FromRange(lower: RustIPv6Address, upper: RustIPv6Address) -> CzOutcome {
  let lower = IPv6Address(lower)
  let upper = IPv6Address(upper)

  return CzOutcome { try CIDRv6(lower: lower, upper: upper) }
}

func cidrV6Lower(address: RustIPv6Address, prefix: UInt8) -> CzOutcome {
  CzOutcome { try CIDRv6(bridged: address, prefix: prefix).lower }
}

func cidrV6Upper(address: RustIPv6Address, prefix: UInt8) -> CzOutcome {
  CzOutcome { try CIDRv6(bridged: address, prefix: prefix).upper }
}

/// Containerization's extension, not ContainerizationExtras'.
func cidrV6Gateway(address: RustIPv6Address, prefix: UInt8) -> CzOutcome {
  CzOutcome { try CIDRv6(bridged: address, prefix: prefix).gateway }
}

func cidrV6Contains(address: RustIPv6Address, prefix: UInt8, ip: RustIPv6Address) -> CzOutcome {
  let ip = IPv6Address(ip)

  return CzOutcome { try CIDRv6(bridged: address, prefix: prefix).contains(ip) }
}

func cidrV6Description(address: RustIPv6Address, prefix: UInt8) -> CzOutcome {
  CzOutcome { try CIDRv6(bridged: address, prefix: prefix).description }
}

// MARK: CIDR

func parseCIDR(cidr: RustStr) -> CzOutcome {
  let cidr = cidr.toString()

  return CzOutcome { try CIDR(cidr) }
}

func cidr(address: RustIpAddress, prefix: UInt8) -> CzOutcome {
  let address = IPAddress(address)

  return CzOutcome { try CIDR(address, prefix: Prefix(bridged: prefix)) }
}

func cidrFromRange(lower: RustIpAddress, upper: RustIpAddress) -> CzOutcome {
  let lower = IPAddress(lower)
  let upper = IPAddress(upper)

  return CzOutcome { try CIDR(lower: lower, upper: upper) }
}

func cidrAddress(address: RustIpAddress, prefix: UInt8) -> CzOutcome {
  CzOutcome { try CIDR(bridged: address, prefix: prefix).address }
}

func cidrPrefix(address: RustIpAddress, prefix: UInt8) -> CzOutcome {
  CzOutcome { try CIDR(bridged: address, prefix: prefix).prefix }
}

func cidrLower(address: RustIpAddress, prefix: UInt8) -> CzOutcome {
  CzOutcome { try CIDR(bridged: address, prefix: prefix).lower }
}

func cidrUpper(address: RustIpAddress, prefix: UInt8) -> CzOutcome {
  CzOutcome { try CIDR(bridged: address, prefix: prefix).upper }
}

func cidrContains(address: RustIpAddress, prefix: UInt8, ip: RustIpAddress) -> CzOutcome {
  let ip = IPAddress(ip)

  return CzOutcome { try CIDR(bridged: address, prefix: prefix).contains(ip) }
}

func cidrDescription(address: RustIpAddress, prefix: UInt8) -> CzOutcome {
  CzOutcome { try CIDR(bridged: address, prefix: prefix).description }
}

// MARK: MACAddress

func macAddress(value: UInt64) -> CzOutcome {
  CzOutcome { MACAddress(value) }
}

func macAddressFromBytes(bytes: RustVec<UInt8>) -> CzOutcome {
  let bytes = bytes.map { $0 }

  return CzOutcome { try MACAddress(bytes) }
}

func parseMACAddress(string: RustStr) -> CzOutcome {
  let string = string.toString()

  return CzOutcome { try MACAddress(string) }
}

func macAddressBytes(value: UInt64) -> CzOutcome {
  CzOutcome { MACAddress(value).bytes }
}

func macAddressDescription(value: UInt64) -> CzOutcome {
  CzOutcome { MACAddress(value).description }
}

func macAddressIsLocallyAdministered(value: UInt64) -> CzOutcome {
  CzOutcome { MACAddress(value).isLocallyAdministered }
}

func macAddressIsMulticast(value: UInt64) -> CzOutcome {
  CzOutcome { MACAddress(value).isMulticast }
}

func macAddressIPv6Address(value: UInt64, network: RustIPv6Address) -> CzOutcome {
  let network = IPv6Address(network)

  return CzOutcome { try MACAddress(value).ipv6Address(network: network) }
}

func macAddressLessThan(lhs: UInt64, rhs: UInt64) -> CzOutcome {
  CzOutcome { MACAddress(lhs) < MACAddress(rhs) }
}
