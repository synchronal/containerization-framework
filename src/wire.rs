//! Turning the model into what the bridge carries.
//!
//! Lists cross as newline-separated strings, mounts and sockets as
//! tab-separated fields within them. Both separators are assumed absent from
//! the values: a mount is two paths and a flag, and an argument or environment
//! variable holding a newline would be read as two elements on the far side.
//!
//! A build plan crosses as JSON instead, since a step's script may contain
//! either separator, and so does a container's configuration, which nests.

use crate::model;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

/// One element per line. Swift reads `""` as no elements, not one empty one.
pub fn lines(values: &[String]) -> String {
  values.join("\n")
}

/// The guest's working directory; `/` when none is declared.
pub fn working_directory(workdir: Option<&Path>) -> String {
  workdir.map_or_else(|| "/".to_string(), |path| path.display().to_string())
}

/// `source\tdestination\tro|rw`, in declared order (matters for nested mounts).
pub fn mounts(mounts: &[model::Mount]) -> Vec<String> {
  mounts
    .iter()
    .map(|mount| {
      format!(
        "{}\t{}\t{}",
        mount.source.display(),
        mount.target.display(),
        if mount.readonly { "ro" } else { "rw" }
      )
    })
    .collect()
}

/// `source\tdestination\tmode\tdirection`, the mode in octal.
pub fn sockets(sockets: &[model::SocketRelay]) -> Vec<String> {
  sockets
    .iter()
    .map(|socket| {
      format!(
        "{}\t{}\t{:o}\t{}",
        socket.source.display(),
        socket.target.display(),
        socket.mode,
        match socket.direction {
          model::Direction::IntoGuest => "into",
          model::Direction::OutOfGuest => "outof",
        }
      )
    })
    .collect()
}

/// What a container is configured with beyond its process, mounts and
/// network, as the Swift side decodes it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration<'a> {
  hostname: Option<&'a str>,
  sysctl: &'a BTreeMap<String, String>,
  dns: Option<Dns<'a>>,
  hosts: Option<Vec<HostsEntry<'a>>>,
  masked_paths: GuardedPaths<'a>,
  readonly_paths: GuardedPaths<'a>,
  use_init: bool,
  nested_virtualization: bool,
  boot_log: Option<String>,
  oci_runtime_path: Option<&'a str>,
  seccomp: Seccomp<'a>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Dns<'a> {
  nameservers: &'a [String],
  domain: Option<&'a str>,
  search_domains: &'a [String],
  options: &'a [String],
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HostsEntry<'a> {
  ip_address: &'a str,
  hostnames: &'a [String],
  comment: Option<&'a str>,
}

/// `{"mode": "default"}`, or `{"mode": "defaultAnd" | "only", "paths": [...]}`.
#[derive(Debug, Serialize)]
#[serde(tag = "mode", content = "paths", rename_all = "camelCase")]
enum GuardedPaths<'a> {
  Default,
  DefaultAnd(&'a [String]),
  Only(&'a [String]),
}

/// `{"mode": "unconfined" | "default"}`, or `{"mode": "profile", "profile": "<json>"}`.
#[derive(Debug, Serialize)]
#[serde(tag = "mode", content = "profile", rename_all = "camelCase")]
enum Seccomp<'a> {
  Unconfined,
  Default,
  Profile(&'a str),
}

impl<'a> From<&'a model::GuardedPaths> for GuardedPaths<'a> {
  fn from(paths: &'a model::GuardedPaths) -> Self {
    match paths {
      model::GuardedPaths::Default => Self::Default,
      model::GuardedPaths::DefaultAnd(paths) => Self::DefaultAnd(paths),
      model::GuardedPaths::Only(paths) => Self::Only(paths),
    }
  }
}

