#![cfg(feature = "integration")]

//! Configuring a container and its processes.
//!
//! What only a booted guest can show: that each setting reaches the container,
//! and that one the framework refuses fails the boot rather than being dropped.
//! The defaults these settings replace are asserted in `session.rs`.
//!
//! Each test is a VM, so one container checks every setting that can share it.

mod support;

use containerization_framework as cfw;
use support::Container;

/// One of Containerization's standard masked paths that this kernel has: a
/// path it lacks, like `/proc/kcore`, is never mounted over either way.
const STANDARD_MASKED_PATH: &str = "/proc/keys";

/// What a shell's failed write said, however it failed. Braced so the
/// redirection's own error is caught: a trailing `2>&1` would miss it.
fn write_error(path: &str) -> String {
  format!("{{ echo x > {path}; }} 2>&1 || true")
}

fn is_mounted_over(container: &Container, id: &str, path: &str) -> bool {
  container
    .sh(
      id,
      &format!("grep -q ' {path} ' /proc/self/mountinfo && echo yes || echo no"),
    )
    .trim()
    == "yes"
}

#[test]
fn names_resolves_and_tunes_the_container() {
  let container = Container::boot_with("cfw-test-config-names", |spec| {
    let configuration = &mut spec.configuration;

    configuration.hostname = Some("configured-host".into());
    configuration.sysctl = [("net.core.somaxconn".to_string(), "4096".to_string())].into();
    configuration.dns = Some(cfw::model::Dns {
      nameservers: vec![support::GATEWAY.into(), "1.1.1.1".into()],
      domain: Some("example.test".into()),
      search_domains: vec!["a.test".into(), "b.test".into()],
      options: vec!["ndots:2".into()],
    });
    configuration.hosts = Some(cfw::model::Hosts {
      entries: vec![
        cfw::model::HostsEntry::new("127.0.0.1", &["localhost"]),
        cfw::model::HostsEntry::new(support::GATEWAY, &["host.internal", "host"]),
      ],
      comment: Some("written by the suite".into()),
    });
  });

  assert_eq!(container.sh("hostname", "hostname").trim(), "configured-host");
  assert_eq!(
    container
      .sh("sysctl", "cat /proc/sys/net/core/somaxconn")
      .trim(),
    "4096"
  );
  assert_eq!(
    container.sh("resolv", "cat /etc/resolv.conf"),
    format!(
      "nameserver {}\nnameserver 1.1.1.1\ndomain example.test\nsearch a.test b.test\noptions ndots:2\n",
      support::GATEWAY
    )
  );
  assert_eq!(
    container.sh("hosts", "cat /etc/hosts"),
    format!(
      "# written by the suite\n127.0.0.1 localhost\n{} host.internal host\n",
      support::GATEWAY
    ),
    "the image's /etc/hosts should be replaced whole"
  );
}

#[test]
fn mounts_what_it_is_given() {
  let shared = tempfile::tempdir().expect("a temporary directory");
  std::fs::write(shared.path().join("greeting"), "shared from the host").expect("a file to share");
  let source = shared.path().display().to_string();

  let container = Container::boot_with("cfw-test-config-mounts", |spec| {
    let mounts = &mut spec.configuration.mounts;

    mounts.push(cfw::model::Mount::share(&source, "/shared", &["ro"]));
    mounts.push(cfw::model::Mount::any("tmpfs", "tmpfs", "/scratch", &["size=1m"]));
  });

  assert_eq!(container.sh("shared", "cat /shared/greeting"), "shared from the host");
  assert!(
    container
      .sh("shared-readonly", &write_error("/shared/probe"))
      .contains("Read-only file system"),
    "a share mounted `ro` should refuse a write"
  );
  assert_eq!(
    container
      .sh("tmpfs", "grep ' /scratch ' /proc/self/mountinfo | grep -o ' - tmpfs '")
      .trim(),
    "- tmpfs",
    "a mount the guest makes by itself should be made"
  );
  assert!(
    is_mounted_over(&container, "standard", "/dev/shm"),
    "extending the mounts should keep the standard ones"
  );
}

#[test]
fn runs_a_process_as_configured() {
  let container = Container::boot("cfw-test-config-process");

  let configuration = cfw::model::LinuxProcessConfiguration {
    environment_variables: vec!["GREETING=configured".into()],
    working_directory: Some("/tmp".into()),
    user: Some(cfw::model::User::named("nobody")),
    ..cfw::model::LinuxProcessConfiguration::new(&["/bin/sh", "-c", "pwd; id -un; echo $GREETING"])
  };

  assert_eq!(
    container.capture_with("process", &configuration),
    "/tmp\nnobody\nconfigured\n"
  );
}

