//===----------------------------------------------------------------------===//
// `RegistryClient`, from ContainerizationOCI.
//===----------------------------------------------------------------------===//

import ContainerizationOCI
import Foundation

func newRegistryClient(reference: RustStr, insecure: Bool, auth: CzAuthentication) -> CzOutcome {
  let reference = reference.toString()
  let auth = auth.authentication

  return CzOutcome { CzRegistryClient(try RegistryClient(reference: reference, insecure: insecure, auth: auth)) }
}

func registryClientWithHost(
  host: RustStr,
  scheme: RustString?,
  port: UInt16?,
  authentication: CzAuthentication,
  clientID: RustString?,
  hasRetryOptions: Bool,
  maxRetries: Int,
  retryInterval: UInt64,
  bufferSize: UInt
) -> CzOutcome {
  let host = host.toString()
  let scheme = scheme?.toString()
  let authentication = authentication.authentication
  let clientID = clientID?.toString()
  let retryOptions = hasRetryOptions ? RetryOptions(maxRetries: maxRetries, retryInterval: retryInterval) : nil

  return CzOutcome {
    CzRegistryClient(
      RegistryClient(
        host: host,
        scheme: scheme,
        port: port.map { Int($0) },
        authentication: authentication,
        clientID: clientID,
        retryOptions: retryOptions,
        bufferSize: try int(bufferSize, "buffer size")
      ))
  }
}

final class CzRegistryClient: Sendable {
  let client: RegistryClient

  init(_ client: RegistryClient) {
    self.client = client
  }

  func ping() -> CzOutcome {
    .awaiting { try await self.client.ping() }
  }

  func resolve(name: RustStr, tag: RustStr) -> CzOutcome {
    let name = name.toString()
    let tag = tag.toString()

    return .awaiting { try await self.client.resolve(name: name, tag: tag) }
  }

  func fetchData(name: RustStr, descriptor: RustDescriptor) -> CzOutcome {
    let name = name.toString()
    let descriptor = Descriptor(descriptor)

    return .awaiting { try await self.client.fetchData(name: name, descriptor: descriptor) }
  }

  func fetchBlob(name: RustStr, descriptor: RustDescriptor, into: RustStr, progress: RustProgressHandler) -> CzOutcome {
    let name = name.toString()
    let descriptor = Descriptor(descriptor)
    let file = URL(filePath: into.toString())
    let progress = progressHandler(progress)
    let client = client

    return CzOutcome {
      Written(
        try blocking { try await client.fetchBlob(name: name, descriptor: descriptor, into: file, progress: progress) }
      )
    }
  }

  func catalog(prefix: RustString?) -> CzOutcome {
    let prefix = prefix?.toString()

    return .awaiting { try await self.client.catalog(prefix: prefix) }
  }

  func referrers(name: RustStr, digest: RustStr, artifactType: RustString?) -> CzOutcome {
    let name = name.toString()
    let digest = digest.toString()
    let artifactType = artifactType?.toString()

    return .awaiting { try await self.client.referrers(name: name, digest: digest, artifactType: artifactType) }
  }
}

extension CzOutcome {
  func registryClient() -> CzRegistryClient { taken() }
}
