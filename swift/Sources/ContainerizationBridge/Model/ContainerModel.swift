//===----------------------------------------------------------------------===//
// The Rust model, read into Containerization's own types: a container to boot,
// and a process to run in one. `BuildModel.swift` is an image to build.
//
// Read once, on the calling thread, from the opaque Rust values (`RustMount`,
// `RustBootSpec`, ...) into `Sendable` types the async work can carry. Fields
// Rust leaves unset keep the image's or library's value.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationExtras
import ContainerizationOCI
import Foundation
// For `FilePermissions` (relayed socket mode).
import SystemPackage

extension Containerization.Mount {
    init(_ mount: RustMountRef) {
        let options = strings(mount.runtime_options_len(), mount.runtime_options_at)
        let runtimeOptions: RuntimeOptions =
            switch mount.runtime_kind() {
            case .Virtioblk: .virtioblk(options)
            case .Virtiofs: .virtiofs(options)
            case .Generic: .any(options)
            }

        self.init(
            type: mount.mount_type().toString(),
            source: mount.source().toString(),
            destination: mount.destination().toString(),
            options: strings(mount.options_len(), mount.options_at),
            runtimeOptions: runtimeOptions
        )
    }
}

extension UnixSocketConfiguration {
    init(_ socket: RustUnixSocketConfigurationRef) throws {
        // `CModeT` is 16-bit: refuse a mode that doesn't fit, not truncate it.
        let permissions = try socket.permissions().map { mode in
            guard let mode = CModeT(exactly: mode) else {
                throw BridgeError.malformed("socket permissions", String(mode, radix: 8))
            }
            return FilePermissions(rawValue: mode)
        }

        let direction: Direction =
            switch socket.direction() {
            case .Into: .into
            case .OutOf: .outOf
            }

        self.init(
            source: URL(filePath: socket.source().toString()),
            destination: URL(filePath: socket.destination().toString()),
            permissions: permissions,
            direction: direction
        )
    }
}

extension NATInterface {
    init(_ interface: RustNatInterfaceRef) throws {
        self.init(
            ipv4Address: try CIDRv4(interface.ipv4_address().toString()),
            ipv4Gateway: try interface.ipv4_gateway().map { try IPv4Address($0.toString()) },
            ipv6Address: try interface.ipv6_address().map { try CIDRv6($0.toString()) },
            ipv6Gateway: try interface.ipv6_gateway().map { try IPv6Address($0.toString()) },
            macAddress: try interface.mac_address().map { try MACAddress($0.toString()) },
            mtu: interface.mtu()
        )
    }
}

extension DNS {
    init(_ dns: RustDnsRef) throws {
        self.init(
            nameservers: strings(dns.nameservers_len(), dns.nameservers_at),
            domain: dns.domain()?.toString(),
            searchDomains: strings(dns.search_domains_len(), dns.search_domains_at),
            options: strings(dns.options_len(), dns.options_at)
        )
        // The library doesn't check; a hostname would silently break DNS.
        try validate()
    }
}

extension Hosts {
    init(_ hosts: RustHostsRef) {
        self.init(
            entries: list(hosts.entries_len(), hosts.entries_at).map { entry in
                Hosts.Entry(
                    ipAddress: entry.ip_address().toString(),
                    hostnames: strings(entry.hostnames_len(), entry.hostnames_at),
                    comment: entry.comment()?.toString()
                )
            },
            comment: hosts.comment()?.toString()
        )
    }
}

extension User {
    init(_ user: RustUserRef) {
        self.init(
            uid: user.uid(),
            gid: user.gid(),
            umask: user.umask(),
            additionalGids: list(user.additional_gids_len(), user.additional_gids_at),
            username: user.username().toString()
        )
    }
}

/// `LinuxProcessConfiguration`, over one already seeded from the image.
struct ProcessSettings: Sendable {
    var arguments: [String]?
    var environmentVariables: [String]
    var workingDirectory: String?
    var user: User?

    init(_ process: RustLinuxProcessConfigurationRef) {
        arguments = process.has_arguments() ? strings(process.arguments_len(), process.arguments_at) : nil
        environmentVariables = strings(process.environment_variables_len(), process.environment_variables_at)
        workingDirectory = process.working_directory()?.toString()
        user = process.has_user() ? User(process.user()) : nil
    }

