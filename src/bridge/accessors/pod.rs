//! Getters and setters for `LinuxPod`'s configurations, which share the
//! container configuration's helpers.

use super::container;
use super::entry_at;
use super::present;
use crate::bridge::ffi;
use crate::containerization;
use crate::containerization::container::linux_pod;
use crate::containerization::container::linux_pod::pod_volume;
use crate::containerization_oci::runtime;
use std::path::PathBuf;
use std::time::Duration;

impl linux_pod::Configuration {
  pub(crate) fn set_virtualization(&mut self, virtualization: bool) {
    self.virtualization = virtualization;
  }

  pub(crate) fn set_share_process_namespace(&mut self, share_process_namespace: bool) {
    self.share_process_namespace = share_process_namespace;
  }

  pub(crate) fn set_hostname(&mut self, hostname: Option<String>) {
    self.hostname = hostname;
  }

  pub(crate) fn set_oci_runtime_path(&mut self, oci_runtime_path: Option<String>) {
    self.oci_runtime_path = oci_runtime_path;
  }

  pub(crate) fn clear_interfaces(&mut self) {
    self.interfaces.clear();
  }

  pub(crate) fn push_interface(
    &mut self,
    ipv4_address: u32,
    ipv4_prefix: u8,
    ipv4_gateway: Option<u32>,
    mac_address: Option<u64>,
    mtu: u32,
  ) {
    self.interfaces.push(container::nat(
      ipv4_address,
      ipv4_prefix,
      ipv4_gateway,
      mac_address,
      mtu,
    ));
  }

  /// Only after [`Self::push_interface`], for the interface it pushed.
  pub(crate) fn set_interface_ipv6_address(&mut self, high: u64, low: u64, zone: Option<String>, prefix: u8) {
    container::set_ipv6_address(&mut self.interfaces, high, low, zone, prefix);
  }

  /// Only after [`Self::push_interface`], for the interface it pushed.
  pub(crate) fn set_interface_ipv6_gateway(&mut self, high: u64, low: u64, zone: Option<String>) {
    container::set_ipv6_gateway(&mut self.interfaces, high, low, zone);
  }

  pub(crate) fn push_vmnet_interface(&mut self, interface: ffi::CzVmnetInterface) {
    self.interfaces.push(container::vmnet(interface));
  }

  pub(crate) fn clear_dns(&mut self) {
    self.dns = None;
  }

  pub(crate) fn set_dns(
    &mut self,
    nameservers: Vec<String>,
    domain: Option<String>,
    search_domains: Vec<String>,
    options: Vec<String>,
  ) {
    self.dns = Some(container::dns(nameservers, domain, search_domains, options));
  }

  pub(crate) fn clear_hosts(&mut self) {
    self.hosts = None;
  }

  pub(crate) fn set_hosts(&mut self, comment: Option<String>) {
    self.hosts = Some(container::hosts(comment));
  }

  /// Only after [`Self::set_hosts`].
  pub(crate) fn push_hosts_entry(&mut self, ip_address: String, hostnames: Vec<String>, comment: Option<String>) {
    container::push_hosts_entry(&mut self.hosts, ip_address, hostnames, comment);
  }

  pub(crate) fn clear_boot_log(&mut self) {
    self.boot_log = None;
  }

  pub(crate) fn set_boot_log_file(&mut self, path: String, append: bool) {
    self.boot_log = Some(container::boot_log_file(path, append));
  }

  pub(crate) fn set_boot_log_file_handle(&mut self, descriptor: i32) {
    self.boot_log = Some(containerization::vm::BootLog::FileHandle(descriptor));
  }

  /// `profile` holds the `LinuxSeccomp` of a `Profile`, or `Absent`.
  pub(crate) fn set_seccomp_profile(&mut self, mode: ffi::SeccompMode, profile: ffi::CzOutcome) {
    self.seccomp_profile = container::seccomp_profile(mode, profile);
  }

  pub(crate) fn clear_volumes(&mut self) {
    self.volumes.clear();
  }

