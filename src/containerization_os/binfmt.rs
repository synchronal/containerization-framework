//! `Binfmt` and its nested `Entry`. Swift's `mounted()`, `mount()`,
//! `register(binaryPath:)` and `deregister()` are Linux only, and aren't
//! bound.

/// `Binfmt.path`.
pub const PATH: &str = "/proc/sys/fs/binfmt_misc";

/// `Binfmt.Entry`, which [`crate::containerization::vm::Vminitd::setup_emulator`]
/// registers in the guest.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Entry {
  pub name: String,
  pub r#type: String,
  pub offset: String,
  pub magic: String,
  pub mask: String,
  pub flags: String,
}

/// `Binfmt.Entry(name:type:offset:magic:mask:flags:)`'s defaulted arguments.
/// [`Default`] is Swift's defaults.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct EntryOptions {
  pub r#type: String,
  pub offset: String,
  pub flags: String,
}

impl Default for EntryOptions {
  fn default() -> Self {
    Self {
      r#type: "M".to_string(),
      offset: String::new(),
      flags: "CF".to_string(),
    }
  }
}

impl Entry {
  /// `Binfmt.Entry(name:type:offset:magic:mask:flags:)`.
  pub fn new(name: &str, magic: &str, mask: &str, options: EntryOptions) -> Self {
    Self {
      name: name.to_string(),
      r#type: options.r#type,
      offset: options.offset,
      magic: magic.to_string(),
      mask: mask.to_string(),
      flags: options.flags,
    }
  }

  /// `Binfmt.Entry.amd64()`: the entry for amd64 ELF binaries. Its magic and
  /// mask are escapes the kernel reads, not the bytes themselves.
  pub fn amd64() -> Self {
    Self::new(
      "x86_64",
      r"\x7fELF\x02\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02\x00\x3e\x00",
      r"\xff\xff\xff\xff\xff\xfe\xfe\x00\xff\xff\xff\xff\xff\xff\xff\xff\xfe\xff\xff\xff",
      EntryOptions::default(),
    )
  }
}
