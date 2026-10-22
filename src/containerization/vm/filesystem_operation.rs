use crate::platform::ffi;

/// `FilesystemOperation`, which
/// [`crate::containerization::container::LinuxContainer::filesystem_operation`]
/// performs.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FilesystemOperation {
  Freeze,
  Thaw,
  Trim,
}

same_variants!(FilesystemOperation, ffi::FilesystemOperationKind { Freeze, Thaw, Trim });
