/// `Signal`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Signal {
  pub raw_value: i32,
}

impl Signal {
  /// `Signal(rawValue:)`.
  pub fn new(raw_value: i32) -> Self {
    Self { raw_value }
  }
}
