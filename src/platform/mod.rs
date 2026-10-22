//! This module provides the Swift bridge on macOS and a stand-in for it on
//! every other platform, and it reads either one's failures as `Error`s. Every
//! wrapper calls through here, so each is written once for every platform.
//!
//! Only Swift calls the closures' `call` methods, so each allows dead code on
//! other platforms, which also covers what it reaches.

// The bridge's functions take Swift's arguments, however many there are.
#[cfg(not(target_os = "macos"))]
#[allow(clippy::too_many_arguments)]
pub(crate) mod unsupported;

#[cfg(target_os = "macos")]
pub(crate) use crate::bridge::ffi;

#[cfg(not(target_os = "macos"))]
pub(crate) use self::unsupported as ffi;

use crate::containerization::container;
use crate::containerization::process;
#[cfg(target_os = "macos")]
use crate::containerization_error;
use crate::containerization_extras;
use crate::containerization_os::file_descriptor_ops;
use crate::error::Error;
use std::any::Any;
use std::collections::BTreeMap;
use std::os::fd::BorrowedFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::time::Duration;
use std::time::SystemTime;

/// A panic in a Rust closure that Swift called. A panic can't unwind through
/// Swift, so it is kept and resumed on the thread that called Swift.
#[derive(Clone, Default)]
pub(crate) struct Panic(Arc<Mutex<Option<Box<dyn Any + Send>>>>);

impl Panic {
  /// What `body` returns, or `None` if it panicked. The first panic is kept.
  pub(crate) fn catch<R>(&self, body: impl FnOnce() -> R) -> Option<R> {
    std::panic::catch_unwind(AssertUnwindSafe(body))
      .map_err(|payload| {
        self
          .0
          .lock()
          .unwrap_or_else(PoisonError::into_inner)
          .get_or_insert(payload);
      })
      .ok()
  }

  pub(crate) fn caught(&self) -> bool {
    self
      .0
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .is_some()
  }

  /// Resumes the panic `catch` kept, if there is one.
  pub(crate) fn resume(&self) {
    if let Some(payload) = self.0.lock().unwrap_or_else(PoisonError::into_inner).take() {
      std::panic::resume_unwind(payload);
    }
  }
}

/// A `ProgressHandler?`, as it crosses to Swift. Once the handler panics, it
/// isn't called again.
pub(crate) struct Progress {
  pub(crate) handler: Option<containerization_extras::ProgressHandler>,
  pub(crate) panic: Panic,
}

impl Progress {
  #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
  pub(crate) fn is_some(&self) -> bool {
    self.handler.is_some()
  }

  /// One batch of events, as a kind and a value each.
  #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
  pub(crate) fn call(&self, kinds: Vec<ffi::ProgressKind>, values: Vec<i64>) {
    let Some(handler) = &self.handler else {
      return;
    };
    if self.panic.caught() {
      return;
    }
    let events: Vec<containerization_extras::ProgressEvent> = kinds
      .into_iter()
      .zip(values)
      .map(|(kind, value)| containerization_extras::ProgressEvent::from_kind(kind, value))
      .collect();

    self.panic.catch(|| handler(&events));
  }
}

/// What `call` returns when it hands Swift `handler`. A panic in the handler
/// is resumed once Swift has returned.
pub(crate) fn progressing<R>(
  handler: Option<containerization_extras::ProgressHandler>,
  call: impl FnOnce(Progress) -> R,
) -> R {
  let panic = Panic::default();
  let returned = call(Progress {
    handler,
    panic: panic.clone(),
  });

  panic.resume();
  returned
}

type Configuring<T> = Box<dyn FnOnce(&mut T) -> Result<(), Error> + Send>;

/// An `(inout T) throws -> Void` closure, as it crosses to Swift. Swift calls
/// it once, with the value it filled, and throws if it returns an error or
/// panics.
pub(crate) struct Configure<T> {
  configure: Mutex<Option<Configuring<T>>>,
  thrown: Thrown,
}

impl<T> Configure<T> {
  pub(crate) fn new(configure: impl FnOnce(&mut T) -> Result<(), Error> + Send + 'static) -> Self {
    Self {
      configure: Mutex::new(Some(Box::new(configure))),
      thrown: Thrown::default(),
    }
  }

