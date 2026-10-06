//===----------------------------------------------------------------------===//
// What Containerization's configuration types compute: `Kernel.CommandLine`'s
// edits, `Mount`'s copy and hash, `DNS`, `Hosts`, `ExitStatus`,
// `LinuxRLimit.Kind`, `LinuxProcessConfiguration` and `Signal`.
//
// A process configuration crosses back the way the manager's configuration
// does: Swift fills a Rust value with it and hands that to a Rust closure.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationOCI
import Foundation
import Logging

// MARK: Kernel.CommandLine

private func bridgedCommandLine(_ kernelArgs: RustVec<RustString>, _ initArgs: RustVec<RustString>)
  -> Kernel.CommandLine
{
  Kernel.CommandLine(kernelArgs: strings(kernelArgs), initArgs: strings(initArgs))
}

func kernelCommandLineAddDebug(kernelArgs: RustVec<RustString>, initArgs: RustVec<RustString>) -> CzOutcome {
  var commandLine = bridgedCommandLine(kernelArgs, initArgs)
  commandLine.addDebug()

  return CzOutcome { commandLine }
}

func kernelCommandLineAddPanic(
  kernelArgs: RustVec<RustString>,
  initArgs: RustVec<RustString>,
  level: Int64
) -> CzOutcome {
  var commandLine = bridgedCommandLine(kernelArgs, initArgs)
  commandLine.addPanic(level: Int(level))

  return CzOutcome { commandLine }
}

func kernelCommandLineSetAgentLogLevel(
  kernelArgs: RustVec<RustString>,
  initArgs: RustVec<RustString>,
  level: LogLevel
) -> CzOutcome {
  var commandLine = bridgedCommandLine(kernelArgs, initArgs)
  let level: Logger.Level =
    switch level {
    case .Trace: .trace
    case .Debug: .debug
    case .Info: .info
    case .Notice: .notice
    case .Warning: .warning
    case .Error: .error
    case .Critical: .critical
    }
  commandLine.setAgentLogLevel(level: level)

  return CzOutcome { commandLine }
}

// MARK: Mount

func cloneMount(mount: RustMount, to: RustStr) -> CzOutcome {
  let mount = Containerization.Mount(mount)
  let to = to.toString()

  return CzOutcome { try mount.clone(to: to) }
}

func mountTagHash(mount: RustMount) -> CzOutcome {
  let mount = Containerization.Mount(mount)

  return CzOutcome { try mount.tagHash }
}

// MARK: DNS and Hosts

func validateDNS(dns: RustDns) -> CzOutcome {
  let dns = DNS(dns)

  return CzOutcome { try dns.validate() }
}

func dnsResolvConf(dns: RustDns) -> CzOutcome {
  let dns = DNS(dns)

  return CzOutcome { dns.resolvConf }
}

func hostsFile(hosts: RustHosts) -> CzOutcome {
  let hosts = Hosts(hosts)

  return CzOutcome { hosts.hostsFile }
}

func hostsEntryRendered(entry: RustHostsEntry) -> CzOutcome {
  let entry = Hosts.Entry(entry)

  return CzOutcome { entry.rendered }
}

func namedHostsEntry(name: HostsEntryName, comment: RustString?) -> CzOutcome {
  let comment = comment?.toString()

  return CzOutcome {
    switch name {
    case .LocalHostIpv4: Hosts.Entry.localHostIPV4(comment: comment)
    case .LocalHostIpv6: Hosts.Entry.localHostIPV6(comment: comment)
    case .Ipv6LocalNet: Hosts.Entry.ipv6LocalNet(comment: comment)
    case .Ipv6MulticastPrefix: Hosts.Entry.ipv6MulticastPrefix(comment: comment)
    case .Ipv6AllNodes: Hosts.Entry.ipv6AllNodes(comment: comment)
    case .Ipv6AllRouters: Hosts.Entry.ipv6AllRouters(comment: comment)
    }
  }
}

// MARK: ExitStatus and LinuxRLimit.Kind

func newExitStatus(exitCode: Int32) -> CzOutcome {
  CzOutcome { ExitStatus(exitCode: exitCode) }
}

func parseLinuxRLimitKind(string: RustStr) -> CzOutcome {
  let string = string.toString()

  return CzOutcome { try LinuxRLimit.Kind(string).description }
}

// MARK: LinuxProcessConfiguration

func linuxProcessConfigurationFromImageConfig(
  config: RustImageConfig,
  seed: RustLinuxProcessConfiguration,
  receive: RustConfigureProcess
) -> CzOutcome {
  let config = ImageConfig(config)

  return CzOutcome {
    fill(seed, from: LinuxProcessConfiguration(from: config))
    return receive.call(seed)
  }
}

