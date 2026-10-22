//! Getters and setters for Containerization's value types.
//!
//! The macros here write out what the container, pod and VM configurations
//! share, each in the `impl` of a configuration that has those fields.

/// The getters for a configuration's `interfaces`, which the container, pod
/// and VM configurations share.
macro_rules! interface_getters {
  () => {
    pub(crate) fn interfaces_len(&self) -> usize {
      self.interfaces.len()
    }

    pub(crate) fn interface_kind_at(&self, index: usize) -> $crate::bridge::ffi::InterfaceKind {
      $crate::bridge::accessors::container::network::interface_kind(&self.interfaces[index])
    }

    /// Swift calls this only when [`Self::interface_kind_at`] is `Nat`.
    pub(crate) fn nat_interface_at(&self, index: usize) -> &$crate::containerization::network::NATInterface {
      $crate::bridge::accessors::container::network::nat_interface(&self.interfaces[index])
    }

    /// Swift calls this only when [`Self::interface_kind_at`] is `Vmnet`.
    pub(crate) fn vmnet_interface_at(&self, index: usize) -> $crate::bridge::ffi::CzVmnetInterface {
      $crate::bridge::accessors::container::network::vmnet_interface(&self.interfaces[index])
    }
  };
}

/// The setters for a configuration's `interfaces`, `dns` and `hosts`, which
/// the container and pod configurations share.
macro_rules! network_setters {
  () => {
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
      self
        .interfaces
        .push($crate::bridge::accessors::container::network::nat(
          ipv4_address,
          ipv4_prefix,
          ipv4_gateway,
          mac_address,
          mtu,
        ));
    }

    pub(crate) fn push_vmnet_interface(&mut self, interface: $crate::bridge::ffi::CzVmnetInterface) {
      self
        .interfaces
        .push($crate::bridge::accessors::container::network::vmnet(interface));
    }

    /// Swift calls this only after [`Self::push_interface`], and it sets the
    /// interface that call pushed.
    pub(crate) fn set_interface_ipv6_address(&mut self, high: u64, low: u64, zone: Option<String>, prefix: u8) {
      $crate::bridge::accessors::container::network::set_ipv6_address(&mut self.interfaces, high, low, zone, prefix);
    }

    /// Swift calls this only after [`Self::push_interface`], and it sets the
    /// interface that call pushed.
    pub(crate) fn set_interface_ipv6_gateway(&mut self, high: u64, low: u64, zone: Option<String>) {
      $crate::bridge::accessors::container::network::set_ipv6_gateway(&mut self.interfaces, high, low, zone);
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
      self.dns = Some($crate::bridge::accessors::container::network::dns(
        nameservers,
        domain,
        search_domains,
        options,
      ));
    }

    pub(crate) fn clear_hosts(&mut self) {
      self.hosts = None;
    }

    pub(crate) fn set_hosts(&mut self, comment: Option<String>) {
      self.hosts = Some($crate::bridge::accessors::container::network::hosts(comment));
    }

    /// Swift calls this only after [`Self::set_hosts`].
    pub(crate) fn push_hosts_entry(&mut self, ip_address: String, hostnames: Vec<String>, comment: Option<String>) {
      $crate::bridge::accessors::container::network::push_hosts_entry(&mut self.hosts, ip_address, hostnames, comment);
    }
  };
}

/// The setters for a configuration's `boot_log`, which the container and pod
/// configurations share.
macro_rules! boot_log_setters {
  () => {
    pub(crate) fn clear_boot_log(&mut self) {
      self.boot_log = None;
    }

    pub(crate) fn set_boot_log_file(&mut self, path: String, append: bool) {
      self.boot_log = Some($crate::bridge::accessors::container::vm::boot_log_file(path, append));
    }

    pub(crate) fn set_boot_log_file_handle(&mut self, descriptor: i32) {
      self.boot_log = Some($crate::containerization::vm::BootLog::FileHandle(descriptor));
    }
  };
}

/// The setter and getters for a configuration's `seccomp_profile`, which the
/// container and pod configurations share.
macro_rules! seccomp_accessors {
  () => {
    /// `profile` holds the `LinuxSeccomp` of a `Profile`, or `Absent`.
    pub(crate) fn set_seccomp_profile(
      &mut self,
      mode: $crate::bridge::ffi::SeccompMode,
      profile: $crate::bridge::ffi::CzOutcome,
    ) {
      self.seccomp_profile = $crate::bridge::accessors::container::configuration::seccomp_profile(mode, profile);
    }

    pub(crate) fn seccomp_mode(&self) -> $crate::bridge::ffi::SeccompMode {
      $crate::bridge::accessors::container::configuration::seccomp_mode(&self.seccomp_profile)
    }

    /// Swift calls this only when [`Self::seccomp_mode`] is `Profile`.
    pub(crate) fn seccomp_profile(&self) -> &$crate::containerization_oci::runtime::LinuxSeccomp {
      $crate::bridge::accessors::container::configuration::custom_seccomp(&self.seccomp_profile)
    }
  };
}

mod configuration;
mod manager;
mod network;
mod pod;
mod process;
mod statistics;
mod vm;
