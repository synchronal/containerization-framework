//! Getters for a virtual machine's configuration, boot log, kernel and
//! platform.

use super::network::interface_kind;
use super::network::nat_interface;
use super::network::vmnet_interface;
use crate::bridge::accessors;
use crate::bridge::ffi;
use crate::containerization::container;
use crate::containerization::network;
use crate::containerization::vm;
use crate::containerization::vm::kernel;
use crate::containerization::vm::system_platform;
use std::collections::BTreeMap;
use std::path::PathBuf;

impl ffi::CzOutcome {
  /// The `AttachedFilesystem` an outcome holds, read field by field.
  pub(crate) fn attached_filesystem(&self) -> vm::AttachedFilesystem {
    vm::AttachedFilesystem {
      r#type: self.attached_filesystem_type(),
      source: self.attached_filesystem_source(),
      destination: self.attached_filesystem_destination(),
      options: self.attached_filesystem_options(),
    }
  }

  /// A held `[String: [AttachedFilesystem]]`.
  pub(crate) fn attached_filesystems_by_id(&self) -> BTreeMap<String, Vec<vm::AttachedFilesystem>> {
    self.map_of(|filesystems| filesystems.list(Self::attached_filesystem))
  }
}

impl From<ffi::InstanceState> for vm::VirtualMachineInstanceState {
  fn from(state: ffi::InstanceState) -> Self {
    match state {
      ffi::InstanceState::Starting => Self::Starting,
      ffi::InstanceState::Running => Self::Running,
      ffi::InstanceState::Stopped => Self::Stopped,
      ffi::InstanceState::Stopping => Self::Stopping,
      ffi::InstanceState::Unknown => Self::Unknown,
    }
  }
}

impl From<ffi::VirtiofsLayoutKind> for vm::VirtiofsLayout {
  fn from(layout: ffi::VirtiofsLayoutKind) -> Self {
    match layout {
      ffi::VirtiofsLayoutKind::Unified => Self::Unified,
      ffi::VirtiofsLayoutKind::PerTag => Self::PerTag,
    }
  }
}

impl From<kernel::LogLevel> for ffi::LogLevel {
  fn from(level: kernel::LogLevel) -> Self {
    match level {
      kernel::LogLevel::Trace => Self::Trace,
      kernel::LogLevel::Debug => Self::Debug,
      kernel::LogLevel::Info => Self::Info,
      kernel::LogLevel::Notice => Self::Notice,
      kernel::LogLevel::Warning => Self::Warning,
      kernel::LogLevel::Error => Self::Error,
      kernel::LogLevel::Critical => Self::Critical,
    }
  }
}

impl vm::BootLog {
  pub(crate) fn kind(&self) -> ffi::BootLogKind {
    match self {
      Self::File { .. } => ffi::BootLogKind::File,
      Self::FileHandle(_) => ffi::BootLogKind::FileHandle,
    }
  }

  /// Empty unless [`Self::kind`] is `File`, and `None` if the file's path
  /// isn't UTF-8.
  pub(crate) fn path(&self) -> Option<String> {
    match self {
      Self::File { path, .. } => accessors::path(path),
      Self::FileHandle(_) => Some(String::new()),
    }
  }

  /// The file's path with any bytes that aren't UTF-8 replaced, for the
  /// error Swift throws when [`Self::path`] is `None`.
  pub(crate) fn lossy_path(&self) -> String {
    match self {
      Self::File { path, .. } => accessors::lossy_path(path),
      Self::FileHandle(_) => String::new(),
    }
  }

  pub(crate) fn append(&self) -> bool {
    matches!(self, Self::File { append: true, .. })
  }

  /// `-1` unless [`Self::kind`] is `FileHandle`.
  pub(crate) fn file_handle(&self) -> i32 {
    match self {
      Self::FileHandle(descriptor) => *descriptor,
      Self::File { .. } => -1,
    }
  }
}

pub(in super::super) fn boot_log_file(path: String, append: bool) -> vm::BootLog {
  vm::BootLog::File {
    path: PathBuf::from(path),
    append,
  }
}

impl vm::VMConfiguration {
  pub(crate) fn cpus(&self) -> u32 {
    self.cpus
  }

  pub(crate) fn memory_in_bytes(&self) -> u64 {
    self.memory_in_bytes
  }

  pub(crate) fn interfaces_len(&self) -> usize {
    self.interfaces.len()
  }

  pub(crate) fn interface_kind_at(&self, index: usize) -> ffi::InterfaceKind {
    interface_kind(&self.interfaces[index])
  }

  /// Only when [`Self::interface_kind_at`] is `Nat`.
  pub(crate) fn nat_interface_at(&self, index: usize) -> &network::NATInterface {
    nat_interface(&self.interfaces[index])
  }

  /// Only when [`Self::interface_kind_at`] is `Vmnet`.
  pub(crate) fn vmnet_interface_at(&self, index: usize) -> ffi::CzVmnetInterface {
    vmnet_interface(&self.interfaces[index])
  }

  pub(crate) fn mounts_by_id_len(&self) -> usize {
    self.mounts_by_id.len()
  }