func linuxProcessConfigurationSetTerminalIO(
  process: RustLinuxProcessConfiguration,
  terminal: CzTerminal,
  receive: RustConfigureProcess
) -> CzOutcome {
  CzOutcome {
    var configuration = try LinuxProcessConfiguration(process)
    configuration.setTerminalIO(terminal: terminal.terminal)
    fill(process, from: configuration)
    return receive.call(process)
  }
}

// MARK: Signal

func parseSignal(name: RustStr) -> CzOutcome {
  let name = name.toString()

  return CzOutcome { try Signal(name).rawValue }
}

func parseSignalFrom(name: RustStr, names: RustVec<RustString>, values: RustVec<Int32>) -> CzOutcome {
  let name = name.toString()
  let map = Dictionary(zip(strings(names), values.map { $0 }), uniquingKeysWith: { _, last in last })

  return CzOutcome { try Signal(name, from: map).rawValue }
}

func linuxSignals() -> CzOutcome {
  CzOutcome { Signal.linux }
}

func platformSignals() -> CzOutcome {
  CzOutcome { Signal.platform }
}

func signalPlatformName(signal: Int32) -> CzOutcome {
  CzOutcome { absent(Signal.platformName(signal)) }
}

func signalLinuxSignal(signal: Int32) -> CzOutcome {
  CzOutcome { absent(Signal(rawValue: signal).linuxSignal()?.rawValue) }
}

// MARK: For unit tests

func linuxRLimitKindDescriptions() -> RustVec<RustString> {
  rustStrings(rlimitKinds.map(\.0.description))
}

func linuxCapabilitiesPresets() -> CzOutcome {
  CzOutcome {
    [
      Containerization.LinuxCapabilities.allCapabilities.toOCI(),
      Containerization.LinuxCapabilities.defaultOCICapabilities.toOCI(),
    ]
  }
}

func linuxSignalValues() -> RustVec<Int32> {
  let signals: [Signal] = [
    Signal.Linux.hup, .Linux.int, .Linux.quit, .Linux.ill, .Linux.trap, .Linux.abrt, .Linux.bus, .Linux.fpe,
    .Linux.kill, .Linux.usr1, .Linux.segv, .Linux.usr2, .Linux.pipe, .Linux.alrm, .Linux.term, .Linux.stkflt,
    .Linux.chld, .Linux.cont, .Linux.stop, .Linux.tstp, .Linux.ttin, .Linux.ttou, .Linux.urg, .Linux.xcpu,
    .Linux.xfsz, .Linux.vtalrm, .Linux.prof, .Linux.winch, .Linux.io, .Linux.poll, .Linux.pwr, .Linux.sys,
    .Linux.rtmax, .Linux.rtmin(),
  ]

  return rawValues(signals)
}

func darwinSignalValues() -> RustVec<Int32> {
  let signals: [Signal] = [
    Signal.Darwin.hup, .Darwin.int, .Darwin.quit, .Darwin.ill, .Darwin.trap, .Darwin.abrt, .Darwin.emt,
    .Darwin.fpe, .Darwin.kill, .Darwin.bus, .Darwin.segv, .Darwin.sys, .Darwin.pipe, .Darwin.alrm,
    .Darwin.term, .Darwin.urg, .Darwin.stop, .Darwin.tstp, .Darwin.cont, .Darwin.chld, .Darwin.ttin,
    .Darwin.ttou, .Darwin.io, .Darwin.xcpu, .Darwin.xfsz, .Darwin.vtalrm, .Darwin.prof, .Darwin.winch,
    .Darwin.info, .Darwin.usr1, .Darwin.usr2,
  ]

  return rawValues(signals)
}

private func rawValues(_ signals: [Signal]) -> RustVec<Int32> {
  let vec = RustVec<Int32>()
  for signal in signals {
    vec.push(value: signal.rawValue)
  }
  return vec
}

func systemPlatformRawValues() -> RustVec<RustString> {
  rustStrings(SystemPlatform.OS.allCases.map(\.rawValue) + SystemPlatform.Architecture.allCases.map(\.rawValue))
}

extension CzOutcome {
  func int32() -> Int32 { taken() }

  func commandLineKernelArgs() -> RustVec<RustString> {
    rustStrings((taken() as Kernel.CommandLine).kernelArgs)
  }

  func commandLineInitArgs() -> RustVec<RustString> {
    rustStrings((taken() as Kernel.CommandLine).initArgs)
  }

  func hostsEntryIpAddress() -> String {
    (taken() as Hosts.Entry).ipAddress
  }

  func hostsEntryHostnames() -> RustVec<RustString> {
    rustStrings((taken() as Hosts.Entry).hostnames)
  }

  func hostsEntryComment() -> String? {
    (taken() as Hosts.Entry).comment
  }
}
