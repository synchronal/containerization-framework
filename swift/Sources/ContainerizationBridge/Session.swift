//===----------------------------------------------------------------------===//
// Booting a session's VM, and running processes in it.
//
// `ContainerManager.create`, `create`, `start`, as `cctl`'s RunCommand, except:
// the rootfs is unpacked once and cloned per container (`Unpacked`); interfaces
// are the caller's, on Virtualization's NAT; and a process attaches to whichever
// terminal asked (the owner's, or a joining caller's).
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationExtras
import ContainerizationOCI
import ContainerizationOS
import Foundation

/// `BootSpec`, and the store it boots from.
struct BootSpec: Sendable {
    var storeRoot: String
    var kernelPath: String
    var initfsReference: String
    var id: String
    var reference: String
    /// Ceiling for the image's unpacked rootfs, which is sparse.
    var rootfsSizeInBytes: UInt64
    var vm: VMResources
    var configuration: ContainerSettings

    init(_ spec: RustBootSpecRef, storeRoot: String, kernelPath: String, initfsReference: String) throws {
        self.storeRoot = storeRoot
        self.kernelPath = kernelPath
        self.initfsReference = initfsReference
        id = spec.id().toString()
        reference = spec.reference().toString()
        rootfsSizeInBytes = spec.rootfs_size_in_bytes()
        vm = VMResources(cpus: Int(spec.vm_cpus()), memoryInBytes: spec.vm_memory_in_bytes())
        configuration = try ContainerSettings(spec.configuration())
    }
}

/// A process to run in a booted session, and the descriptors it runs against.
struct ExecRequest {
    var name: String
    var id: String
    var configuration: ProcessSettings
    /// The terminal the process reads and is sized against, or `-1` for a
    /// caller that has none and attaches `stdin` instead.
    var terminal: Int32
    /// The caller's own streams. Each is `-1` when it leaves that stream
    /// unattached, and the guest reads or writes nothing on it. A process on a
    /// terminal still writes to `stdout`, wherever that points, and has no
    /// separate `stderr`.
    var stdin: Int32
    var stdout: Int32
    var stderr: Int32
}

/// The store's `containers` directory, shared by sessions and builders.
func containers(in root: URL) -> URL {
    root.appending(path: "containers")
}

/// A container's directory (its rootfs and the library's boot log) and the
/// rootfs within it.
func container(_ name: String, in root: URL) -> (directory: URL, rootfs: URL) {
    let directory = containers(in: root).appending(path: name)

    return (directory, directory.appending(path: "rootfs.ext4"))
}

enum Session {
    static func boot(_ spec: BootSpec) async throws {
        // First, so an unsigned build says so instead of failing in whichever
        // Virtualization call comes first, with an error that never names the
        // cause.
        guard Entitlement.hasVirtualization else {
            throw BridgeError.unentitled
        }

        let root = URL(filePath: spec.storeRoot)
        let kernel = Kernel(path: URL(filePath: spec.kernelPath), platform: .linuxArm)

        // No `Network`; the interfaces are the caller's. `VmnetNetwork` (what
        // `cctl` uses) fails with VMNET_MEM_FAILURE from an unprivileged
        // process, which is why the `container` CLI runs vmnet as a separate
        // helper. Virtualization's own NAT needs no extra privilege.
        var manager = try await ContainerManager(
            kernel: kernel,
            initfsReference: spec.initfsReference,
            root: root,
            network: nil
        )

        // Pulled if missing, like `create(reference:)`, which we avoid because
        // it unpacks every run. So we create the container directory (where the
        // boot log goes) ourselves.
        let image = try await manager.imageStore.get(reference: spec.reference, pull: true)
        let paths = container(spec.id, in: root)
        try FileManager.default.createDirectory(at: paths.directory, withIntermediateDirectories: true)
        let rootfs = try await Unpacked(store: root, capacityInBytes: spec.rootfsSizeInBytes)
            .rootfs(for: image, at: paths.rootfs)

        // `networking: false`: the manager has no `Network` to allocate from.
        let container = try await manager.create(
            spec.id,
            image: image,
            rootfs: rootfs,
            networking: false,
            vm: spec.vm
        ) { config in
            spec.configuration.apply(to: &config)
        }

        try await container.create()
        try await container.start()

        // Kept for seeding `exec`.
        let imageConfig = try? await image.config(for: .current).config

        Sessions.shared.insert(
            spec.id,
            Booted(manager: manager, container: container, imageConfig: imageConfig)
        )
    }

    /// Runs a process to completion in an already-booted session and returns its
    /// exit code.
    ///
    /// Takes a descriptor rather than `Terminal.current` because a joining
    /// caller passes its tty over the control socket; from here the two cases
    /// are identical.
    static func exec(_ request: ExecRequest) async throws -> Int32 {
        guard let booted = Sessions.shared.get(request.name) else {
            throw BridgeError.notBooted(request.name)
        }

        let imageConfig = booted.imageConfig

        // `setInitState: false`: the caller set raw mode and restores it. The
        // descriptor is ours, a duplicate for this attach; closing it stops our
        // reads.
        let terminal = request.terminal < 0 ? nil : try Terminal(descriptor: request.terminal, setInitState: false)

        // Every descriptor is a duplicate made for this attach, and closes with
        // it; the caller keeps the streams they came from.
        let reader = handle(request.stdin).map(FileReader.init)
        let out = handle(request.stdout).map { FileWriter($0, owned: true) }
        let error = handle(request.stderr).map { FileWriter($0, owned: true) }

        // `defer`, so an error before the wait doesn't leave a reader on a
        // terminal someone is still typing at.
        defer {
            try? terminal?.close()
            reader?.close()
        }

        let process = try await booted.container.exec(request.id) { config in
            // Seeded from the image like the first process: a bare exec config
            // runs as root with only a default PATH, ignoring the image's `USER`.
            if let imageConfig {
                let fallback = config.environmentVariables
                config = .init(from: imageConfig)

                // Seeding replaces the environment; keep the default PATH if the
                // image declares none.
                if !config.environmentVariables.contains(where: { $0.hasPrefix("PATH=") }) {
                    config.environmentVariables += fallback
                }
            }

            request.configuration.apply(to: &config)

            // Not `setTerminalIO`, which writes back to the terminal it reads.
            // The caller's stdout may be somewhere else entirely, and this is
            // what keeps a redirection its shell made. Its stderr has nowhere
            // else to go: one pty carries every stream, and Containerization
            // refuses a separate stderr beside `terminal`.
            if let terminal {
                config.terminal = true

                // What `setTerminalIO` sets, unless the caller chose.
                if !config.environmentVariables.contains(where: { $0.hasPrefix("TERM=") }) {
                    config.environmentVariables.append("TERM=xterm")
                }

                config.stdin = terminal
                config.stdout = out
            } else {
                config.stdin = reader
                config.stdout = out
                config.stderr = error
            }
        }

        Sessions.shared.insert(process: process, id: request.id)
        defer { Sessions.shared.remove(process: request.id) }

        try await process.start()

        if let terminal {
            try? await process.resize(to: try terminal.size)
        }

        let status = try await process.wait()
        try? await process.delete()

        return status.exitCode
    }

    /// Re-reads the size from the attached terminal and tells the guest.
    ///
    /// Takes the descriptor rather than a size, so a stale size can't race a
    /// second resize.
    static func resize(id: String, terminal: Int32) async throws {
        guard let process = Sessions.shared.process(id) else {
            // The process ended after the SIGWINCH; not an error.
            return
        }

        let size = try Terminal(descriptor: terminal, setInitState: false).size
        try await process.resize(to: size)
    }
}
