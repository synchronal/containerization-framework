use crate::platform::ffi;

/// `VirtiofsLayout`, which says how a VM exposes virtiofs devices to its
/// guest.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VirtiofsLayout {
  /// One device, tagged `virtiofs`, carries each share as a subdirectory.
  Unified,
  /// One device per share.
  PerTag,
}

same_variants!(VirtiofsLayout, ffi::VirtiofsLayoutKind { Unified, PerTag });
