//! Getters and setters for `LinuxPod`'s configurations, which share the
//! container configuration's helpers.

use super::configuration;
use crate::bridge::accessors;
use crate::bridge::accessors::present;
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

  network_setters!();
  boot_log_setters!();
  seccomp_accessors!();
  interface_getters!();

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
        // A `Duration` near its maximum crosses as a `Double` just above it,
        // which `try_from_secs_f64` rejects.
        timeout: timeout.map(|seconds| Duration::try_from_secs_f64(seconds).unwrap_or(Duration::MAX)),
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

  getters! {
    copy virtualization: bool;
    present boot_log: containerization::vm::BootLog => has_boot_log;
    copy share_process_namespace: bool;
    optional_str hostname;
    present dns: containerization::network::DNS => has_dns;
    present hosts: containerization::network::Hosts => has_hosts;
    list volumes: linux_pod::PodVolume => volumes_len, volumes_at;
    optional_str oci_runtime_path;
  }
}

impl linux_pod::PodVolume {
  getters! {
    str name;
    str format;
  }

  pub(crate) fn source_kind(&self) -> ffi::PodVolumeKind {
    match self.source {
      pod_volume::Source::Nbd { .. } => ffi::PodVolumeKind::Nbd,
      pod_volume::Source::DiskImage { .. } => ffi::PodVolumeKind::DiskImage,
      pod_volume::Source::Tmpfs { .. } => ffi::PodVolumeKind::Tmpfs,
    }
  }

  /// An `nbd` source's URL or a `diskImage`'s path, and empty for a `tmpfs`.
  /// It is `None` if a `diskImage`'s path isn't UTF-8.
  pub(crate) fn location(&self) -> Option<String> {
    match &self.source {
      pod_volume::Source::Nbd { url, .. } => Some(url.clone()),
      pod_volume::Source::DiskImage { path, .. } => accessors::path(path),
      pod_volume::Source::Tmpfs { .. } => Some(String::new()),
    }
  }

  /// [`Self::location`] with any bytes that aren't UTF-8 replaced, for the
  /// error Swift throws when it is `None`.
  pub(crate) fn lossy_location(&self) -> String {
    match &self.source {
      pod_volume::Source::Nbd { url, .. } => url.clone(),
      pod_volume::Source::DiskImage { path, .. } => accessors::lossy_path(path),
      pod_volume::Source::Tmpfs { .. } => String::new(),
    }
  }

  /// An `nbd` source's timeout, in seconds.
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

  /// A `tmpfs` source's size.
  pub(crate) fn size_bytes(&self) -> Option<u64> {
    match self.source {
      pod_volume::Source::Tmpfs { size_bytes } => size_bytes,
      _ => None,
    }
  }
}

impl linux_pod::ContainerConfiguration {
  getters! {
    borrow process: containerization::process::LinuxProcessConfiguration;
    copy cpus: u32;
    copy memory_in_bytes: u64;
    optional_str hostname;
    map sysctl => sysctl_len, sysctl_key_at, sysctl_value_at;
    list mounts: containerization::container::Mount => mounts_len, mounts_at;
    list masked_paths: str => masked_paths_len, masked_paths_at;
    list readonly_paths: str => readonly_paths_len, readonly_paths_at;
  }

  pub(crate) fn sockets_len(&self) -> usize {
    self.sockets.len()
  }

  pub(crate) fn sockets_at(&self, index: usize) -> ffi::CzUnixSocketConfiguration {
    self.sockets[index].handle.duplicate()
  }

  getters! {
    present dns: containerization::network::DNS => has_dns;
    present hosts: containerization::network::Hosts => has_hosts;
  }

  pub(crate) fn has_seccomp_profile(&self) -> bool {
    self.seccomp_profile.is_some()
  }

  /// Swift calls this only when [`Self::has_seccomp_profile`] is true.
  pub(crate) fn seccomp_mode(&self) -> ffi::SeccompMode {
    configuration::seccomp_mode(self.own_seccomp_profile())
  }

  /// Swift calls this only when [`Self::seccomp_mode`] is `Profile`.
  pub(crate) fn seccomp_profile(&self) -> &runtime::LinuxSeccomp {
    configuration::custom_seccomp(self.own_seccomp_profile())
  }

  fn own_seccomp_profile(&self) -> &containerization::container::linux_container::configuration::SeccompProfile {
    present(&self.seccomp_profile)
  }

  getters! {
    copy use_init: bool;
    map unified => unified_len, unified_key_at, unified_value_at;
  }
}
