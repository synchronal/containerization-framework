//! `ProcNetSnmp`, its nested `ProcNetSnmp.Counters`, and the
//! [`ProcNetSnmpOptions`] its initializer takes.

use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// The defaulted arguments of `ProcNetSnmp(snmp:netstat:snmp6:)`.
/// [`Default`] uses Swift's defaults, which are both `nil`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProcNetSnmpOptions<'a> {
  /// The contents of `/proc/net/netstat`.
  pub netstat: Option<&'a str>,
  /// The contents of `/proc/net/snmp6`.
  pub snmp6: Option<&'a str>,
}

/// `ProcNetSnmp.Counters`, the traffic counters for one protocol. [`Default`]
/// uses Swift's `init()`: zero packets, and every other counter `None`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Counters {
  pub received_packets: u64,
  pub transmitted_packets: u64,
  /// `None` when the kernel keeps no byte counter for the protocol.
  pub received_bytes: Option<u64>,
  pub transmitted_bytes: Option<u64>,
  /// Packets discarded for lack of resources or a destination. The kernel's
  /// causes differ for each protocol. `None` when the protocol has no
  /// matching counter, and zero when none happened.
  pub received_drops: Option<u64>,
  pub transmitted_drops: Option<u64>,
  /// Malformed or failed packets. As with drops, the causes differ for each
  /// protocol.
  pub received_errors: Option<u64>,
  pub transmitted_errors: Option<u64>,
}

/// `ProcNetSnmp`, the protocol traffic counters of a network namespace, as
/// the kernel reports them in `/proc/net/snmp`, `/proc/net/netstat` and
/// `/proc/net/snmp6`. They cover the whole namespace, including loopback,
/// and not one interface. [`Default`] uses Swift's `init()`: every counter at
/// its default, and no IPv6 counters.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProcNetSnmp {
  /// TCP segments, both IPv4 and IPv6, because the kernel shares its TCP
  /// counters between the two families.
  pub tcp: Counters,
  /// IPv4 UDP datagrams.
  pub udp: Counters,
  /// IPv4 ICMP messages.
  pub icmp: Counters,
  /// All IPv4 packets and bytes.
  pub ip: Counters,
  /// IPv6 UDP datagrams. `None` without `/proc/net/snmp6`.
  pub udp6: Option<Counters>,
  /// IPv6 ICMP messages. `None` without `/proc/net/snmp6`.
  pub icmp6: Option<Counters>,
  /// All IPv6 packets and bytes. `None` without `/proc/net/snmp6`.
  pub ip6: Option<Counters>,
}

impl ProcNetSnmp {
  /// `ProcNetSnmp(snmp:netstat:snmp6:)`, which parses the contents of
  /// `/proc/net/snmp` and, if given, `/proc/net/netstat` and
  /// `/proc/net/snmp6`. Swift matches fields by name, so columns that newer
  /// kernels add don't break parsing.
  pub fn new(snmp: &str, options: ProcNetSnmpOptions<'_>) -> Result<Self, Error> {
    platform::outcome(
      ffi::cz_proc_net_snmp_new(
        snmp,
        options.netstat.map(str::to_string),
        options.snmp6.map(str::to_string),
      ),
      "parse network protocol counters",
    )
    .map(|outcome| outcome.proc_net_snmp())
  }

  /// `ProcNetSnmp.read(pid:)`, which reads the counters of the network
  /// namespace of process `pid`, or of the calling thread for `None`, which
  /// is Swift's default. Only
  /// Linux has these files, so on macOS it fails.
  pub fn read(pid: Option<i32>) -> Result<Self, Error> {
    platform::outcome(ffi::cz_proc_net_snmp_read(pid), "read network protocol counters")
      .map(|outcome| outcome.proc_net_snmp())
  }
}
