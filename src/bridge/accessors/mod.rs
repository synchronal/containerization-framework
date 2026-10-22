//! Crate-private getters Swift reads value types through, the setters it
//! fills them with, and the `CzOutcome` readers that turn what Swift returns
//! into Rust values.
//!
//! The files are grouped by the public module each type is in, not by the
//! bridge module that declares it. `images.rs` covers Containerization's and
//! OCI's image types, `container/` the rest of the `containerization`
//! module, `spec/` the OCI runtime spec, and `os.rs` all of
//! `containerization_os`, including the `binfmt::Entry` that `bridge/vm.rs`
//! declares.
//!
//! Each getter lends what the value holds, so Swift's copy is the only one.
//! Swift calls each by the `swift_name` it has in `bridge::ffi`, which is the
//! Rust name in camel case.
//!
//! A shape swift-bridge can't return becomes a set of getters. An optional
//! nested value is read through `has_x` and `x`, and a list through `x_len`
//! and `x_at`. A map is read as its length and the key and value at an
//! index, and an enum as its kind and then its payload. Swift asks for an
//! index only below the length.
//!
//! Swift never makes a Rust value. It fills one that Rust hands it, clearing
//! and pushing what it holds, so that it can hand back a configuration it
//! seeded. A Rust function that returned a new one to Swift would get a
//! redundant cast in swift-bridge's glue.
//!
//! A getter that only lends a field is written as an entry in `getters!`,
//! and the rest are written out. `container/mod.rs` has the macros for what
//! the container, pod and VM configurations share.

