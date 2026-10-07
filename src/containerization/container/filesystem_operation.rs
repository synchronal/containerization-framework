/// `FilesystemOperation`, which
/// [`super::LinuxContainer::filesystem_operation`] performs.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FilesystemOperation {
  Freeze,
  Thaw,
  Trim,
}