  /// Runs the closure on the value Swift filled, the first time only, and
  /// says whether it threw. Its error or panic is kept for Rust.
  #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
  pub(crate) fn call(&self, value: &mut T) -> bool {
    let configure = self
      .configure
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .take();

    configure.is_some_and(|configure| self.thrown.threw(|| configure(value)))
  }
}

/// The manager's `(inout LinuxContainer.Configuration) throws -> Void`.
pub(crate) type ConfigureContainer = Configure<container::linux_container::Configuration>;

/// An `(inout LinuxProcessConfiguration) throws -> Void`.
pub(crate) type ConfigureProcess = Configure<process::LinuxProcessConfiguration>;

/// An `(inout LinuxPod.Configuration) throws -> Void`.
pub(crate) type ConfigurePod = Configure<container::linux_pod::Configuration>;

/// An `(inout LinuxPod.ContainerConfiguration) throws -> Void`.
pub(crate) type ConfigurePodContainer = Configure<container::linux_pod::ContainerConfiguration>;

/// What a throwing Swift call returned, when `call` hands it `configure` as
/// its closure, or what it threw as a failure of `action`, as
/// `Thrown::outcome` reads it.
pub(crate) fn configured<T>(
  configure: impl FnOnce(&mut T) -> Result<(), Error> + Send + 'static,
  call: impl FnOnce(Configure<T>) -> ffi::CzOutcome,
  action: impl Into<String>,
) -> Result<ffi::CzOutcome, Error> {
  let configure = Configure::new(configure);
  let thrown = configure.thrown.clone();
  let returned = call(configure);

  thrown.outcome(returned, action)
}

