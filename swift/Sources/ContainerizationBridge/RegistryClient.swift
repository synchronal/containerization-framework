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
  let client = RegistryClient(
    host: host.toString(),
    scheme: scheme?.toString(),
    port: port.map { Int($0) },
    authentication: authentication.authentication,
    clientID: clientID?.toString(),
    retryOptions: hasRetryOptions ? RetryOptions(maxRetries: maxRetries, retryInterval: retryInterval) : nil,
    bufferSize: Int(bufferSize)
  )

  return CzOutcome { CzRegistryClient(client) }
}

final class CzRegistryClient: Sendable {
  let client: RegistryClient

  init(_ client: RegistryClient) {
    self.client = client
  }

  func ping() -> CzOutcome {
    let client = client

    return CzOutcome { try blocking { try await client.ping() } }
  }

  func resolve(name: RustStr, tag: RustStr) -> CzOutcome {
    let name = name.toString()
    let tag = tag.toString()
    let client = client

    return CzOutcome { try blocking { try await client.resolve(name: name, tag: tag) } }
  }

  func fetchData(name: RustStr, descriptor: RustDescriptor) -> CzOutcome {
    let name = name.toString()
    let descriptor = Descriptor(descriptor)
    let client = client

    return CzOutcome { try blocking { try await client.fetchData(name: name, descriptor: descriptor) } }
  }

  func fetchBlob(name: RustStr, descriptor: RustDescriptor, into: RustStr, progress: RustProgressHandler) -> CzOutcome {
    let name = name.toString()
    let descriptor = Descriptor(descriptor)
    let file = URL(filePath: into.toString())
    let progress = progressHandler(progress)
    let client = client

    return CzOutcome {
      let (size, digest) = try blocking {
        try await client.fetchBlob(name: name, descriptor: descriptor, into: file, progress: progress)
      }
      return Written(size: size, digest: digest.digestString)
    }
  }

  func catalog(prefix: RustString?) -> CzOutcome {
    let prefix = prefix?.toString()
    let client = client

    return CzOutcome { try blocking { try await client.catalog(prefix: prefix) } }
  }

  func referrers(name: RustStr, digest: RustStr, artifactType: RustString?) -> CzOutcome {
    let name = name.toString()
    let digest = digest.toString()
    let artifactType = artifactType?.toString()
    let client = client

    return CzOutcome {
      try blocking { try await client.referrers(name: name, digest: digest, artifactType: artifactType) }
    }
  }
}

extension CzOutcome {
  func registryClient() -> CzRegistryClient { taken() }
}
