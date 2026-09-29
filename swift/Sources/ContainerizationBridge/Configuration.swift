//===----------------------------------------------------------------------===//
// What a container is configured with beyond its process, mounts and network.
//
// Crosses as JSON (`wire::Configuration`), since it nests. Everything left
// unset keeps whatever the framework, the manager or `NAT` chose.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationOCI
import Foundation

struct ContainerConfiguration: Decodable {
    /// The container's name when absent.
    var hostname: String?
    var sysctl: [String: String]
    /// Resolves through the gateway (`NAT.join`) when absent.
    var dns: DNSSettings?
    /// Keeps the image's `/etc/hosts` when absent.
    var hosts: [HostsEntry]?
    var maskedPaths: GuardedPaths
    var readonlyPaths: GuardedPaths
    var useInit: Bool
    var nestedVirtualization: Bool
    /// The manager's, in the container's directory, when absent.
    var bootLog: String?
    var ociRuntimePath: String?
    var seccomp: SeccompSettings

    struct DNSSettings: Decodable {
        var nameservers: [String]
        var domain: String?
        var searchDomains: [String]
        var options: [String]
    }

    struct HostsEntry: Decodable {
        var ipAddress: String
        var hostnames: [String]
        var comment: String?
    }

    /// `{"mode": "default" | "defaultAnd" | "only", "paths": [...]}`.
    struct GuardedPaths: Decodable {
        var mode: String
        var paths: [String]?

        /// This set, against the framework's standard one.
        func resolve(standard: [String], kind: String) throws -> [String] {
            switch mode {
            case "default": standard
            case "defaultAnd": standard + (paths ?? [])
            case "only": paths ?? []
            default: throw BridgeError.malformed(kind, mode)
            }
        }
    }

    /// `{"mode": "unconfined" | "default" | "profile", "profile": "<json>"}`.
    struct SeccompSettings: Decodable {
        var mode: String
        var profile: String?

        var resolved: LinuxContainer.Configuration.SeccompProfile {
            get throws {
                switch mode {
                case "unconfined":
                    return .unconfined
                case "default":
                    return .default
                case "profile":
                    guard let profile else {
                        throw BridgeError.malformed("seccomp profile", "none given")
                    }
                    return .profile(try LinuxSeccomp.decode(from: Data(profile.utf8)))
                default:
                    throw BridgeError.malformed("seccomp mode", mode)
                }
            }
        }
    }

    /// Applies what the caller set over `config`, last, so it wins.
    func apply(to config: inout LinuxContainer.Configuration) throws {
        config.hostname = hostname
        config.sysctl = sysctl

        if let dns {
            let resolved = DNS(
                nameservers: dns.nameservers,
                domain: dns.domain,
                searchDomains: dns.searchDomains,
                options: dns.options
            )
            // The framework writes whatever it is given; a hostname here would
            // leave a guest resolving nothing, and say so nowhere.
            try resolved.validate()
            config.dns = resolved
        }

        if let hosts {
            config.hosts = Hosts(
                entries: hosts.map {
                    Hosts.Entry(ipAddress: $0.ipAddress, hostnames: $0.hostnames, comment: $0.comment)
                }
            )
        }

        config.maskedPaths = try maskedPaths.resolve(
            standard: LinuxContainer.defaultMaskedPaths(),
            kind: "masked paths"
        )
        config.readonlyPaths = try readonlyPaths.resolve(
            standard: LinuxContainer.defaultReadonlyPaths(),
            kind: "read-only paths"
        )
        config.useInit = useInit
        config.virtualization = nestedVirtualization

        if let bootLog {
            config.bootLog = .file(path: URL(filePath: bootLog), append: false)
        }

        config.ociRuntimePath = ociRuntimePath
        config.seccompProfile = try seccomp.resolved
    }
}