/// What a throwing Swift call returned, when `call` hands it `body` as its
/// `(FileDescriptor) throws -> Void` closure, or what it threw as a failure of
/// `action`, as `Thrown::outcome` reads it.
pub(crate) fn with_descriptor(
  body: impl FnOnce(BorrowedFd<'_>) -> Result<(), Error> + 'static,
  call: impl FnOnce(WithDescriptor) -> ffi::CzOutcome,
  action: impl Into<String>,
) -> Result<ffi::CzOutcome, Error> {
  let body = WithDescriptor::new(body);
  let thrown = body.thrown.clone();
  let returned = call(body);

  thrown.outcome(returned, action)
}

/// What a throwing Swift call returned, when `call` hands it `body` as
/// `FileDescriptorOps.enumerate(_:_:)`'s body, or what it threw as a failure
/// of `action`, as `Thrown::outcome` reads it.
pub(crate) fn enumerating(
  body: impl FnMut(&Path, file_descriptor_ops::EntryType, BorrowedFd<'_>) -> Result<(), Error> + 'static,
  call: impl FnOnce(Enumerate) -> ffi::CzOutcome,
  action: impl Into<String>,
) -> Result<ffi::CzOutcome, Error> {
  let body = Enumerate::new(body);
  let thrown = body.thrown.clone();
  let returned = call(body);

  thrown.outcome(returned, action)
}

/// The descriptor that a Swift call handed over, or what it threw as a
/// failure of `action`. Every bridge call whose descriptor reaches here gives
/// it up as it crosses, so the caller is its only owner.
pub(crate) fn owned_fd(outcome: ffi::CzOutcome, action: impl Into<String>) -> Result<OwnedFd, Error> {
  // SAFETY: Swift opened the descriptor for the caller, or its `FileHandle`
  // doesn't close it and forgets it once it crosses, so nothing else holds it.
  self::outcome(outcome, action).map(|outcome| unsafe { OwnedFd::from_raw_fd(outcome.int32()) })
}

/// Runs `call` with a closure that keeps the value Swift hands it, and
/// returns what `call` returned along with that value, if Swift handed one.
pub(crate) fn filled<T: Default + Send + 'static, R>(call: impl FnOnce(Configure<T>) -> R) -> (R, Option<T>) {
  let filled = Arc::new(Mutex::new(None));
  let slot = Arc::clone(&filled);
  let receive = Configure::new(move |value| {
    *slot.lock().unwrap_or_else(PoisonError::into_inner) = Some(std::mem::take(value));
    Ok(())
  });

  let returned = call(receive);

  (returned, filled.lock().unwrap_or_else(PoisonError::into_inner).take())
}

/// The process configuration that the throwing Swift call in `call` filled
/// and handed back, or what that call threw as a failure of `action`.
pub(crate) fn filled_process(
  call: impl FnOnce(ConfigureProcess) -> ffi::CzOutcome,
  action: impl Into<String>,
) -> Result<process::LinuxProcessConfiguration, Error> {
  let (returned, process) = filled(call);

  outcome(returned, action)?;
  Ok(process.expect("Swift hands back the configuration it fills"))
}

/// What a Rust closure that Swift calls threw, or how it panicked, kept for
/// the thread that called Swift.
#[derive(Clone, Default)]
pub(crate) struct Thrown {
  error: Arc<Mutex<Option<Error>>>,
  panic: Panic,
}

impl Thrown {
  /// Runs `body`, and says whether it threw. Its error or panic is kept.
  pub(crate) fn threw(&self, body: impl FnOnce() -> Result<(), Error>) -> bool {
    match self.panic.catch(body) {
      Some(Ok(())) => false,
      Some(Err(error)) => {
        *self.error.lock().unwrap_or_else(PoisonError::into_inner) = Some(error);
        true
      }
      None => true,
    }
  }

  /// What the throwing Swift call returned, or what it threw as a failure of
  /// `action`. If the closure threw, its own error is returned, as Swift
  /// rethrows it. If it panicked, the panic is resumed here.
  pub(crate) fn outcome(&self, returned: ffi::CzOutcome, action: impl Into<String>) -> Result<ffi::CzOutcome, Error> {
    self.panic.resume();
    if let Some(error) = self
      .error
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .take()
    {
      return Err(error);
    }

    outcome(returned, action)
  }
}

type DirectoryBody = Box<dyn FnOnce(BorrowedFd<'_>) -> Result<(), Error>>;

/// A `(FileDescriptor) throws -> Void` closure, as it crosses to Swift. Swift
/// calls it at most once, with a directory it keeps open for the call.
pub(crate) struct WithDescriptor {
  body: Mutex<Option<DirectoryBody>>,
  thrown: Thrown,
}

impl WithDescriptor {
  fn new(body: impl FnOnce(BorrowedFd<'_>) -> Result<(), Error> + 'static) -> Self {
    Self {
      body: Mutex::new(Some(Box::new(body))),
      thrown: Thrown::default(),
    }
  }

  /// Runs the closure on `descriptor`, the first time only, and says whether
  /// it threw.
  #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
  pub(crate) fn call(&self, descriptor: i32) -> bool {
    let body = self
      .body
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .take();

    // SAFETY: Swift keeps the descriptor open until the call returns.
    body.is_some_and(|body| {
      self
        .thrown
        .threw(|| body(unsafe { BorrowedFd::borrow_raw(descriptor) }))
    })
  }
}

type EnumerateBody = Box<dyn FnMut(&Path, file_descriptor_ops::EntryType, BorrowedFd<'_>) -> Result<(), Error>>;

/// `FileDescriptorOps.enumerate(_:_:)`'s body, as it crosses to Swift. Swift
/// calls it once per entry, with the entry's parent directory, which it keeps
/// open for the call.
pub(crate) struct Enumerate {
  body: Mutex<EnumerateBody>,
  thrown: Thrown,
}

impl Enumerate {
  fn new(
    body: impl FnMut(&Path, file_descriptor_ops::EntryType, BorrowedFd<'_>) -> Result<(), Error> + 'static,
  ) -> Self {
    Self {
      body: Mutex::new(Box::new(body)),
      thrown: Thrown::default(),
    }
  }

  /// Runs the closure on one entry, and says whether it threw.
  #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
  pub(crate) fn call(&self, path: String, entry_type: ffi::FdOpsEntryType, parent: i32) -> bool {
    let mut body = self.body.lock().unwrap_or_else(PoisonError::into_inner);

    // SAFETY: Swift keeps the descriptor open until the call returns.
    self.thrown.threw(|| {
      body(Path::new(&path), entry_type.into(), unsafe {
        BorrowedFd::borrow_raw(parent)
      })
    })
  }
}

/// What a throwing Swift call returned, or what it threw as a failure of
/// `action`.
pub(crate) fn outcome(outcome: ffi::CzOutcome, action: impl Into<String>) -> Result<ffi::CzOutcome, Error> {
  match outcome.error() {
    Some(message) => Err(error(&outcome, action, message)),
    None => Ok(outcome),
  }
}

#[cfg(target_os = "macos")]
fn error(outcome: &ffi::CzOutcome, action: impl Into<String>, message: String) -> Error {
  if let Some(error) = outcome.thrown_fd_ops_error() {
    return Error::FileDescriptorOps {
      action: action.into(),
      error,
    };
  }

  let code = outcome
    .error_code()
    .and_then(|code| containerization_error::Code::from_description(&code));
  Error::failed_with_code(action, message, code)
}

/// Elsewhere the bridge only ever fails, and nothing was attempted.
#[cfg(not(target_os = "macos"))]
fn error(_outcome: &ffi::CzOutcome, action: impl Into<String>, message: String) -> Error {
  Error::unavailable(action, message)
}

/// A path as it crosses to Swift. Swift's `String` holds only UTF-8, so a
/// path that isn't UTF-8 fails here and Swift is never asked to act on it.
pub(crate) fn path(path: &Path) -> Result<&str, Error> {
  path.to_str().ok_or_else(|| {
    Error::failed(
      format!("pass the path {} to Swift", path.display()),
      "the path isn't UTF-8, which Swift's String requires",
    )
  })
}

/// The `SystemTime` of a Swift `Date`, which crosses as seconds since 1970
/// and is negative before it. A `Date` can hold dates that `SystemTime` can't,
/// and those fail.
pub(crate) fn system_time(seconds: f64) -> Result<SystemTime, Error> {
  let duration = |seconds: f64| Duration::try_from_secs_f64(seconds).ok();
  let time = if seconds < 0.0 {
    duration(-seconds).and_then(|before| SystemTime::UNIX_EPOCH.checked_sub(before))
  } else {
    duration(seconds).and_then(|after| SystemTime::UNIX_EPOCH.checked_add(after))
  };

  time.ok_or_else(|| {
    Error::failed(
      "read a Swift Date as a SystemTime",
      format!("{seconds} seconds from 1970 is outside SystemTime's range"),
    )
  })
}

/// A struct Swift sends as its bytes, laid out as Swift's is.
///
/// # Safety
///
/// It is `repr(C)`, and any bytes of its size make a valid one.
pub(crate) unsafe trait Plain: Copy {}

/// A value from the bytes Swift sent for it. It panics if Swift's size isn't
/// Rust's.
pub(crate) fn from_bytes<T: Plain>(bytes: &[u8]) -> T {
  assert_eq!(
    bytes.len(),
    std::mem::size_of::<T>(),
    "Swift's {} is a different size from Rust's",
    std::any::type_name::<T>()
  );

  // SAFETY: `Plain` makes any bytes of its size valid, and the read doesn't
  // assume the bytes are aligned.
  unsafe { bytes.as_ptr().cast::<T>().read_unaligned() }
}

/// A `SystemTime` as a Swift `Date`'s seconds since 1970.
pub(crate) fn seconds(time: SystemTime) -> f64 {
  match time.duration_since(SystemTime::UNIX_EPOCH) {
    Ok(after) => after.as_secs_f64(),
    Err(before) => -before.duration().as_secs_f64(),
  }
}

/// A `[String: Data]` of extended attributes as it crosses to Swift: the
/// names, each value's length, and the values joined into one.
pub(crate) fn xattrs(xattrs: &BTreeMap<String, Vec<u8>>) -> (Vec<String>, Vec<u64>, Vec<u8>) {
  (
    xattrs.keys().cloned().collect(),
    xattrs.values().map(|value| value.len() as u64).collect(),
    xattrs.values().flatten().copied().collect(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn crosses_dates_on_either_side_of_1970() {
    for seconds in [-86_400.5, 0.0, 1_700_000_000.25] {
      assert_eq!(super::seconds(system_time(seconds).unwrap()), seconds);
    }
  }

  #[test]
  fn fails_dates_system_time_cant_hold() {
    for seconds in [9.3e18, -9.3e18, f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
      assert!(matches!(system_time(seconds), Err(Error::Failed { .. })), "{seconds}");
    }
  }

  #[test]
  fn crosses_a_utf8_path_as_it_is() {
    assert_eq!(super::path(Path::new("/tmp/file")).unwrap(), "/tmp/file");
  }

  #[test]
  fn fails_a_path_that_isnt_utf8() {
    use std::os::unix::ffi::OsStrExt;

    let path = Path::new(std::ffi::OsStr::from_bytes(b"/tmp/\xff"));

    assert!(matches!(super::path(path), Err(Error::Failed { .. })));
  }
}
