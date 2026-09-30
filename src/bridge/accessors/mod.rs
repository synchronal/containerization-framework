//! Crate-private getters Swift reads the model through.
//!
//! Each lends what the model holds, so Swift's copy is the only one. Shapes
//! swift-bridge can't return become sets: optional nested value → `has_x` +
//! `x`; list → `x_len` + `x_at`; map → length + key and value at an index;
//! enum → mode + payload. Swift asks for an index only below the length.
//!
//! Split as `model` is.

mod build;
mod container;

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
