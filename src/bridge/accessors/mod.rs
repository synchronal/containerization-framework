//! Crate-private getters Swift reads value types through, and the setters it
//! fills them with.
//!
//! Each getter lends what the value holds, so Swift's copy is the only one.
//! Swift calls each by its `swift_name` in `bridge::ffi`, Rust's in camel case.
//! Shapes swift-bridge can't return become sets: optional nested value →
//! `has_x` + `x`; list → `x_len` + `x_at`; map → length + key and value at an
//! index; enum → kind + payload. Swift asks for an index only below the length.
//!
//! Swift never makes a Rust value: it fills one Rust hands it, clearing and
//! pushing what it holds, so it can hand back a configuration it seeded. (A
//! Rust function returning one to Swift gets a redundant cast in
//! swift-bridge's glue.)

mod addresses;
mod archive;
mod container;
mod ext4;
mod images;
mod os;
mod pod;
mod registry;
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
