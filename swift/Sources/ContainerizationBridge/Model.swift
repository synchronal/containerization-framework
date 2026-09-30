//===----------------------------------------------------------------------===//
// The Rust model, read into Containerization's own types.
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

/// A `Vec<String>`'s elements.
func strings(_ vec: RustVec<RustString>) -> [String] {
    vec.map { $0.as_str().toString() }
}

extension Containerization.Mount {
    init(_ mount: RustMountRef) {
        let options = strings(mount.runtime_options())
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
            options: strings(mount.options()),
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
            nameservers: strings(dns.nameservers()),
            domain: dns.domain()?.toString(),
            searchDomains: strings(dns.search_domains()),
            options: strings(dns.options())
        )
        // The library doesn't check; a hostname would silently break DNS.
        try validate()
    }
}

extension Hosts {
    init(_ hosts: RustHostsRef) {
        self.init(
            entries: hosts.entries().map { entry in
                Hosts.Entry(
                    ipAddress: entry.ip_address().toString(),
                    hostnames: strings(entry.hostnames()),
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
            additionalGids: Array(user.additional_gids()),
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
        arguments = process.has_arguments() ? strings(process.arguments()) : nil
        environmentVariables = strings(process.environment_variables())
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
        sysctl = Dictionary(
            uniqueKeysWithValues: strings(configuration.sysctl_keys()).compactMap { key in
                configuration.sysctl(key).map { (key, $0.toString()) }
            }
        )
        interfaces = try configuration.interfaces().map { try NATInterface($0) }
        sockets = try configuration.sockets().map { try UnixSocketConfiguration($0) }
        mounts = configuration.mounts().map { Containerization.Mount($0) }
        maskedPaths = strings(configuration.masked_paths())
        readonlyPaths = strings(configuration.readonly_paths())
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
