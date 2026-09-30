//! A container to boot, and a process to run in one.

use super::GIB;
use super::MIB;
use super::strings;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The VM a container runs in. `VMResources`.
///
/// Sized apart from the container's cgroup limits
/// ([`LinuxContainerConfiguration`]), with no headroom added.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VmResources {
  pub cpus: u32,
  /// Rounded up to the VMM's alignment.
  pub memory_in_bytes: u64,
}

impl VmResources {
  /// Memory for the guest kernel and `vminitd`. Never added for you.
  pub const GUEST_MEMORY_OVERHEAD: u64 = 128 * MIB;
}

/// 4 vCPUs and 1024 MiB, as `VMResources.default`.
impl Default for VmResources {
  fn default() -> Self {
    Self {
      cpus: 4,
      memory_in_bytes: GIB,
    }
  }
}

/// The device backing a mount, with its options. `Mount.RuntimeOptions`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeOptions {
  /// A host image file, as a block device.
  Virtioblk(Vec<String>),
  /// A shared host directory.
  Virtiofs(Vec<String>),
  /// Made by the guest alone (`proc`, `tmpfs`).
  Any(Vec<String>),
}

/// A filesystem mount exposed to a container. `Mount`.
///
/// Build one with [`Mount::share`], [`Mount::block`] or [`Mount::any`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mount {
  /// The filesystem type, as the mount syscall takes it.
  pub r#type: String,
  pub source: String,
  pub destination: String,
  pub options: Vec<String>,
  pub runtime_options: RuntimeOptions,
}

impl Mount {
  /// A host directory shared over virtiofs; `"ro"` makes it read-only.
  pub fn share(source: impl Into<String>, destination: impl Into<String>, options: &[&str]) -> Self {
    Self {
      r#type: "virtiofs".to_string(),
      source: source.into(),
      destination: destination.into(),
      options: strings(options),
      runtime_options: RuntimeOptions::Virtiofs(Vec::new()),
    }
  }

  /// A filesystem image on the host, attached as a block device.
  pub fn block(
    format: impl Into<String>,
    source: impl Into<String>,
    destination: impl Into<String>,
    options: &[&str],
  ) -> Self {
    Self {
      r#type: format.into(),
      source: source.into(),
      destination: destination.into(),
      options: strings(options),
      runtime_options: RuntimeOptions::Virtioblk(Vec::new()),
    }
  }

  /// A mount the guest makes by itself, such as a `tmpfs`.
  pub fn any(
    r#type: impl Into<String>,
    source: impl Into<String>,
    destination: impl Into<String>,
    options: &[&str],
  ) -> Self {
    Self {
      r#type: r#type.into(),
      source: source.into(),
      destination: destination.into(),
      options: strings(options),
      runtime_options: RuntimeOptions::Any(Vec::new()),
    }
  }
}

/// Which way a relayed socket is reached. `UnixSocketConfiguration.Direction`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Direction {
  /// A host socket shared into the guest.
  #[default]
  Into,
  /// A guest socket shared onto the host.
  OutOf,
}

/// A unix socket relayed between host and guest. `UnixSocketConfiguration`.
///
/// Not a [`Mount`]: mounting a socket relays nothing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnixSocketConfiguration {
  /// On the host for [`Direction::Into`], in the guest for [`Direction::OutOf`].
  pub source: PathBuf,
  /// In the guest for [`Direction::Into`], on the host for [`Direction::OutOf`].
  pub destination: PathBuf,
  /// Mode of the socket this creates. `None` leaves it to the relay.
  pub permissions: Option<u32>,
  pub direction: Direction,
}

impl UnixSocketConfiguration {
  /// A socket relayed into the guest with the relay's own mode.
  pub fn new(source: impl Into<PathBuf>, destination: impl Into<PathBuf>) -> Self {
    Self {
      source: source.into(),
      destination: destination.into(),
      permissions: None,
      direction: Direction::Into,
    }
  }
}

/// An interface on Virtualization.framework's NAT. `NATInterface`.
///
/// Addresses are static and caller-allocated; collisions go undetected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NatInterface {
  /// As CIDR.
  pub ipv4_address: String,
  pub ipv4_gateway: Option<String>,
  /// As CIDR.
  pub ipv6_address: Option<String>,
  pub ipv6_gateway: Option<String>,
  pub mac_address: Option<String>,
  pub mtu: u32,
}

