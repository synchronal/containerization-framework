//===----------------------------------------------------------------------===//
// Building an image.
//
// Pull the base, unpack it to a writable ext4 block, boot it, run each step as an
// exec, export the block to a tar, and ingest that as a single-layer image. All
// in-process through Containerization, with no daemon.
//
// The image is one layer regardless of step count. The cache is block
// snapshots, not layers: a rebuild resumes from the deepest matching one. See
// `Cache.swift`.
//
// A build also removes stale builder rootfs and unreferenced blobs.
//
// The export is the fragile step; see `Ingest.swift`.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationExtras
import ContainerizationOCI
import Foundation

enum Build {
    static let cacheDirectory = "build-cache"

    static func run(_ plan: BuildPlan) async throws {
        guard Entitlement.hasVirtualization else {
            throw BridgeError.unentitled
        }

        let root = URL(filePath: plan.storeRoot)
        // Our own because `ImageStore.contentStore` is internal and `ingest`
        // needs it. Same directory the store would use, so readers find the
        // blobs.
        let contentStore = try LocalContentStore(path: root.appending(path: "content"))
        let imageStore = try ImageStore(path: root, contentStore: contentStore)
        let platform = Platform.current

        // Pulled up front: the finished image inherits the base's config, and
        // its digest salts every cache key.
        let base = try await imageStore.get(reference: plan.base, pull: true)
        let baseConfig = try? await base.config(for: platform).config
        let environment = merge(baseConfig?.env ?? [], plan.environment)

        let cache = Cache(
            root: root.appending(path: cacheDirectory),
            keep: plan.cache.keep,
            keepFor: plan.cache.keepForSeconds
        )
        let keys = Keys(plan: plan, baseDigest: base.digest)

        // Nothing changed: re-tag, skipping the export.
        if plan.cache.restore, let descriptor = cache.image(keys.image),
            await Cache.holds(descriptor, in: contentStore)
        {
            note("\(plan.tag) is already built")
            try await retag(plan.tag, to: descriptor, in: imageStore)
            cache.evict()
            return
        }

        let (containerDirectory, rootfsPath) = container(plan.name, in: root)

        try? FileManager.default.removeItem(at: containerDirectory)
        sweepBuilders(in: root, keeping: plan.name)
        try FileManager.default.createDirectory(at: containerDirectory, withIntermediateDirectories: true)
        markBuilder(containerDirectory)

        let start = try await prepare(
            rootfs: rootfsPath,
            plan: plan,
            keys: keys,
            cache: cache,
            base: base,
            platform: platform
        )

        if start < plan.steps.endIndex {
            try await run(
                steps: start..<plan.steps.endIndex,
                of: plan,
                on: rootfsPath,
                keys: keys,
                cache: cache,
                base: base,
                imageStore: imageStore,
                environment: environment
            )
        }

        let descriptor = try await ingest(
            rootfs: rootfsPath,
            plan: plan,
            base: baseConfig,
            environment: environment,
            platform: platform,
            contentStore: contentStore
        )

        try await retag(plan.tag, to: descriptor, in: imageStore)

        cache.save(image: descriptor, as: keys.image)

        // Not `manager.delete`: a fully cached build has no manager, and its only
        // extra work is releasing a network interface, which there isn't.
        try? FileManager.default.removeItem(at: containerDirectory)

        if plan.reclaim {
            await reclaim(imageStore, root: root)
        }

        cache.evict()
    }

    /// Puts a rootfs at `rootfs` and returns the index of the first step to run:
    /// from the deepest cached step, else the cached unpacked base, else a fresh
    /// unpack.
    private static func prepare(
        rootfs: URL,
        plan: BuildPlan,
        keys: Keys,
        cache: Cache,
        base: Containerization.Image,
        platform: Platform
    ) async throws -> Int {
        if plan.cache.restore {
            for index in plan.steps.indices.reversed() where cache.holdsRootfs(keys.steps[index]) {
                note("cached through \(plan.steps[index].name)")
                try cache.restore(keys.steps[index], to: rootfs)

                return index + 1
            }

            if cache.holdsRootfs(keys.base) {
                try cache.restore(keys.base, to: rootfs)

                return 0
            }
        }

        let unpacker = EXT4Unpacker(capacityInBytes: plan.rootfsSizeInBytes)

        _ = try await unpacker.unpack(base, for: platform, at: rootfs)
        cache.save(rootfs: rootfs, as: keys.base)

        return 0
    }

