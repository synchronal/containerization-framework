//===----------------------------------------------------------------------===//
// The Rust model, read into what a build carries: an image to build, its
// steps, and how it caches them. `ContainerModel.swift` is the rest.
//===----------------------------------------------------------------------===//

import Containerization

/// A step's shell script, and who runs it.
struct BuildStep: Sendable {
    /// The step's label in the build log.
    var name: String
    /// The guest user, as the image names it. Root when absent.
    var user: String?
    var script: String
    /// Unsalted; see `Keys`.
    var cacheKey: String

    init(_ step: RustBuildStepRef) {
        name = step.name().toString()
        user = step.user()?.toString()
        script = step.script().toString()
        cacheKey = step.cache_key().toString()
    }
}

/// How this build treats its rootfs snapshots. Snapshots are written whether
/// or not they are read.
struct CachePolicy: Sendable {
    /// Resume from the deepest matching snapshot.
    var restore: Bool
    /// Snapshots kept, most recently used first.
    var keep: Int
    /// Anything unused this long goes regardless.
    var keepForSeconds: Double
}

/// `BuildPlan`, and the store it builds into.
struct BuildPlan: Sendable {
    /// The builder container's id; its store directory is removed before and
    /// after the build.
    var name: String
    var storeRoot: String
    var kernelPath: String
    var initfsReference: String
    var initfsPath: String
    /// The image every step runs on top of, registry-qualified.
    var base: String
    /// What the finished image is registered as.
    var tag: String
    /// The builder's cgroup limits, and its VM.
    var cpus: Int
    var memoryInBytes: UInt64
    var vm: VMResources
    var rootfsSizeInBytes: UInt64
    /// Shared into every step: what scripts read instead of `COPY`.
    var mounts: [Containerization.Mount]
    var steps: [BuildStep]
    /// `NAME=VALUE`, written into the image config and visible to every step,
    /// like `ENV`.
    var environment: [String]
    /// The finished image's OCI labels, as the caller named them.
    var labels: [String: String]
    /// The user and directory the finished image runs as.
    var user: String?
    var workingDirectory: String?
    /// The builder's network; steps resolve through its gateway.
    var interface: NATInterface
    var baseKey: String
    var cache: CachePolicy
    /// What runs each step's script, with the script appended. Defaults to
    /// `bash -euo pipefail -c` on the Rust side: the generated scripts chain
    /// with `&&`, and a silent mid-step failure would be baked into the image.
    var shell: [String]
    /// The builder's first process. Steps are `exec`s and need the container
    /// to outlive them; the base's `Cmd` would exit.
    var keepalive: [String]
    /// Delete unreferenced blobs and unpacked rootfs once the image is stored.
    var reclaim: Bool

    init(
        _ plan: RustBuildPlanRef,
        storeRoot: String,
        kernelPath: String,
        initfsReference: String,
        initfsPath: String
    ) throws {
        name = plan.name().toString()
        self.storeRoot = storeRoot
        self.kernelPath = kernelPath
        self.initfsReference = initfsReference
        self.initfsPath = initfsPath
        base = plan.base().toString()
        tag = plan.tag().toString()
        cpus = Int(plan.cpus())
        memoryInBytes = plan.memory_in_bytes()
        vm = VMResources(cpus: Int(plan.vm_cpus()), memoryInBytes: plan.vm_memory_in_bytes())
        rootfsSizeInBytes = plan.rootfs_size_in_bytes()
        mounts = list(plan.mounts_len()) { Containerization.Mount(plan.mounts_at($0)) }
        steps = list(plan.steps_len()) { BuildStep(plan.steps_at($0)) }
        environment = strings(plan.environment_len(), plan.environment_at)
        labels = dictionary(plan.labels_len(), plan.label_key_at, plan.label_value_at)
        user = plan.user()?.toString()
        workingDirectory = plan.working_directory()?.toString()
        interface = try NATInterface(plan.interface())
        baseKey = plan.base_key().toString()
        cache = CachePolicy(
            restore: plan.cache_restore(),
            keep: Int(plan.cache_keep()),
            keepForSeconds: Double(plan.cache_keep_for_seconds())
        )
        shell = strings(plan.shell_len(), plan.shell_at)
        keepalive = strings(plan.keepalive_len(), plan.keepalive_at)
        reclaim = plan.reclaim()
    }
}
