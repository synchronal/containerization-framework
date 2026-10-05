use std::time::SystemTime;

/// `ExitStatus`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExitStatus {
  pub exit_code: i32,
  pub exited_at: SystemTime,
}