impl NatInterface {
  pub fn new(ipv4_address: impl Into<String>, ipv4_gateway: impl Into<String>) -> Self {
    Self {
      ipv4_address: ipv4_address.into(),
      ipv4_gateway: Some(ipv4_gateway.into()),
      ipv6_address: None,
      ipv6_gateway: None,
      mac_address: None,
      mtu: 1500,
    }
  }
}

/// The guest's `/etc/resolv.conf`. `DNS`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dns {
  /// IPv4 or IPv6 addresses. A hostname is refused at boot.
  pub nameservers: Vec<String>,
  pub domain: Option<String>,
  pub search_domains: Vec<String>,
  /// Written as they are, e.g. `ndots:2`.
  pub options: Vec<String>,
}

impl Dns {
  /// `DNS.defaultNameservers`.
  pub fn default_nameservers() -> Vec<String> {
    strings(&["1.1.1.1"])
  }
}

impl Default for Dns {
  fn default() -> Self {
    Self {
      nameservers: Self::default_nameservers(),
      domain: None,
      search_domains: Vec::new(),
      options: Vec::new(),
    }
  }
}

/// One line of the guest's `/etc/hosts`. `Hosts.Entry`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostsEntry {
  pub ip_address: String,
  pub hostnames: Vec<String>,
  pub comment: Option<String>,
}

impl HostsEntry {
  pub fn new(ip_address: impl Into<String>, hostnames: &[&str]) -> Self {
    Self {
      ip_address: ip_address.into(),
      hostnames: strings(hostnames),
      comment: None,
    }
  }
}

/// The guest's `/etc/hosts`, written whole. `Hosts`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hosts {
  pub entries: Vec<HostsEntry>,
  /// Rendered at the top of the file.
  pub comment: Option<String>,
}

/// `Hosts.default`: localhost and the standard IPv6 names.
impl Default for Hosts {
  fn default() -> Self {
    Self {
      entries: vec![
        HostsEntry::new("127.0.0.1", &["localhost"]),
        HostsEntry::new("::1", &["localhost", "ip6-localhost", "ip6-loopback"]),
        HostsEntry::new("fe00::", &["ip6-localnet"]),
        HostsEntry::new("ff00::", &["ip6-mcastprefix"]),
        HostsEntry::new("ff02::1", &["ip6-allnodes"]),
        HostsEntry::new("ff02::2", &["ip6-allrouters"]),
      ],
      comment: None,
    }
  }
}

/// Where the guest's serial console is written. `BootLog.file`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootLog {
  pub path: PathBuf,
  /// Otherwise the file is replaced.
  pub append: bool,
}

impl BootLog {
  /// Appending, as `BootLog.file(path:)` does by default.
  pub fn file(path: impl Into<PathBuf>) -> Self {
    Self {
      path: path.into(),
      append: true,
    }
  }
}

/// A container's seccomp filter. `LinuxContainer.Configuration.SeccompProfile`.
///
/// Only an OCI runtime installs one: anything but `Unconfined` without
/// [`LinuxContainerConfiguration::oci_runtime_path`] fails the boot.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum SeccompProfile {
  #[default]
  Unconfined,
  /// containerd's allowlist, resolved against the process's capabilities.
  Default,
  /// The JSON of an OCI runtime spec's `linux.seccomp`, applied unvalidated. A
  /// Docker-format profile is refused.
  Profile(String),
}

/// Who a process runs as. `User`, from the OCI runtime spec.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct User {
  pub uid: u32,
  pub gid: u32,
  pub umask: Option<u32>,
  pub additional_gids: Vec<u32>,
  /// A name the guest resolves, e.g. from an image's `USER`. Empty for none.
  pub username: String,
}

impl User {
  /// A user the guest looks up by name, or as `uid[:gid]`.
  pub fn named(username: impl Into<String>) -> Self {
    Self {
      username: username.into(),
      ..Self::default()
    }
  }
}

