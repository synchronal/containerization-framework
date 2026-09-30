//===----------------------------------------------------------------------===//
// What the bridge itself fails with. Containerization's own errors pass through
// unchanged; `Bridge.swift` hands either to Rust as a message.
//===----------------------------------------------------------------------===//

enum BridgeError: Error, CustomStringConvertible {
    /// Cannot happen: the semaphore is only signalled after `outcome` is set.
    case noOutcome
    /// A value the bridge could not parse: a malformed wire string, or a
    /// setting that never made sense.
    case malformed(String, String)
    /// An `exec` for a session this process does not hold, e.g. a joining
    /// caller reaching the wrong process. The control socket prevents it.
    case notBooted(String)
    /// The binary is not signed for virtualization, usually a rebuild that was
    /// not re-signed.
    case unentitled
    /// A build step exited non-zero. Carries the step's label so the message
    /// names what failed.
    case stepFailed(String, Int32)
    /// The ingest body left no index descriptor: a bug in `Build.ingest`.
    case notIngested(String)
    /// The kernel download returned a non-success status.
    case kernelUnavailable(String, Int)
    /// The downloaded archive lacks the kernel at the expected path: the
    /// release layout has changed.
    case kernelMissing(String)
    /// A rootfs could not be copied to or from the build cache. Filesystems
    /// without clones fall back to a copy and never land here.
    case notCloned(String, String)

    var description: String {
        switch self {
        case .noOutcome:
            return "the bridged task signalled completion without an outcome"
        case .malformed(let what, let value):
            return "malformed \(what): \(value.debugDescription)"
        case .notBooted(let name):
            return "this process does not own a session named \(name)"
        case .stepFailed(let label, let code):
            return "build step failed with exit code \(code): \(label)"
        case .notIngested(let reference):
            return "nothing was ingested for \(reference)"
        case .kernelUnavailable(let url, let status):
            return "downloading a kernel from \(url) answered \(status)"
        case .kernelMissing(let path):
            return "the downloaded archive has no \(path)"
        case .notCloned(let name, let reason):
            return "could not copy \(name): \(reason)"
        case .unentitled:
            return "this build is not signed for virtualization!"
        }
    }
}
