// swift-tools-version: 6.2

import Foundation
import PackageDescription

// Scoped to this target: a `-Xswiftc` flag would apply to every target, making
// swift-bridge's regenerated header an input to swift-nio, gRPC and
// Containerization, so each Rust bridge change costs a minute of rebuilding.
//
// Absolute, because the flag reaches the compiler verbatim and its working
// directory is unspecified.
let bridgingHeader =
  "\(URL(fileURLWithPath: #filePath).deletingLastPathComponent().path)/Sources/ContainerizationBridge/bridging-header.h"

// Pinned exactly: the init image that `versions::INIT_IMAGE` names carries a
// guest agent speaking this release's protocol. Any other library is a
// runtime mismatch, not a compile error.
let containerization: Version = "0.53.0"

let package = Package(
  name: "ContainerizationBridge",
  platforms: [.macOS("26.0")],
  products: [
    .library(
      name: "ContainerizationBridge",
      type: .static,
      targets: ["ContainerizationBridge"]
    )
  ],
  dependencies: [
    .package(url: "https://github.com/apple/containerization.git", exact: containerization),
    // Containerization's own requirements: the bridge imports all three.
    .package(url: "https://github.com/apple/swift-crypto.git", "3.0.0"..<"5.0.0"),
    .package(url: "https://github.com/apple/swift-log.git", from: "1.10.1"),
    .package(url: "https://github.com/apple/swift-system.git", from: "1.6.4"),
  ],
  targets: [
    .target(
      name: "ContainerizationBridge",
      dependencies: [
        .product(name: "CloudHypervisor", package: "containerization"),
        .product(name: "Containerization", package: "containerization"),
        .product(name: "ContainerizationArchive", package: "containerization"),
        .product(name: "ContainerizationEXT4", package: "containerization"),
        .product(name: "ContainerizationExtras", package: "containerization"),
        .product(name: "ContainerizationIO", package: "containerization"),
        .product(name: "ContainerizationOCI", package: "containerization"),
        .product(name: "ContainerizationOS", package: "containerization"),
        .product(name: "Crypto", package: "swift-crypto"),
        .product(name: "Logging", package: "swift-log"),
        .product(name: "SystemPackage", package: "swift-system"),
      ],
      swiftSettings: [.unsafeFlags(["-import-objc-header", bridgingHeader])]
    )
  ]
)