/// Getters that lend a field as it is, written as one entry for each field.
/// An entry names the kind of field, then the field, then the getters' names
/// where they differ from the field's. Each getter keeps the name Swift calls
/// it by.
///
/// - `copy x: T;` returns a field that is `Copy`.
/// - `str x;` lends a `String` as `&str`, and `str x => name;` names the
///   getter.
/// - `optional_str x;` lends an `Option<String>` as `Option<&str>`.
/// - `borrow x: T;` lends a `T`.
/// - `present x: T => has_x;` lends an `Option<T>` through `has_x` and `x`,
///   which Swift calls only when `has_x` is true.
/// - `list x: T => x_len, x_at;` lends a `Vec<T>` as its length and the
///   element at an index. `T` is `str` for a `Vec<String>`.
/// - `optional_list x: T => has_x, x_len, x_at;` lends an `Option<Vec<T>>`
///   the same way, with a length of zero when it is `None`.
/// - `map x => x_len, x_key_at, x_value_at;` lends a `BTreeMap<String, String>`
///   as its length and the key and value at an index.
/// - `optional_map x => has_x, x_len, x_key_at, x_value_at;` lends an
///   `Option<BTreeMap<String, String>>` the same way.
///
/// An entry can carry doc comments, which go on the getter it is named for.
macro_rules! getters {
  () => {};
  ($(#[$attr:meta])* copy $field:ident: $type:ty; $($rest:tt)*) => {
    $(#[$attr])*
    pub(crate) fn $field(&self) -> $type {
      self.$field
    }

    getters! { $($rest)* }
  };
  ($(#[$attr:meta])* str $field:ident; $($rest:tt)*) => {
    getters! { $(#[$attr])* str $field => $field; $($rest)* }
  };
  ($(#[$attr:meta])* str $field:ident => $name:ident; $($rest:tt)*) => {
    $(#[$attr])*
    pub(crate) fn $name(&self) -> &str {
      &self.$field
    }

    getters! { $($rest)* }
  };
  ($(#[$attr:meta])* optional_str $field:ident; $($rest:tt)*) => {
    $(#[$attr])*
    pub(crate) fn $field(&self) -> Option<&str> {
      self.$field.as_deref()
    }

    getters! { $($rest)* }
  };
  ($(#[$attr:meta])* borrow $field:ident: $type:ty; $($rest:tt)*) => {
    $(#[$attr])*
    pub(crate) fn $field(&self) -> &$type {
      &self.$field
    }

    getters! { $($rest)* }
  };
  ($(#[$attr:meta])* present $field:ident: $type:ty => $has:ident; $($rest:tt)*) => {
    pub(crate) fn $has(&self) -> bool {
      self.$field.is_some()
    }

    #[doc = concat!("Swift calls this only when [`Self::", stringify!($has), "`] is true.")]
    $(#[$attr])*
    pub(crate) fn $field(&self) -> &$type {
      $crate::bridge::accessors::present(&self.$field)
    }

    getters! { $($rest)* }
  };
  ($(#[$attr:meta])* list $field:ident: $type:ty => $len:ident, $at:ident; $($rest:tt)*) => {
    pub(crate) fn $len(&self) -> usize {
      self.$field.len()
    }

    $(#[$attr])*
    pub(crate) fn $at(&self, index: usize) -> &$type {
      &self.$field[index]
    }

    getters! { $($rest)* }
  };
  (
    $(#[$attr:meta])* optional_list $field:ident: $type:ty => $has:ident, $len:ident, $at:ident;
    $($rest:tt)*
  ) => {
    pub(crate) fn $has(&self) -> bool {
      self.$field.is_some()
    }

    pub(crate) fn $len(&self) -> usize {
      self.$field.as_ref().map_or(0, Vec::len)
    }

    $(#[$attr])*
    pub(crate) fn $at(&self, index: usize) -> &$type {
      &$crate::bridge::accessors::present(&self.$field)[index]
    }

    getters! { $($rest)* }
  };
  ($(#[$attr:meta])* map $field:ident => $len:ident, $key_at:ident, $value_at:ident; $($rest:tt)*) => {
    pub(crate) fn $len(&self) -> usize {
      self.$field.len()
    }

    $(#[$attr])*
    pub(crate) fn $key_at(&self, index: usize) -> &str {
      $crate::bridge::accessors::entry_at(&self.$field, index).0
    }

    pub(crate) fn $value_at(&self, index: usize) -> &str {
      $crate::bridge::accessors::entry_at(&self.$field, index).1
    }

    getters! { $($rest)* }
  };
  (
    $(#[$attr:meta])* optional_map $field:ident => $has:ident, $len:ident, $key_at:ident, $value_at:ident;
    $($rest:tt)*
  ) => {
    pub(crate) fn $has(&self) -> bool {
      self.$field.is_some()
    }

    pub(crate) fn $len(&self) -> usize {
      self.$field.as_ref().map_or(0, ::std::collections::BTreeMap::len)
    }

    $(#[$attr])*
    pub(crate) fn $key_at(&self, index: usize) -> &str {
      $crate::bridge::accessors::entry_at($crate::bridge::accessors::present(&self.$field), index).0
    }

    pub(crate) fn $value_at(&self, index: usize) -> &str {
      $crate::bridge::accessors::entry_at($crate::bridge::accessors::present(&self.$field), index).1
    }

    getters! { $($rest)* }
  };
}

mod addresses;
mod archive;
mod cloud_hypervisor;
mod container;
mod ext4;
mod images;
mod os;
mod spec;

use crate::bridge::ffi;
use std::collections::BTreeMap;
use std::path::Path;

/// A path Swift reads while it builds a model, or `None` if it isn't UTF-8,
/// which Swift's `String` requires. Swift then throws, naming the path by
/// [`lossy_path`].
fn path(path: &Path) -> Option<String> {
  path.to_str().map(str::to_string)
}

/// A path as text, with any bytes that aren't UTF-8 replaced, for the error
/// Swift throws when [`path`] is `None`.
fn lossy_path(path: &Path) -> String {
  path.display().to_string()
}

/// What an optional field holds. Swift asks for it only after `has_`.
fn present<T>(value: &Option<T>) -> &T {
  value
    .as_ref()
    .expect("Swift asks for an optional field only after its `has_` getter")
}

/// The `index`th entry of a sorted map.
fn entry_at<T>(map: &BTreeMap<String, T>, index: usize) -> (&str, &T) {
  map
    .iter()
    .nth(index)
    .map(|(key, value)| (key.as_str(), value))
    .expect("Swift asks for an entry only below the length")
}

impl ffi::CzOutcome {
  /// What an outcome holds, or `None` for `Absent`.
  fn optional<T>(&self, read: impl FnOnce(&Self) -> T) -> Option<T> {
    self.is_some().then(|| read(self))
  }

  /// Each element of a held list.
  fn list<T>(&self, read: impl Fn(&Self) -> T) -> Vec<T> {
    (0..self.len()).map(|index| read(&self.at(index))).collect()
  }

  /// A held `[String: String]`.
  fn map(&self) -> BTreeMap<String, String> {
    self.map_keys().into_iter().zip(self.map_values()).collect()
  }

  /// A held `[String: T]`, each value read by `read`.
  fn map_of<T>(&self, read: impl Fn(&Self) -> T) -> BTreeMap<String, T> {
    self
      .entry_keys()
      .into_iter()
      .zip(self.entry_values().list(read))
      .collect()
  }

  /// What an outcome holds as text, or `None` for `Absent`.
  pub(crate) fn optional_text(&self) -> Option<String> {
    self.optional(Self::text)
  }

  /// The `Int32?` an outcome holds.
  pub(crate) fn optional_int32(&self) -> Option<i32> {
    self.optional(Self::int32)
  }

  /// A held `[String: Int32]`.
  pub(crate) fn int32_map(&self) -> BTreeMap<String, i32> {
    self.map_of(Self::int32)
  }
}