#[test]
fn guards_paths_beyond_the_standard_ones() {
  let container = Container::boot_with("cfw-test-config-guarded", |spec| {
    let configuration = &mut spec.configuration;

    configuration.masked_paths.push(support::MARKER_PATH.into());
    configuration.readonly_paths.push("/etc".into());
  });

  assert!(
    is_mounted_over(&container, "masked-standard", STANDARD_MASKED_PATH),
    "adding a masked path should keep the standard ones"
  );
  assert_eq!(
    container.sh("masked", &format!("cat {}", support::MARKER_PATH)),
    "",
    "a masked file should read as empty"
  );

  assert!(
    container
      .sh("readonly-standard", &write_error("/proc/sys/kernel/hostname"))
      .contains("Read-only file system"),
    "adding a read-only path should keep the standard ones"
  );
  assert!(
    container
      .sh("readonly", &write_error("/etc/probe"))
      .contains("Read-only file system"),
    "a read-only path should refuse a write"
  );
  assert_eq!(
    container.exec("writable", &["/bin/touch", "/tmp/probe"]),
    0,
    "and only there"
  );
}

#[test]
fn drops_the_standard_guards_when_told() {
  let container = Container::boot_with("cfw-test-config-unguarded", |spec| {
    spec.configuration.masked_paths.clear();
    spec.configuration.readonly_paths.clear();
  });

  assert!(
    !is_mounted_over(&container, "unmasked", STANDARD_MASKED_PATH),
    "no masked paths should mask nothing"
  );

  // Still refused — the default capabilities lack CAP_SYS_ADMIN — but for want
  // of the capability, not because the path is read-only.
  let refused = container.sh("unprotected", &write_error("/proc/sys/kernel/hostname"));
  assert!(
    !refused.contains("Read-only file system"),
    "no read-only paths should protect nothing, but said: {refused}"
  );
}

#[test]
fn runs_its_first_process_under_an_init() {
  let container = Container::boot_with("cfw-test-config-init", |spec| {
    spec.configuration.use_init = true;
  });

  let pid_one = container.sh("pid-one", "tr '\\0' ' ' < /proc/1/cmdline");
  assert!(
    pid_one.starts_with("/.cz-init -- /bin/sh "),
    "the keepalive should run under the init, but PID 1 is {pid_one:?}"
  );
}

#[test]
fn writes_its_boot_log_where_told() {
  let logs = tempfile::tempdir().expect("a temporary directory");
  let log = logs.path().join("boot.log");

  let container = Container::boot_with("cfw-test-config-boot-log", |spec| {
    spec.configuration.boot_log = Some(cfw::model::BootLog::file(&log));
  });

  assert!(
    std::fs::metadata(&log).is_ok_and(|metadata| metadata.len() > 0),
    "the guest's console should be written to {}",
    log.display()
  );
  assert!(
    !container
      .session()
      .store()
      .container_dir(container.name())
      .join("bootlog.log")
      .exists(),
    "and not to the container's directory as well"
  );
}

/// Only an M3 or later can: elsewhere the boot fails, which is the behavior
/// asked for, and there is nothing further to check.
#[test]
fn boots_with_nested_virtualization_where_the_host_has_it() {
  let booted = Container::try_boot_with("cfw-test-config-nested", |spec| {
    spec.configuration.virtualization = true;
  });

  match booted {
    Ok(container) => assert_eq!(container.exec("true", &["/bin/true"]), 0),
    Err(error)
      if error
        .to_string()
        .contains("nested virtualization is not supported") =>
    {
      eprintln!("this host has no nested virtualization: {error}");
    }
    Err(error) => panic!("a nested-virtualization boot should succeed or say it is unsupported: {error}"),
  }
}

#[test]
fn refuses_seccomp_without_an_oci_runtime() {
  let refused = Container::try_boot_with("cfw-test-config-seccomp", |spec| {
    spec.configuration.seccomp_profile = cfw::model::SeccompProfile::Default;
  });

  let error = refused
    .err()
    .expect("a filter nothing installs should fail the boot, not run unfiltered");
  assert!(error.to_string().contains("seccomp"), "{error}");
}

#[test]
fn refuses_a_seccomp_profile_that_is_not_one() {
  let refused = Container::try_boot_with("cfw-test-config-seccomp-profile", |spec| {
    spec.configuration.oci_runtime_path = Some("/sbin/runc".into());
    spec.configuration.seccomp_profile = cfw::model::SeccompProfile::Profile("not json".into());
  });

  let error = refused
    .err()
    .expect("a malformed profile should fail the boot");
  assert!(error.to_string().contains("seccomp"), "{error}");
}

/// The stock init image carries no `runc`. That one runs when present is
/// beyond what this suite's store can show.
#[test]
fn refuses_an_oci_runtime_the_init_image_lacks() {
  let refused = Container::try_boot_with("cfw-test-config-oci-runtime", |spec| {
    spec.configuration.oci_runtime_path = Some("/sbin/no-such-runtime".into());
  });

  let error = refused
    .err()
    .expect("a runtime that is not there should fail the boot");
  // The guest names no cause, but this is where it fails: at the start, not
  // as a configuration refused beforehand.
  assert!(error.to_string().contains("failed to start process"), "{error}");
}

#[test]
fn refuses_a_nameserver_that_is_not_an_address() {
  let refused = Container::try_boot_with("cfw-test-config-nameserver", |spec| {
    spec.configuration.dns = Some(cfw::model::Dns {
      nameservers: vec!["dns.example.test".into()],
      ..Default::default()
    });
  });

  let error = refused
    .err()
    .expect("a hostname nameserver should fail the boot");
  assert!(error.to_string().contains("dns.example.test"), "{error}");
}
