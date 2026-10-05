//===----------------------------------------------------------------------===//
// `LinuxContainer` and `LinuxProcess`, from Containerization.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationOS
import Foundation

final class CzLinuxContainer: Sendable {
  let container: LinuxContainer

  init(_ container: LinuxContainer) {
    self.container = container
  }

  func id() -> String {
    container.id
  }

  func create() -> CzOutcome {
    let container = container

    return CzOutcome { try blocking { try await container.create() } }
  }

  func start() -> CzOutcome {
    let container = container

    return CzOutcome { try blocking { try await container.start() } }
  }

  func stop() -> CzOutcome {
    let container = container

    return CzOutcome { try blocking { try await container.stop() } }
  }

  func kill(signal: Int32) -> CzOutcome {
    let container = container

    return CzOutcome { try blocking { try await container.kill(Signal(rawValue: signal)) } }
  }

  func wait(timeoutInSeconds: Int64?) -> CzOutcome {
    let container = container

    return CzOutcome { try blocking { try await container.wait(timeoutInSeconds: timeoutInSeconds) } }
  }

  func resize(width: UInt16, height: UInt16) -> CzOutcome {
    let container = container

    return CzOutcome { try blocking { try await container.resize(to: Terminal.Size(width: width, height: height)) } }
  }

  func exec(id: RustStr, configuration: RustLinuxProcessConfiguration) -> CzOutcome {
    let id = id.toString()
    let container = container

    return CzOutcome {
      let configuration = try LinuxProcessConfiguration(configuration)

      return CzLinuxProcess(try blocking { try await container.exec(id, configuration: configuration) })
    }
  }

  func closeStdin() -> CzOutcome {
    let container = container

    return CzOutcome { try blocking { try await container.closeStdin() } }
  }
}

final class CzLinuxProcess: Sendable {
  let process: LinuxProcess

  init(_ process: LinuxProcess) {
    self.process = process
  }

  func id() -> String {
    process.id
  }

  func pid() -> Int32 {
    process.pid
  }

  func start() -> CzOutcome {
    let process = process

    return CzOutcome { try blocking { try await process.start() } }
  }

  func kill(signal: Int32) -> CzOutcome {
    let process = process

    return CzOutcome { try blocking { try await process.kill(Signal(rawValue: signal)) } }
  }

  func resize(width: UInt16, height: UInt16) -> CzOutcome {
    let process = process

    return CzOutcome { try blocking { try await process.resize(to: Terminal.Size(width: width, height: height)) } }
  }

  func closeStdin() -> CzOutcome {
    let process = process

    return CzOutcome { try blocking { try await process.closeStdin() } }
  }

  func wait(timeoutInSeconds: Int64?) -> CzOutcome {
    let process = process

    return CzOutcome { try blocking { try await process.wait(timeoutInSeconds: timeoutInSeconds) } }
  }

  func delete() -> CzOutcome {
    let process = process

    return CzOutcome { try blocking { try await process.delete() } }
  }
}
