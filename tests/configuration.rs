#![cfg(feature = "integration")]

//! Configuring a container and its processes.
//!
//! What only a booted guest can show: that each setting reaches the container,
//! and that one Containerization refuses fails the container rather than being
//! dropped.
//!
//! Each test is a VM, so one container checks every setting that can share it.

mod support;

use containerization_framework as cfw;
use support::container::Container;

/// One of Containerization's standard masked paths that this kernel has: a
/// path it lacks, like `/proc/kcore`, is never mounted over either way.
const STANDARD_MASKED_PATH: &str = "/proc/keys";

/// A file Alpine ships, to mask.
const IMAGE_FILE: &str = "/etc/alpine-release";

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
  let container = Container::boot_with("cfw-test-config-names", |configuration| {
    configuration.hostname = Some("configured-host".into());
    configuration.sysctl = [("net.core.somaxconn".to_string(), "4096".to_string())].into();
    configuration.dns = Some(cfw::containerization::network::Dns {
      nameservers: vec![support::network::GATEWAY.into(), "1.1.1.1".into()],
      domain: Some("example.test".into()),
      search_domains: vec!["a.test".into(), "b.test".into()],
      options: vec!["ndots:2".into()],
    });
    configuration.hosts = Some(cfw::containerization::network::Hosts {
      entries: vec![
        cfw::containerization::network::hosts::Entry::new("127.0.0.1", &["localhost"]),
        cfw::containerization::network::hosts::Entry::new(support::network::GATEWAY, &["host.internal", "host"]),
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
      support::network::GATEWAY
    )
  );
  assert_eq!(
    container.sh("hosts", "cat /etc/hosts"),
    format!(
      "# written by the suite\n127.0.0.1 localhost\n{} host.internal host\n",
      support::network::GATEWAY
    ),
    "the image's /etc/hosts should be replaced whole"
  );
}

#[test]
fn addresses_its_interface_as_configured() {
  let mac_address =
    cfw::containerization_extras::address::MACAddress::parse("02:42:ac:11:00:02").expect("Swift parses a MAC address");
  let ipv6_address =
    cfw::containerization_extras::address::CIDRv6::parse("fd00:cf::2/64").expect("Swift parses an IPv6 CIDR block");

  let container = Container::boot_with("cfw-test-config-interface", move |configuration| {
    let cfw::containerization::network::Interface::Nat(interface) = &mut configuration.interfaces[0] else {
      panic!("the suite gives each container a NAT interface");
    };

    interface.mac_address = Some(mac_address);
    interface.ipv6_address = Some(ipv6_address);
  });

  assert_eq!(
    container
      .sh("mac", "cat /sys/class/net/eth0/address")
      .trim(),
    "02:42:ac:11:00:02"
  );
  assert!(
    container
      .sh("ipv6", "ip -6 addr show dev eth0")
      .contains("inet6 fd00:cf::2/64"),
    "the interface should carry its IPv6 address"
  );
}

#[test]
fn mounts_what_it_is_given() {
  let shared = tempfile::tempdir().expect("a temporary directory");
  std::fs::write(shared.path().join("greeting"), "shared from the host").expect("a file to share");
  let source = shared.path().display().to_string();

  let container = Container::boot_with("cfw-test-config-mounts", move |configuration| {
    let mounts = &mut configuration.mounts;

    mounts.push(cfw::containerization::container::Mount::share(
      &source,
      "/shared",
      &["ro"],
      &[],
    ));
    mounts.push(cfw::containerization::container::Mount::any(
      "tmpfs",
      "tmpfs",
      "/scratch",
      &["size=1m"],
      &[],
    ));
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

  let mut configuration =
    cfw::containerization::process::LinuxProcessConfiguration::new(&["/bin/sh", "-c", "pwd; id -un; echo $GREETING"]);
  configuration
    .environment_variables
    .push("GREETING=configured".into());
  configuration.working_directory = "/tmp".into();
  configuration.user = cfw::containerization_oci::runtime::User {
    username: "nobody".into(),
    ..Default::default()
  };

  assert_eq!(
    container.capture_with("process", configuration),
    "/tmp\nnobody\nconfigured\n"
  );
}

/// What the closure sees comes back over a channel: it runs on Swift's thread,
/// where a failed assertion couldn't unwind.
#[test]
fn hands_the_closure_what_the_manager_seeded() {
  let (send, seen) = std::sync::mpsc::channel();

  let _container = Container::boot_with("cfw-test-config-seeded", move |configuration| {
    let _ = send.send(configuration.clone());
  });
  let seeded = seen.recv().expect("the closure should have run");

  assert!(
    seeded
      .process
      .environment_variables
      .iter()
      .any(|variable| variable.starts_with("PATH=")),
    "the image's environment should be seeded: {:?}",
    seeded.process.environment_variables
  );
  assert!(
    matches!(seeded.boot_log, Some(cfw::containerization::vm::BootLog::File { .. })),
    "the manager's boot log should be seeded: {:?}",
    seeded.boot_log
  );
}

#[test]
fn guards_paths_beyond_the_standard_ones() {
  let container = Container::boot_with("cfw-test-config-guarded", |configuration| {
    configuration.masked_paths.push(IMAGE_FILE.into());
    configuration.readonly_paths.push("/etc".into());
  });

  assert!(
    is_mounted_over(&container, "masked-standard", STANDARD_MASKED_PATH),
    "adding a masked path should keep the standard ones"
  );
  assert_eq!(
    container.sh("masked", &format!("cat {IMAGE_FILE}")),
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
  let container = Container::boot_with("cfw-test-config-unguarded", |configuration| {
    configuration.masked_paths.clear();
    configuration.readonly_paths.clear();
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
  let container = Container::boot_with("cfw-test-config-init", |configuration| {
    configuration.use_init = true;
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
  let configured = log.clone();

  let container = Container::boot_with("cfw-test-config-boot-log", move |configuration| {
    configuration.boot_log = Some(cfw::containerization::vm::BootLog::file(configured));
  });

  assert!(
    std::fs::metadata(&log).is_ok_and(|metadata| metadata.len() > 0),
    "the guest's console should be written to {}",
    log.display()
  );
  assert!(
    !container.directory().join("bootlog.log").exists(),
    "and not to the container's directory as well"
  );
}

/// Only an M3 or later can: elsewhere the container fails, which is the
/// behavior asked for, and there is nothing further to check.
#[test]
fn boots_with_nested_virtualization_where_the_host_has_it() {
  let booted = Container::try_boot_with("cfw-test-config-nested", |configuration| {
    configuration.virtualization = true;
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
    Err(error) => panic!("a nested-virtualization container should start or say it is unsupported: {error}"),
  }
}

#[test]
fn refuses_seccomp_without_an_oci_runtime() {
  let refused = Container::try_boot_with("cfw-test-config-seccomp", |configuration| {
    configuration.seccomp_profile = cfw::containerization::container::linux_container::SeccompProfile::Default;
  });

  let error = refused
    .err()
    .expect("a filter nothing installs should fail the container, not run unfiltered");
  assert!(error.to_string().contains("seccomp"), "{error}");
}

/// The profile crosses to Swift and back whole, and the container gets as far
/// as starting `runc`.
#[test]
fn carries_a_seccomp_profile_of_its_own() {
  let profile = cfw::containerization::container::linux_container::SeccompProfile::Profile(
    cfw::containerization_oci::runtime::LinuxSeccomp::default_profile(
      None,
      cfw::containerization_oci::runtime::Arch::current_verified().expect("a seccomp architecture"),
    )
    .expect("the default profile"),
  );

  // Never created, so the rootfs needn't exist.
  let carried = cfw::containerization::container::LinuxContainer::new(
    "cfw-test-config-seccomp-profile-carried",
    cfw::containerization::container::Mount::block("ext4", "/rootfs.ext4", "/", &[], &[]),
    None,
    &support::store::vmm(),
    support::vm(),
    cfw::containerization::container::linux_container::Configuration {
      oci_runtime_path: Some("/sbin/runc".into()),
      seccomp_profile: profile.clone(),
      ..Default::default()
    },
  )
  .expect("a container should be made from a configuration");
  assert_eq!(carried.config().seccomp_profile, profile);

  let refused = Container::try_boot_with("cfw-test-config-seccomp-profile", |configuration| {
    configuration.oci_runtime_path = Some("/sbin/runc".into());
    configuration.seccomp_profile = profile;
  });

  let error = refused.err().expect("the stock init image has no runc");
  assert!(error.to_string().contains("failed to start process"), "{error}");
}

/// The stock init image carries no `runc`. That one runs when present is
/// beyond what this suite's store can show.
#[test]
fn refuses_an_oci_runtime_the_init_image_lacks() {
  let refused = Container::try_boot_with("cfw-test-config-oci-runtime", |configuration| {
    configuration.oci_runtime_path = Some("/sbin/no-such-runtime".into());
  });

  let error = refused
    .err()
    .expect("a runtime that is not there should fail the container");
  // The guest names no cause, but this is where it fails: at the start, not
  // as a configuration refused beforehand.
  assert!(error.to_string().contains("failed to start process"), "{error}");
}