/// A process in a container. `LinuxProcessConfiguration`.
///
/// Seeded from the image, as `ContainerManager` does; `None` keeps the image's.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LinuxProcessConfiguration {
  /// `None` is the image's entrypoint and command.
  pub arguments: Option<Vec<String>>,
  /// `NAME=VALUE`, appended to the image's, so these win.
  pub environment_variables: Vec<String>,
  /// `None` is the image's, or `/`.
  pub working_directory: Option<String>,
  /// `None` is the image's `USER`, or root.
  pub user: Option<User>,
}

impl LinuxProcessConfiguration {
  /// Runs `arguments`, everything else the image's.
  pub fn new(arguments: &[&str]) -> Self {
    Self {
      arguments: Some(strings(arguments)),
      ..Self::default()
    }
  }
}

/// A container. `LinuxContainer.Configuration`, with the same defaults.
///
/// `mounts`, `masked_paths` and `readonly_paths` start as the standard sets:
/// push to extend, replace to opt out.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinuxContainerConfiguration {
  /// The first process; when it exits, the container goes with it.
  pub process: LinuxProcessConfiguration,
  /// CPU cgroup limit, in cores. The VM is [`BootSpec::vm`].
  pub cpus: u32,
  /// Memory cgroup limit. The VM is [`BootSpec::vm`].
  pub memory_in_bytes: u64,
  /// `None` is the container's id.
  pub hostname: Option<String>,
  /// Kernel parameters, by dotted name (`net.core.somaxconn`).
  pub sysctl: BTreeMap<String, String>,
  /// The first is the default route. Empty means no network.
  pub interfaces: Vec<NatInterface>,
  pub sockets: Vec<UnixSocketConfiguration>,
  /// In order, which matters for nested paths.
  pub mounts: Vec<Mount>,
  /// Paths hidden from the container's processes.
  pub masked_paths: Vec<String>,
  /// Paths the container's processes cannot write.
  pub readonly_paths: Vec<String>,
  /// `None` keeps the image's `resolv.conf`, which usually resolves nothing;
  /// set it when the container has an interface.
  pub dns: Option<Dns>,
  /// `None` keeps the image's `/etc/hosts`.
  pub hosts: Option<Hosts>,
  /// Nested virtualization. M3 or later; elsewhere the boot fails.
  pub virtualization: bool,
  /// `None` is `bootlog.log` in the container's directory.
  pub boot_log: Option<BootLog>,
  /// **Experimental.** An OCI runtime (`runc`), as a path in the *init*
  /// filesystem. The default init image has none.
  pub oci_runtime_path: Option<String>,
  pub seccomp_profile: SeccompProfile,
  /// Run the first process under a minimal init (signal forwarding, reaping).
  pub use_init: bool,
}

impl LinuxContainerConfiguration {
  /// `LinuxContainer.defaultMounts()`.
  pub fn default_mounts() -> Vec<Mount> {
    let defaults = ["nosuid", "noexec", "nodev"];

    vec![
      Mount::any("proc", "proc", "/proc", &[]),
      Mount::any("sysfs", "sysfs", "/sys", &defaults),
      Mount::any("devtmpfs", "none", "/dev", &["nosuid", "mode=755"]),
      Mount::any("mqueue", "mqueue", "/dev/mqueue", &defaults),
      Mount::any(
        "tmpfs",
        "tmpfs",
        "/dev/shm",
        &["nosuid", "noexec", "nodev", "mode=1777", "size=65536k"],
      ),
      Mount::any("cgroup2", "none", "/sys/fs/cgroup", &defaults),
      Mount::any(
        "devpts",
        "devpts",
        "/dev/pts",
        &["nosuid", "noexec", "newinstance", "gid=5", "mode=0620", "ptmxmode=0666"],
      ),
    ]
  }

  /// `LinuxContainer.defaultMaskedPaths()`: the OCI runtime spec's.
  pub fn default_masked_paths() -> Vec<String> {
    strings(&[
      "/proc/asound",
      "/proc/acpi",
      "/proc/kcore",
      "/proc/keys",
      "/proc/latency_stats",
      "/proc/timer_list",
      "/proc/timer_stats",
      "/proc/sched_debug",
      "/proc/scsi",
      "/sys/firmware",
      "/sys/devices/virtual/powercap",
    ])
  }