    func apply(to process: inout LinuxProcessConfiguration) {
        if let arguments {
            process.arguments = arguments
        }

        // Last, so a variable the caller sets beats the image's.
        process.environmentVariables += environmentVariables

        if let workingDirectory {
            process.workingDirectory = workingDirectory
        }

        if let user {
            process.user = user
        }
    }
}

/// `LinuxContainer.Configuration`, over one `ContainerManager` seeded.
struct ContainerSettings: Sendable {
    var process: ProcessSettings
    var cpus: Int
    var memoryInBytes: UInt64
    var hostname: String?
    var sysctl: [String: String]
    var interfaces: [NATInterface]
    var sockets: [UnixSocketConfiguration]
    var mounts: [Containerization.Mount]
    var maskedPaths: [String]
    var readonlyPaths: [String]
    var dns: DNS?
    var hosts: Hosts?
    var virtualization: Bool
    var bootLog: BootLog?
    var ociRuntimePath: String?
    var seccompProfile: LinuxContainer.Configuration.SeccompProfile
    var useInit: Bool

    init(_ configuration: RustLinuxContainerConfigurationRef) throws {
        process = ProcessSettings(configuration.process())
        cpus = Int(configuration.cpus())
        memoryInBytes = configuration.memory_in_bytes()
        hostname = configuration.hostname()?.toString()
        sysctl = dictionary(configuration.sysctl_len(), configuration.sysctl_key_at, configuration.sysctl_value_at)
        interfaces = try list(configuration.interfaces_len()) { try NATInterface(configuration.interfaces_at($0)) }
        sockets = try list(configuration.sockets_len()) {
            try UnixSocketConfiguration(configuration.sockets_at($0))
        }
        mounts = list(configuration.mounts_len()) { Containerization.Mount(configuration.mounts_at($0)) }
        maskedPaths = strings(configuration.masked_paths_len(), configuration.masked_paths_at)
        readonlyPaths = strings(configuration.readonly_paths_len(), configuration.readonly_paths_at)
        dns = configuration.has_dns() ? try DNS(configuration.dns()) : nil
        hosts = configuration.has_hosts() ? Hosts(configuration.hosts()) : nil
        virtualization = configuration.virtualization()

        if configuration.has_boot_log() {
            let bootLog = configuration.boot_log()
            self.bootLog = .file(path: URL(filePath: bootLog.path().toString()), append: bootLog.append())
        } else {
            bootLog = nil
        }

        ociRuntimePath = configuration.oci_runtime_path()?.toString()
        seccompProfile =
            switch configuration.seccomp_mode() {
            case .Unconfined: .unconfined
            case .Default: .default
            case .Profile:
                .profile(try LinuxSeccomp.decode(from: Data((configuration.seccomp_profile()?.toString() ?? "").utf8)))
            }
        useInit = configuration.use_init()
    }

    func apply(to config: inout LinuxContainer.Configuration) {
        process.apply(to: &config.process)
        config.cpus = cpus
        config.memoryInBytes = memoryInBytes
        config.hostname = hostname
        config.sysctl = sysctl
        config.interfaces = interfaces
        config.sockets = sockets
        config.mounts = mounts
        config.maskedPaths = maskedPaths
        config.readonlyPaths = readonlyPaths
        config.dns = dns
        config.hosts = hosts
        config.virtualization = virtualization

        // Absent keeps the manager's, in the container's directory.
        if let bootLog {
            config.bootLog = bootLog
        }

        config.ociRuntimePath = ociRuntimePath
        config.seccompProfile = seccompProfile
        config.useInit = useInit
    }
}

/// `BootSpec`, and the store it boots from.
struct BootSpec: Sendable {
    var storeRoot: String
    var kernelPath: String
    var initfsReference: String
    var initfsPath: String
    var id: String
    var reference: String
    /// Ceiling for the image's unpacked rootfs, which is sparse.
    var rootfsSizeInBytes: UInt64
    var vm: VMResources
    var configuration: ContainerSettings

    init(
        _ spec: RustBootSpecRef,
        storeRoot: String,
        kernelPath: String,
        initfsReference: String,
        initfsPath: String
    ) throws {
        self.storeRoot = storeRoot
        self.kernelPath = kernelPath
        self.initfsReference = initfsReference
        self.initfsPath = initfsPath
        id = spec.id().toString()
        reference = spec.reference().toString()
        rootfsSizeInBytes = spec.rootfs_size_in_bytes()
        vm = VMResources(cpus: Int(spec.vm_cpus()), memoryInBytes: spec.vm_memory_in_bytes())
        configuration = try ContainerSettings(spec.configuration())
    }
}