    /// Runs the remaining steps, snapshotting the rootfs after each.
    ///
    /// One container per step: the block is only consistent once `stop` has
    /// unmounted it in the guest, so each snapshot costs a boot.
    private static func run(
        steps: Range<Int>,
        of plan: BuildPlan,
        on rootfs: URL,
        keys: Keys,
        cache: Cache,
        base: Containerization.Image,
        imageStore: ImageStore,
        environment: [String]
    ) async throws {
        let kernel = Kernel(path: URL(filePath: plan.kernelPath), platform: .linuxArm)
        var manager = try ContainerManager(
            kernel: kernel,
            initfs: try await Initfs.mount(plan.initfsReference, at: URL(filePath: plan.initfsPath), in: imageStore),
            imageStore: imageStore,
            network: nil
        )

        let mounts = plan.mounts
        let interface = plan.interface
        let block = Containerization.Mount.ext4Root(rootfs)

        for index in steps {
            let container = try await manager.create(
                plan.name,
                image: base,
                rootfs: block,
                networking: false,
                vm: plan.vm
            ) { config in
                config.cpus = plan.cpus
                config.memoryInBytes = plan.memoryInBytes
                // A keepalive, as in a session: steps are execs and need the
                // container to outlive them. The base's `Cmd` would exit.
                config.process.arguments = plan.keepalive
                config.process.user = .init()
                config.process.workingDirectory = "/"
                config.process.environmentVariables = environment
                config.mounts += mounts
                // Steps that install anything need network and DNS.
                config.interfaces = [interface]
                config.dns = interface.ipv4Gateway.map { DNS(nameservers: [$0.description]) }
            }

            try await container.create()
            try await container.start()

            do {
                try await step(
                    plan.steps[index],
                    index: index,
                    in: container,
                    shell: plan.shell,
                    environment: environment
                )
            } catch {
                // Left for inspection, not cached; the next build sweeps it.
                try? await container.stop()
                throw error
            }

            try await container.stop()

            cache.save(rootfs: rootfs, as: keys.steps[index])
        }
    }

    /// Marks a `containers` directory as a builder's, not a session's. Holds the
    /// owning build's pid.
    private static let builderMarker = ".builder"

    private static func markBuilder(_ directory: URL) {
        try? Data("\(getpid())\n".utf8).write(to: directory.appending(path: builderMarker))
    }

    /// Removes rootfs left by failed builds and by tags since renamed.
    ///
    /// Only marked directories (sessions share `containers`), and only when the
    /// owning pid is gone (builds in other projects share this store).
    private static func sweepBuilders(in root: URL, keeping current: String) {
        let directories =
            (try? FileManager.default.contentsOfDirectory(at: containers(in: root), includingPropertiesForKeys: nil))
            ?? []

        for directory in directories where directory.lastPathComponent != current {
            let marker = directory.appending(path: builderMarker)

            guard let owner = try? String(contentsOf: marker, encoding: .utf8) else {
                continue
            }

            // Signal 0: does the process exist.
            if let pid = pid_t(owner.trimmingCharacters(in: .whitespacesAndNewlines)), kill(pid, 0) == 0 {
                continue
            }

            note("removing the rootfs left by \(directory.lastPathComponent)")
            try? FileManager.default.removeItem(at: directory)
        }
    }

    /// Points a reference at what this build produced.
    ///
    /// Delete first: `create` will not replace an existing reference, which a
    /// rebuild always has.
    private static func retag(_ reference: String, to descriptor: Descriptor, in imageStore: ImageStore) async throws {
        try? await imageStore.delete(reference: reference)
        try await imageStore.create(description: .init(reference: reference, descriptor: descriptor))
    }

    /// Deletes unreferenced blobs (chiefly the previous build's multi-gigabyte
    /// layer, orphaned by the re-tag) and unpacked rootfs of removed images.
    ///
    /// Only after `create`: until then this build's own blobs are unreferenced.
    /// Failure is logged, not thrown; the image is already usable.
    private static func reclaim(_ imageStore: ImageStore, root: URL) async {
        if let images = try? await imageStore.list() {
            Unpacked(store: root).evict(keeping: images.map(\.digest))
        }

        do {
            let (deleted, freed) = try await imageStore.cleanUpOrphanedBlobs()

            guard !deleted.isEmpty else {
                return
            }

            let size = ByteCountFormatter.string(fromByteCount: Int64(freed), countStyle: .file)

            note("reclaimed \(size) from \(deleted.count) unreferenced blob\(deleted.count == 1 ? "" : "s")")
        } catch {
            note("could not reclaim unreferenced blobs: \(error)")
        }
    }

    /// Runs one step to completion, throwing when it fails.
    ///
    /// `shell` is the caller's, with the script appended: whether a step that
    /// fails halfway through fails the build is its policy, not this one's.
    private static func step(
        _ step: BuildStep,
        index: Int,
        in container: LinuxContainer,
        shell: [String],
        environment: [String]
    ) async throws {
        let log = FileWriter(FileHandle.standardError)

        log.line("--> \(step.name)")

        let process = try await container.exec("build-\(index)") { config in
            config.arguments = shell + [step.script]
            config.environmentVariables = environment
            config.workingDirectory = "/"
            config.user = step.user.map { User(username: $0) } ?? User()
            config.stdout = log
            config.stderr = log
        }

        try await process.start()
        let status = try await process.wait()
        try? await process.delete()

        guard status.exitCode == 0 else {
            throw BridgeError.stepFailed(step.name, status.exitCode)
        }
    }

    /// `ENV` semantics: the plan's variables override the base's, and everything
    /// else the base declares is kept, in the base's order.
    private static func merge(_ base: [String], _ additions: [String]) -> [String] {
        func name(_ variable: String) -> String {
            String(variable.prefix(while: { $0 != "=" }))
        }

        let overridden = Set(additions.map(name))
        var merged = base.filter { !overridden.contains(name($0)) }

        merged.append(contentsOf: additions)

        return merged
    }
}
