//! The OCI runtime spec's lifecycle hooks.

use crate::bridge::ffi;
use crate::containerization_oci::runtime;

impl ffi::CzOutcome {
  fn hook(&self) -> runtime::Hook {
    runtime::Hook {
      path: self.hook_path(),
      args: self.hook_args(),
      env: self.hook_env(),
      timeout: self.hook_timeout(),
    }
  }

  pub(super) fn hooks(&self) -> runtime::Hooks {
    let hooks = |kind| self.hooks_of(kind).list(Self::hook);

    runtime::Hooks {
      prestart: hooks(ffi::HookKind::Prestart),
      create_runtime: hooks(ffi::HookKind::CreateRuntime),
      create_container: hooks(ffi::HookKind::CreateContainer),
      start_container: hooks(ffi::HookKind::StartContainer),
      poststart: hooks(ffi::HookKind::Poststart),
      poststop: hooks(ffi::HookKind::Poststop),
    }
  }
}

impl runtime::Hook {
  pub(crate) fn path(&self) -> &str {
    &self.path
  }

  pub(crate) fn args_len(&self) -> usize {
    self.args.len()
  }

  pub(crate) fn args_at(&self, index: usize) -> &str {
    &self.args[index]
  }

  pub(crate) fn env_len(&self) -> usize {
    self.env.len()
  }

  pub(crate) fn env_at(&self, index: usize) -> &str {
    &self.env[index]
  }

  pub(crate) fn timeout(&self) -> Option<isize> {
    self.timeout
  }
}

impl runtime::Hooks {
  pub(crate) fn hooks_len(&self, kind: ffi::HookKind) -> usize {
    self.of(kind).len()
  }

  pub(crate) fn hooks_at(&self, kind: ffi::HookKind, index: usize) -> &runtime::Hook {
    &self.of(kind)[index]
  }

  fn of(&self, kind: ffi::HookKind) -> &[runtime::Hook] {
    match kind {
      ffi::HookKind::Prestart => &self.prestart,
      ffi::HookKind::CreateRuntime => &self.create_runtime,
      ffi::HookKind::CreateContainer => &self.create_container,
      ffi::HookKind::StartContainer => &self.start_container,
      ffi::HookKind::Poststart => &self.poststart,
      ffi::HookKind::Poststop => &self.poststop,
    }
  }
}
