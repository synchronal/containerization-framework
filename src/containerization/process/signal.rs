//! `Signal`, and its nested `Signal.Linux` and `Signal.Darwin`.

use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::collections::BTreeMap;

/// `Signal`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Signal {
  pub raw_value: i32,
}

impl Signal {
  /// `Signal.hup`.
  pub const HUP: Self = Self::new(1);
  /// `Signal.int`.
  pub const INT: Self = Self::new(2);
  /// `Signal.quit`.
  pub const QUIT: Self = Self::new(3);
  /// `Signal.kill`.
  pub const KILL: Self = Self::new(9);
  /// `Signal.term`.
  pub const TERM: Self = Self::new(15);
  /// `Signal.winch`.
  pub const WINCH: Self = Self::new(28);

  /// `Signal(rawValue:)`.
  pub const fn new(raw_value: i32) -> Self {
    Self { raw_value }
  }

  /// `Signal(_:)`, from a name such as `SIGKILL` or `kill`, or a number such
  /// as `9`, looked up in `Signal.linux`.
  pub fn parse(name: &str) -> Result<Self, Error> {
    Self::parsed(ffi::cz_signal_parse(name), name)
  }

  /// `Signal(_:from:)`, looked up in `map` instead.
  pub fn parse_from(name: &str, map: &BTreeMap<String, i32>) -> Result<Self, Error> {
    Self::parsed(
      ffi::cz_signal_parse_from(name, map.keys().cloned().collect(), map.values().copied().collect()),
      name,
    )
  }

  fn parsed(outcome: ffi::CzOutcome, name: &str) -> Result<Self, Error> {
    platform::outcome(outcome, format!("parse the signal {name}")).map(|outcome| Self::new(outcome.int32()))
  }

  /// `Signal.linux`: every Linux signal by name, without its `SIG` prefix,
  /// real-time signals included.
  pub fn linux() -> Result<BTreeMap<String, i32>, Error> {
    platform::outcome(ffi::cz_signal_linux(), "list Linux's signals").map(|outcome| outcome.int32_map())
  }

  /// `Signal.platform`: every signal of the host, by name.
  pub fn platform() -> Result<BTreeMap<String, i32>, Error> {
    platform::outcome(ffi::cz_signal_platform(), "list the host's signals").map(|outcome| outcome.int32_map())
  }

  /// `Signal.platformName()`: this signal's name on the host.
  pub fn platform_name(self) -> Result<Option<String>, Error> {
    Self::platform_name_of(self.raw_value)
  }

  /// `Signal.platformName(_:)`: a signal number's name on the host.
  pub fn platform_name_of(signal: i32) -> Result<Option<String>, Error> {
    platform::outcome(
      ffi::cz_signal_platform_name(signal),
      format!("name the signal {signal}"),
    )
    .map(|outcome| outcome.optional_text())
  }

  /// `Signal.linuxSignal()`: the Linux signal of the same name as this host
  /// signal.
  pub fn linux_signal(self) -> Result<Option<Self>, Error> {
    platform::outcome(
      ffi::cz_signal_linux_signal(self.raw_value),
      format!("find the Linux signal for {}", self.raw_value),
    )
    .map(|outcome| outcome.is_some().then(|| Self::new(outcome.int32())))
  }
}

/// `ExpressibleByIntegerLiteral`.
impl From<i32> for Signal {
  fn from(raw_value: i32) -> Self {
    Self::new(raw_value)
  }
}

macro_rules! signals {
  ($($name:ident = $value:literal),* $(,)?) => {
    $(#[doc = concat!("`SIG", stringify!($name), "`.")] pub const $name: super::Signal = super::Signal::new($value);)*

    /// Each signal, in the order Swift declares them.
    #[cfg(all(test, target_os = "macos"))]
    pub(crate) const ALL: &[super::Signal] = &[$($name),*];
  };
}

/// `Signal.Linux`.
pub mod linux {
  signals!(
    HUP = 1,
    INT = 2,
    QUIT = 3,
    ILL = 4,
    TRAP = 5,
    ABRT = 6,
    BUS = 7,
    FPE = 8,
    KILL = 9,
    USR1 = 10,
    SEGV = 11,
    USR2 = 12,
    PIPE = 13,
    ALRM = 14,
    TERM = 15,
    STKFLT = 16,
    CHLD = 17,
    CONT = 18,
    STOP = 19,
    TSTP = 20,
    TTIN = 21,
    TTOU = 22,
    URG = 23,
    XCPU = 24,
    XFSZ = 25,
    VTALRM = 26,
    PROF = 27,
    WINCH = 28,
    IO = 29,
    POLL = 29,
    PWR = 30,
    SYS = 31,
    RTMAX = 64,
  );

  /// `Signal.Linux.rtmin(offset:)`: the real-time signal `offset` past the
  /// first. Swift's `offset` defaults to 0.
  pub const fn rtmin(offset: i32) -> super::Signal {
    super::Signal::new(34 + offset)
  }
}

/// `Signal.Darwin`.
pub mod darwin {
  signals!(
    HUP = 1,
    INT = 2,
    QUIT = 3,
    ILL = 4,
    TRAP = 5,
    ABRT = 6,
    EMT = 7,
    FPE = 8,
    KILL = 9,
    BUS = 10,
    SEGV = 11,
    SYS = 12,
    PIPE = 13,
    ALRM = 14,
    TERM = 15,
    URG = 16,
    STOP = 17,
    TSTP = 18,
    CONT = 19,
    CHLD = 20,
    TTIN = 21,
    TTOU = 22,
    IO = 23,
    XCPU = 24,
    XFSZ = 25,
    VTALRM = 26,
    PROF = 27,
    WINCH = 28,
    INFO = 29,
    USR1 = 30,
    USR2 = 31,
  );
}
