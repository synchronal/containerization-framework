//===----------------------------------------------------------------------===//
// The surface Rust calls.
//
// Flat and synchronous: no Swift types cross, every function blocks, and
// failure returns a negative code with the message in `lastError` for Rust to
// collect. swift-bridge handles `async` poorly in this direction.
//
// Argument labels are the Rust parameter names; swift-bridge's generated
// `@_cdecl` shims call with them, so they are part of the contract.
//
// Nested data crosses as opaque Rust values, read into Containerization's
// types (`Model/`) before any async work.
//===----------------------------------------------------------------------===//

import Foundation
import Synchronization

/// Set by the last failing bridged call, read by `czbridge_last_error`.
/// Locked because two attached terminals can fail at once.
private let lastError = Mutex("")

/// Bridge failure. Outside the guest exit code range 0...255.
private let failed: Int32 = -1

/// Turns a thrown error into `failed`, storing its message in `lastError`.
private func reporting(_ body: () throws -> Int32) -> Int32 {
    do {
        return try body()
    } catch {
        lastError.withLock { $0 = "\(error)" }
        return failed
    }
}

/// A line in the build log, on stderr, unprefixed for the host to label.
func note(_ message: String) {
    try? FileHandle.standardError.write(contentsOf: Data("\(message)\n".utf8))
}

func czbridge_last_error() -> String {
    lastError.withLock { $0 }
}

/// Boots a session's VM and leaves it running, owned by this process.
func czbridge_boot(
    spec: RustBootSpec,
    store_root: RustStr,
    kernel_path: RustStr,
    initfs_reference: RustStr,
    initfs_path: RustStr
) -> Int32 {
    reporting {
        let spec = try BootSpec(
            spec,
            storeRoot: store_root.toString(),
            kernelPath: kernel_path.toString(),
            initfsReference: initfs_reference.toString(),
            initfsPath: initfs_path.toString()
        )

        try blocking { try await Session.boot(spec) }

        return 0
    }
}

/// Builds an image from a plan, with no builder and no daemon.
func czbridge_build(
    plan: RustBuildPlan,
    store_root: RustStr,
    kernel_path: RustStr,
    initfs_reference: RustStr,
    initfs_path: RustStr
) -> Int32 {
    reporting {
        let plan = try BuildPlan(
            plan,
            storeRoot: store_root.toString(),
            kernelPath: kernel_path.toString(),
            initfsReference: initfs_reference.toString(),
            initfsPath: initfs_path.toString()
        )

        try blocking { try await Build.run(plan) }

        return 0
    }
}

/// Puts a kernel and an init image in the store, fetching whatever is missing.
func czbridge_provision(
    store_root: RustStr,
    kernel_path: RustStr,
    kernel_url: RustStr,
    kernel_in_archive: RustStr,
    initfs_reference: RustStr,
    initfs_path: RustStr
) -> Int32 {
    reporting {
        let spec = ProvisionSpec(
            storeRoot: store_root.toString(),
            kernelPath: kernel_path.toString(),
            kernelURL: kernel_url.toString(),
            kernelInArchive: kernel_in_archive.toString(),
            initfsReference: initfs_reference.toString(),
            initfsPath: initfs_path.toString()
        )

        try blocking { try await Provision.run(spec) }

        return 0
    }
}

/// Runs a process in a booted session and blocks until it exits, returning its
/// exit code.
///
/// `terminal` is a descriptor in this process: its own, or one a joining caller
/// passed over the control socket. `-1` runs without a terminal.
func czbridge_exec(
    name: RustStr,
    id: RustStr,
    configuration: RustLinuxProcessConfiguration,
    terminal: Int32,
    stdin: Int32,
    stdout: Int32,
    stderr: Int32
) -> Int32 {
    reporting {
        let request = ExecRequest(
            name: name.toString(),
            id: id.toString(),
            configuration: ProcessSettings(configuration),
            terminal: terminal,
            stdin: stdin,
            stdout: stdout,
            stderr: stderr
        )

        return try blocking { try await Session.exec(request) }
    }
}

/// Tells the guest the terminal changed size. A no-op once the process has gone.
///
/// A failure here is dropped rather than recorded: resizes run beside the
/// attach they belong to, and one that stored its message in `lastError` would
/// be read back as the reason that attach failed.
func czbridge_resize(id: RustStr, terminal: Int32) -> Int32 {
    do {
        let id = id.toString()

        try blocking { try await Session.resize(id: id, terminal: terminal) }

        return 0
    } catch {
        return failed
    }
}

/// 1 when this image is already unpacked, 0 when the next run must unpack it.
func czbridge_is_unpacked(store_root: RustStr, image_reference: RustStr) -> Int32 {
    reporting {
        let unpacked = Unpacked(store: URL(filePath: store_root.toString()))
        let reference = image_reference.toString()

        return try blocking { try await unpacked.holds(reference) } ? 1 : 0
    }
}

/// Whether this process owns a running session by that name.
func czbridge_is_running(name: RustStr) -> Bool {
    Sessions.shared.get(name.toString()) != nil
}
