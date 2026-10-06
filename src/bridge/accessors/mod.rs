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
mod registry;
mod spec;

use std::collections::BTreeMap;
use std::path::Path;

fn path(path: &Path) -> String {
  path.display().to_string()
}

/// The `index`th entry of a sorted map.
fn entry_at(map: &BTreeMap<String, String>, index: usize) -> (&str, &str) {
  map
    .iter()
    .nth(index)
    .map(|(key, value)| (key.as_str(), value.as_str()))
    .expect("Swift asks for an entry only below the length")
}