impl<'a> From<&'a model::BootSpec> for Configuration<'a> {
  fn from(spec: &'a model::BootSpec) -> Self {
    Self {
      hostname: spec.hostname.as_deref(),
      sysctl: &spec.sysctl,
      dns: spec.dns.as_ref().map(|dns| Dns {
        nameservers: &dns.nameservers,
        domain: dns.domain.as_deref(),
        search_domains: &dns.search_domains,
        options: &dns.options,
      }),
      hosts: spec.hosts.as_ref().map(|entries| {
        entries
          .iter()
          .map(|entry| HostsEntry {
            ip_address: &entry.ip_address,
            hostnames: &entry.hostnames,
            comment: entry.comment.as_deref(),
          })
          .collect()
      }),
      masked_paths: (&spec.masked_paths).into(),
      readonly_paths: (&spec.readonly_paths).into(),
      use_init: spec.use_init,
      nested_virtualization: spec.nested_virtualization,
      boot_log: spec
        .boot_log
        .as_ref()
        .map(|path| path.display().to_string()),
      oci_runtime_path: spec.oci_runtime.as_deref(),
      seccomp: match &spec.seccomp {
        model::Seccomp::Unconfined => Seccomp::Unconfined,
        model::Seccomp::Default => Seccomp::Default,
        model::Seccomp::Profile(profile) => Seccomp::Profile(profile),
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;
  use std::path::PathBuf;

  #[test]
  fn writes_mounts_in_declared_order_with_their_mode() {
    let declared = [
      model::Mount {
        readonly: false,
        source: PathBuf::from("/Users/user/workspace"),
        target: PathBuf::from("/workspace"),
      },
      model::Mount {
        readonly: true,
        source: PathBuf::from("/Users/user/.cargo/registry"),
        target: PathBuf::from("/Users/user/.cargo/registry"),
      },
    ];

    assert_eq!(
      lines(&mounts(&declared)),
      "/Users/user/workspace\t/workspace\trw\n/Users/user/.cargo/registry\t/Users/user/.cargo/registry\tro"
    );
  }

  #[test]
  fn writes_no_mounts_as_nothing_at_all() {
    assert_eq!(lines(&mounts(&[])), "");
    assert_eq!(lines(&[]), "");
    assert_eq!(lines(&sockets(&[])), "");
  }

  #[test]
  fn works_in_the_root_unless_told_otherwise() {
    assert_eq!(working_directory(None), "/");
    assert_eq!(working_directory(Some(Path::new("/workspace"))), "/workspace");
  }

  #[test]
  fn writes_a_relayed_socket_with_its_mode_and_direction() {
    let declared = [model::SocketRelay::into_guest(
      "/state/ports/7001.sock",
      "/run/session/ports/7001.sock",
    )];

    assert_eq!(
      sockets(&declared),
      ["/state/ports/7001.sock\t/run/session/ports/7001.sock\t666\tinto"]
    );
  }

  #[test]
  fn writes_the_direction_a_socket_is_reached_from() {
    let out = model::SocketRelay {
      direction: model::Direction::OutOfGuest,
      mode: 0o600,
      ..model::SocketRelay::into_guest("/host.sock", "/guest.sock")
    };

    assert_eq!(sockets(&[out]), ["/host.sock\t/guest.sock\t600\toutof"]);
  }

  fn spec() -> model::BootSpec {
    model::BootSpec::new(
      "container",
      "docker.io/library/alpine:3",
      model::Resources {
        cpus: 1,
        memory_in_bytes: 1 << 30,
      },
      model::Network {
        ipv4_address: "192.168.64.7/24".into(),
        ipv4_gateway: "192.168.64.1".into(),
      },
    )
  }

  fn configuration(spec: &model::BootSpec) -> serde_json::Value {
    serde_json::to_value(Configuration::from(spec)).expect("a configuration should serialize")
  }

  /// Swift decodes by property name; a renamed field fails only at runtime.
  #[test]
  fn leaves_an_unconfigured_container_to_the_framework() {
    assert_eq!(
      configuration(&spec()),
      json!({
        "hostname": null,
        "sysctl": {},
        "dns": null,
        "hosts": null,
        "maskedPaths": {"mode": "default"},
        "readonlyPaths": {"mode": "default"},
        "useInit": false,
        "nestedVirtualization": false,
        "bootLog": null,
        "ociRuntimePath": null,
        "seccomp": {"mode": "unconfined"},
      })
    );
  }

  #[test]
  fn writes_every_setting_a_caller_makes() {
    let spec = model::BootSpec {
      hostname: Some("box".into()),
      sysctl: [("net.core.somaxconn".to_string(), "4096".to_string())].into(),
      dns: Some(model::Dns {
        nameservers: vec!["1.1.1.1".into()],
        domain: Some("example.test".into()),
        search_domains: vec!["a.test".into(), "b.test".into()],
        options: vec!["ndots:2".into()],
      }),
      hosts: Some(vec![model::HostsEntry {
        comment: Some("the host".into()),
        ..model::HostsEntry::new("192.168.64.1", &["host.internal"])
      }]),
      masked_paths: model::GuardedPaths::DefaultAnd(vec!["/secret".into()]),
      readonly_paths: model::GuardedPaths::Only(vec![]),
      use_init: true,
      nested_virtualization: true,
      boot_log: Some(PathBuf::from("/logs/boot.log")),
      oci_runtime: Some("/sbin/runc".into()),
      seccomp: model::Seccomp::Profile(r#"{"defaultAction":"SCMP_ACT_ALLOW"}"#.into()),
      ..spec()
    };

    assert_eq!(
      configuration(&spec),
      json!({
        "hostname": "box",
        "sysctl": {"net.core.somaxconn": "4096"},
        "dns": {
          "nameservers": ["1.1.1.1"],
          "domain": "example.test",
          "searchDomains": ["a.test", "b.test"],
          "options": ["ndots:2"],
        },
        "hosts": [{"ipAddress": "192.168.64.1", "hostnames": ["host.internal"], "comment": "the host"}],
        "maskedPaths": {"mode": "defaultAnd", "paths": ["/secret"]},
        "readonlyPaths": {"mode": "only", "paths": []},
        "useInit": true,
        "nestedVirtualization": true,
        "bootLog": "/logs/boot.log",
        "ociRuntimePath": "/sbin/runc",
        "seccomp": {"mode": "profile", "profile": r#"{"defaultAction":"SCMP_ACT_ALLOW"}"#},
      })
    );
  }

  #[test]
  fn writes_the_default_seccomp_profile_by_name() {
    let spec = model::BootSpec {
      seccomp: model::Seccomp::Default,
      ..spec()
    };

    assert_eq!(configuration(&spec)["seccomp"], json!({"mode": "default"}));
  }
}
