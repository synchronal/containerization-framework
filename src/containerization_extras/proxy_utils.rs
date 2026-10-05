//! `ProxyUtils`.

use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::collections::BTreeMap;

/// `ProxyUtils.proxyFromEnvironment(scheme:host:env:)`: the proxy URL to
/// reach `host` through, or `None` to connect directly. With `env` as `None`,
/// Swift reads its default, the process environment.
pub fn proxy_from_environment(
  scheme: Option<&str>,
  host: &str,
  env: Option<&BTreeMap<String, String>>,
) -> Result<Option<String>, Error> {
  let (keys, values) = env
    .into_iter()
    .flatten()
    .map(|(key, value)| (key.clone(), value.clone()))
    .unzip();

  platform::outcome(
    ffi::cz_proxy_from_environment(scheme.map(str::to_string), host, env.is_some(), keys, values),
    format!("find the proxy for {host}"),
  )
  .map(|outcome| outcome.is_some().then(|| outcome.text()))
}