  /// `LinuxContainer.defaultReadonlyPaths()`: the OCI runtime spec's.
  pub fn default_readonly_paths() -> Vec<String> {
    strings(&["/proc/bus", "/proc/fs", "/proc/irq", "/proc/sys", "/proc/sysrq-trigger"])
  }
}

impl Default for LinuxContainerConfiguration {
  fn default() -> Self {
    Self {
      process: LinuxProcessConfiguration::default(),
      cpus: 4,
      memory_in_bytes: GIB,
      hostname: None,
      sysctl: BTreeMap::new(),
      interfaces: Vec::new(),
      sockets: Vec::new(),
      mounts: Self::default_mounts(),
      masked_paths: Self::default_masked_paths(),
      readonly_paths: Self::default_readonly_paths(),
      dns: None,
      hosts: None,
      virtualization: false,
      boot_log: None,
      oci_runtime_path: None,
      seccomp_profile: SeccompProfile::Unconfined,
      use_init: false,
    }
  }
}

/// A container to create and start: `ContainerManager.create`'s arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootSpec {
  /// The container's id, and its directory in the store.
  pub id: String,
  /// Registry-qualified (nothing expands `debian:stable-slim`). Pulled if
  /// missing.
  pub reference: String,
  /// Sparse ceiling for the unpacked rootfs. Applies only to an image's first
  /// boot: later ones clone that unpack.
  pub rootfs_size_in_bytes: u64,
  pub vm: VmResources,
  pub configuration: LinuxContainerConfiguration,
}

impl BootSpec {
  /// `ContainerManager.create`'s own default.
  pub const DEFAULT_ROOTFS_SIZE_IN_BYTES: u64 = 8 * GIB;

  /// Everything else at Containerization's defaults.
  pub fn new(id: impl Into<String>, reference: impl Into<String>) -> Self {
    Self {
      id: id.into(),
      reference: reference.into(),
      rootfs_size_in_bytes: Self::DEFAULT_ROOTFS_SIZE_IN_BYTES,
      vm: VmResources::default(),
      configuration: LinuxContainerConfiguration::default(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn defaults_a_container_as_containerization_does() {
    let configuration = LinuxContainerConfiguration::default();

    assert_eq!(configuration.cpus, 4);
    assert_eq!(configuration.memory_in_bytes, GIB);
    assert_eq!(configuration.mounts.len(), 7);
    assert!(
      configuration
        .masked_paths
        .contains(&"/proc/kcore".to_string())
    );
    assert!(
      configuration
        .readonly_paths
        .contains(&"/proc/sys".to_string())
    );
    assert_eq!(configuration.dns, None, "nothing writes resolv.conf unless asked");
    assert_eq!(configuration.seccomp_profile, SeccompProfile::Unconfined);
  }

  #[test]
  fn defaults_the_rest_as_containerization_does() {
    assert_eq!(VmResources::default().memory_in_bytes, GIB);
    assert_eq!(Dns::default().nameservers, ["1.1.1.1"]);
    assert_eq!(Hosts::default().entries.len(), 6);
    assert!(BootLog::file("/boot.log").append);
    assert_eq!(UnixSocketConfiguration::new("/a", "/b").direction, Direction::Into);
    assert_eq!(NatInterface::new("10.0.0.2/24", "10.0.0.1").mtu, 1500);
    assert_eq!(BootSpec::new("id", "image").rootfs_size_in_bytes, 8 * GIB);
  }

  #[test]
  fn builds_mounts_of_each_kind() {
    let share = Mount::share("/Users/user/workspace", "/workspace", &["ro"]);
    assert_eq!(share.r#type, "virtiofs");
    assert_eq!(share.options, ["ro"]);
    assert_eq!(share.runtime_options, RuntimeOptions::Virtiofs(vec![]));

    assert_eq!(
      Mount::block("ext4", "/images/data.ext4", "/data", &[]).runtime_options,
      RuntimeOptions::Virtioblk(vec![])
    );
    assert_eq!(
      Mount::any("tmpfs", "tmpfs", "/scratch", &[]).runtime_options,
      RuntimeOptions::Any(vec![])
    );
  }
}