  /// `location`, `timeout`, `read_only` and `size_bytes` as
  /// [`linux_pod::PodVolume`]'s getters give them.
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn push_volume(
    &mut self,
    name: String,
    format: String,
    kind: ffi::PodVolumeKind,
    location: String,
    timeout: Option<f64>,
    read_only: bool,
    size_bytes: Option<u64>,
  ) {
    let source = match kind {
      ffi::PodVolumeKind::Nbd => pod_volume::Source::Nbd {
        url: location,
        timeout: timeout.map(Duration::from_secs_f64),
        read_only,
      },
      ffi::PodVolumeKind::DiskImage => pod_volume::Source::DiskImage {
        path: PathBuf::from(location),
        read_only,
      },
      ffi::PodVolumeKind::Tmpfs => pod_volume::Source::Tmpfs { size_bytes },
    };

    self
      .volumes
      .push(linux_pod::PodVolume { name, source, format });
  }

  pub(crate) fn interfaces_len(&self) -> usize {
    self.interfaces.len()
  }

  pub(crate) fn interface_kind_at(&self, index: usize) -> ffi::InterfaceKind {
    container::interface_kind(&self.interfaces[index])
  }

  /// Only when [`Self::interface_kind_at`] is `Nat`.
  pub(crate) fn nat_interface_at(&self, index: usize) -> &containerization::network::NATInterface {
    container::nat_interface(&self.interfaces[index])
  }

  /// Only when [`Self::interface_kind_at`] is `Vmnet`.
  pub(crate) fn vmnet_interface_at(&self, index: usize) -> ffi::CzVmnetInterface {
    container::vmnet_interface(&self.interfaces[index])
  }

  pub(crate) fn virtualization(&self) -> bool {
    self.virtualization
  }

  pub(crate) fn has_boot_log(&self) -> bool {
    self.boot_log.is_some()
  }

  /// Only when [`Self::has_boot_log`].
  pub(crate) fn boot_log(&self) -> &containerization::vm::BootLog {
    present(&self.boot_log)
  }

  pub(crate) fn share_process_namespace(&self) -> bool {
    self.share_process_namespace
  }

  pub(crate) fn hostname(&self) -> Option<&str> {
    self.hostname.as_deref()
  }

  pub(crate) fn has_dns(&self) -> bool {
    self.dns.is_some()
  }

  /// Only when [`Self::has_dns`].
  pub(crate) fn dns(&self) -> &containerization::network::DNS {
    present(&self.dns)
  }

  pub(crate) fn has_hosts(&self) -> bool {
    self.hosts.is_some()
  }

  /// Only when [`Self::has_hosts`].
  pub(crate) fn hosts(&self) -> &containerization::network::Hosts {
    present(&self.hosts)
  }

  pub(crate) fn volumes_len(&self) -> usize {
    self.volumes.len()
  }

  pub(crate) fn volumes_at(&self, index: usize) -> &linux_pod::PodVolume {
    &self.volumes[index]
  }

  pub(crate) fn oci_runtime_path(&self) -> Option<&str> {
    self.oci_runtime_path.as_deref()
  }

  pub(crate) fn seccomp_mode(&self) -> ffi::SeccompMode {
    container::seccomp_mode(&self.seccomp_profile)
  }

  /// Only when [`Self::seccomp_mode`] is `Profile`.
  pub(crate) fn seccomp_profile(&self) -> &runtime::LinuxSeccomp {
    container::custom_seccomp(&self.seccomp_profile)
  }
}

impl linux_pod::PodVolume {
  pub(crate) fn name(&self) -> &str {
    &self.name
  }

  pub(crate) fn format(&self) -> &str {
    &self.format
  }

  pub(crate) fn source_kind(&self) -> ffi::PodVolumeKind {
    match self.source {
      pod_volume::Source::Nbd { .. } => ffi::PodVolumeKind::Nbd,
      pod_volume::Source::DiskImage { .. } => ffi::PodVolumeKind::DiskImage,
      pod_volume::Source::Tmpfs { .. } => ffi::PodVolumeKind::Tmpfs,
    }
  }