  pub(crate) fn mounts_by_id_key_at(&self, index: usize) -> &str {
    self.mounts_by_id_entry(index).0
  }

  pub(crate) fn mounts_by_id_mounts_len(&self, index: usize) -> usize {
    self.mounts_by_id_entry(index).1.len()
  }

  pub(crate) fn mounts_by_id_mount_at(&self, index: usize, mount: usize) -> &container::Mount {
    &self.mounts_by_id_entry(index).1[mount]
  }

  fn mounts_by_id_entry(&self, index: usize) -> (&str, &Vec<container::Mount>) {
    accessors::entry_at(&self.mounts_by_id, index)
  }

  pub(crate) fn has_boot_log(&self) -> bool {
    self.boot_log.is_some()
  }

  /// Only when [`Self::has_boot_log`].
  pub(crate) fn boot_log(&self) -> &vm::BootLog {
    accessors::present(&self.boot_log)
  }

  pub(crate) fn nested_virtualization(&self) -> bool {
    self.nested_virtualization
  }
}

impl vm::AttachedFilesystem {
  pub(crate) fn filesystem_type(&self) -> &str {
    &self.r#type
  }

  pub(crate) fn source(&self) -> &str {
    &self.source
  }

  pub(crate) fn destination(&self) -> &str {
    &self.destination
  }

  pub(crate) fn options_len(&self) -> usize {
    self.options.len()
  }

  pub(crate) fn options_at(&self, index: usize) -> &str {
    &self.options[index]
  }
}

impl From<system_platform::OS> for ffi::PlatformOs {
  fn from(os: system_platform::OS) -> Self {
    match os {
      system_platform::OS::Linux => Self::Linux,
      system_platform::OS::Darwin => Self::Darwin,
    }
  }
}

impl From<ffi::PlatformOs> for system_platform::OS {
  fn from(os: ffi::PlatformOs) -> Self {
    match os {
      ffi::PlatformOs::Linux => Self::Linux,
      ffi::PlatformOs::Darwin => Self::Darwin,
    }
  }
}

impl From<system_platform::Architecture> for ffi::PlatformArchitecture {
  fn from(architecture: system_platform::Architecture) -> Self {
    match architecture {
      system_platform::Architecture::Arm64 => Self::Arm64,
      system_platform::Architecture::Amd64 => Self::Amd64,
    }
  }
}

impl From<ffi::PlatformArchitecture> for system_platform::Architecture {
  fn from(architecture: ffi::PlatformArchitecture) -> Self {
    match architecture {
      ffi::PlatformArchitecture::Arm64 => Self::Arm64,
      ffi::PlatformArchitecture::Amd64 => Self::Amd64,
    }
  }
}

impl vm::SystemPlatform {
  pub(crate) fn os(&self) -> ffi::PlatformOs {
    self.os.into()
  }

  pub(crate) fn architecture(&self) -> ffi::PlatformArchitecture {
    self.architecture.into()
  }
}

impl vm::Kernel {
  /// The kernel's path, or `None` if it isn't UTF-8.
  pub(crate) fn path(&self) -> Option<String> {
    accessors::path(&self.path)
  }

  /// The kernel's path with any bytes that aren't UTF-8 replaced, for the
  /// error Swift throws when [`Self::path`] is `None`.
  pub(crate) fn lossy_path(&self) -> String {
    accessors::lossy_path(&self.path)
  }

  pub(crate) fn platform(&self) -> &vm::SystemPlatform {
    &self.platform
  }

  pub(crate) fn kernel_args_len(&self) -> usize {
    self.command_line.kernel_args.len()
  }

  pub(crate) fn kernel_args_at(&self, index: usize) -> &str {
    &self.command_line.kernel_args[index]
  }

  pub(crate) fn init_args_len(&self) -> usize {
    self.command_line.init_args.len()
  }

  pub(crate) fn init_args_at(&self, index: usize) -> &str {
    &self.command_line.init_args[index]
  }
}

#[cfg(test)]
mod tests {
  use crate::bridge::ffi;
  use crate::containerization::vm;
  use crate::containerization::vm::system_platform;

  #[test]
  fn copies_swifts_system_platform_raw_values() {
    let os = system_platform::OS::ALL_CASES.iter().map(|os| os.as_str());
    let architectures = system_platform::Architecture::ALL_CASES
      .iter()
      .map(|architecture| architecture.as_str());

    assert_eq!(
      ffi::cz_system_platform_raw_values(),
      os.chain(architectures).collect::<Vec<_>>()
    );
  }

  #[test]
  fn lends_no_kernel_path_that_isnt_utf8() {
    use std::os::unix::ffi::OsStrExt;

    let path = std::ffi::OsStr::from_bytes(b"/tmp/\xff");
    let kernel = vm::Kernel::new(path, vm::SystemPlatform::LINUX_ARM);

    assert_eq!(kernel.path(), None);
    assert_eq!(kernel.lossy_path(), "/tmp/\u{FFFD}");
    assert_eq!(
      vm::Kernel::new("/vmlinux", vm::SystemPlatform::LINUX_ARM).path(),
      Some("/vmlinux".to_string())
    );
  }
}
