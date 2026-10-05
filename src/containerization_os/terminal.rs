//! `Terminal`'s nested types.

/// `Terminal.Size`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Size {
  pub width: u16,
  pub height: u16,
}