  /// An `nbd` source's URL or a `diskImage`'s path, and empty for a `tmpfs`.
  pub(crate) fn location(&self) -> String {
    match &self.source {
      pod_volume::Source::Nbd { url, .. } => url.clone(),
      pod_volume::Source::DiskImage { path, .. } => super::path(path),
      pod_volume::Source::Tmpfs { .. } => String::new(),
    }
  }

  /// An `nbd` source's, in seconds.
  pub(crate) fn timeout(&self) -> Option<f64> {
    match self.source {
      pod_volume::Source::Nbd { timeout, .. } => timeout.map(|timeout| timeout.as_secs_f64()),
      _ => None,
    }
  }

  pub(crate) fn read_only(&self) -> bool {
    match self.source {
      pod_volume::Source::Nbd { read_only, .. } | pod_volume::Source::DiskImage { read_only, .. } => read_only,
      pod_volume::Source::Tmpfs { .. } => false,
    }
  }

  /// A `tmpfs` source's.
  pub(crate) fn size_bytes(&self) -> Option<u64> {
    match self.source {
      pod_volume::Source::Tmpfs { size_bytes } => size_bytes,
      _ => None,
    }
  }
}

impl linux_pod::ContainerConfiguration {
  pub(crate) fn process(&self) -> &containerization::process::LinuxProcessConfiguration {
    &self.process
  }

  pub(crate) fn cpus(&self) -> u32 {
    self.cpus
  }

  pub(crate) fn memory_in_bytes(&self) -> u64 {
    self.memory_in_bytes
  }

  pub(crate) fn hostname(&self) -> Option<&str> {
    self.hostname.as_deref()
  }

  pub(crate) fn sysctl_len(&self) -> usize {
    self.sysctl.len()
  }

  pub(crate) fn sysctl_key_at(&self, index: usize) -> &str {
    entry_at(&self.sysctl, index).0
  }

  pub(crate) fn sysctl_value_at(&self, index: usize) -> &str {
    entry_at(&self.sysctl, index).1
  }

  pub(crate) fn mounts_len(&self) -> usize {
    self.mounts.len()
  }

  pub(crate) fn mounts_at(&self, index: usize) -> &containerization::container::Mount {
    &self.mounts[index]
  }

  pub(crate) fn masked_paths_len(&self) -> usize {
    self.masked_paths.len()
  }

  pub(crate) fn masked_paths_at(&self, index: usize) -> &str {
    &self.masked_paths[index]
  }

  pub(crate) fn readonly_paths_len(&self) -> usize {
    self.readonly_paths.len()
  }

  pub(crate) fn readonly_paths_at(&self, index: usize) -> &str {
    &self.readonly_paths[index]
  }

  pub(crate) fn sockets_len(&self) -> usize {
    self.sockets.len()
  }

  pub(crate) fn sockets_at(&self, index: usize) -> ffi::CzUnixSocketConfiguration {
    self.sockets[index].handle.duplicate()
  }

  pub(crate) fn has_dns(&self) -> bool {
    self.dns.is_some()
  }

  /// Only when [`Self::has_dns`].
  pub(crate) fn dns(&self) -> &containerization::network::DNS {
    present(&self.dns)
  }

  pub(crate) fn has_hosts(&self) -> bool {
    self.hosts.is_some()
  }

  /// Only when [`Self::has_hosts`].
  pub(crate) fn hosts(&self) -> &containerization::network::Hosts {
    present(&self.hosts)
  }

  pub(crate) fn has_seccomp_profile(&self) -> bool {
    self.seccomp_profile.is_some()
  }

  /// Only when [`Self::has_seccomp_profile`].
  pub(crate) fn seccomp_mode(&self) -> ffi::SeccompMode {
    container::seccomp_mode(self.own_seccomp_profile())
  }

  /// Only when [`Self::seccomp_mode`] is `Profile`.
  pub(crate) fn seccomp_profile(&self) -> &runtime::LinuxSeccomp {
    container::custom_seccomp(self.own_seccomp_profile())
  }

  fn own_seccomp_profile(&self) -> &containerization::container::linux_container::SeccompProfile {
    present(&self.seccomp_profile)
  }

  pub(crate) fn use_init(&self) -> bool {
    self.use_init
  }
}
