#![cfg(feature = "integration")]

//! What Swift computes for Containerization's configuration types, without a
//! VM: command lines, mounts, DNS, hosts, exit statuses, resource limits,
//! capabilities, process configurations and signals.

use containerization_framework as cfw;
use std::collections::BTreeMap;
use std::time::Duration;
use std::time::SystemTime;

fn strings(values: &[&str]) -> Vec<String> {
  values.iter().map(|value| value.to_string()).collect()
}

#[test]
fn edits_a_kernel_command_line() {
  let mut command_line = cfw::containerization::vm::kernel::CommandLine::new(false, 0, Vec::new());

  command_line.add_debug().expect("debug");
  command_line.add_panic(3).expect("a panic level");
  command_line
    .set_agent_log_level(cfw::containerization::vm::kernel::LogLevel::Warning)
    .expect("a log level");

  assert_eq!(
    command_line.kernel_args,
    ["console=hvc0", "tsc=reliable", "panic=0", "debug", "panic=3"]
  );
  assert_eq!(command_line.init_args, ["--log-level", "warning"]);
}

#[test]
fn reads_a_kernels_arguments_from_its_command_line() {
  let mut kernel =
    cfw::containerization::vm::Kernel::new("/vmlinux", cfw::containerization::vm::SystemPlatform::LINUX_ARM);
  kernel.command_line.init_args = strings(&["--debug"]);

  assert_eq!(kernel.kernel_args(), ["console=hvc0", "tsc=reliable", "panic=0"]);
  assert_eq!(kernel.init_args(), ["--debug"]);
}

#[test]
fn copies_a_mounts_source() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let source = directory.path().join("disk.img");
  let copy = directory.path().join("copy.img");
  std::fs::write(&source, b"blocks").expect("a file");

  let mount = cfw::containerization::container::Mount::block("ext4", source.to_str().unwrap(), "/data", &["ro"], &[]);
  let cloned = mount.clone_to(copy.to_str().unwrap()).expect("a copy");

  assert_eq!(std::fs::read(&copy).expect("the copy"), b"blocks");
  assert_eq!(
    cloned,
    cfw::containerization::container::Mount {
      source: copy.to_str().unwrap().to_string(),
      ..mount
    }
  );
}

#[test]
fn says_which_mount_it_could_not_copy() {
  let mount = cfw::containerization::container::Mount::block("ext4", "/nowhere/disk.img", "/data", &[], &[]);

  let error = mount
    .clone_to("/nowhere/copy.img")
    .err()
    .expect("a missing source");

  assert!(error.to_string().contains("/nowhere/disk.img"), "{error}");
}

#[test]
fn hashes_a_mounts_source_through_symlinks() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let shared = directory.path().join("shared");
  let link = directory.path().join("link");
  std::fs::create_dir(&shared).expect("a directory");
  std::os::unix::fs::symlink(&shared, &link).expect("a symlink");

  let hash = |path: &std::path::Path| {
    cfw::containerization::container::Mount::share(path.to_str().unwrap(), "/shared", &[], &[])
      .tag_hash()
      .expect("a hash")
  };

  assert_eq!(hash(&shared).len(), 36);
  assert_eq!(hash(&shared), hash(&link));
}

#[test]
fn tells_block_mounts_from_others() {
  let shared = cfw::containerization::container::Mount::shared_mount("cache", "/cache", &["ro"]);

  assert_eq!(shared.r#type, "none");
  assert_eq!(shared.source, "cache");
  assert_eq!(
    shared.runtime_options,
    cfw::containerization::container::mount::RuntimeOptions::Shared
  );
  assert!(!shared.is_block());
  assert!(cfw::containerization::container::Mount::block("ext4", "/disk.img", "/data", &[], &[]).is_block());
}

#[test]
fn renders_and_validates_dns() {
  let dns = cfw::containerization::network::DNS {
    nameservers: strings(&["1.1.1.1", "2606:4700:4700::1111"]),
    domain: Some("example.com".into()),
    search_domains: strings(&["example.com", "internal"]),
    options: strings(&["ndots:2"]),
  };

  dns.validate().expect("valid nameservers");
  assert_eq!(
    dns.resolv_conf().expect("resolv.conf"),
    "nameserver 1.1.1.1\nnameserver 2606:4700:4700::1111\ndomain example.com\nsearch example.com internal\noptions ndots:2\n"
  );

  let invalid = cfw::containerization::network::DNS {
    nameservers: strings(&["dns.example.com"]),
    ..Default::default()
  };
  let error = invalid
    .validate()
    .err()
    .expect("a hostname as a nameserver");
  assert!(error.to_string().contains("dns.example.com"), "{error}");
}

#[test]
fn renders_hosts() {
  let hosts = cfw::containerization::network::Hosts {
    entries: vec![
      cfw::containerization::network::hosts::Entry::local_host_ipv4(Some("loopback")).expect("an entry"),
      cfw::containerization::network::hosts::Entry::new("192.168.64.2", &["db", "db.local"]),
    ],
    comment: Some("made by a test".into()),
  };

  assert_eq!(
    hosts.hosts_file().expect("a hosts file"),
    "# made by a test\n127.0.0.1 localhost # loopback \n192.168.64.2 db db.local\n"
  );
  assert_eq!(hosts.entries[1].rendered().expect("a line"), "192.168.64.2 db db.local");
}

#[test]
fn makes_the_default_hosts_entries() {
  let entry = cfw::containerization::network::hosts::Entry::ipv6_all_routers(Some("routers")).expect("an entry");
  assert_eq!(entry.comment.as_deref(), Some("routers"));

  let entries = [
    cfw::containerization::network::hosts::Entry::local_host_ipv4(None),
    cfw::containerization::network::hosts::Entry::local_host_ipv6(None),
    cfw::containerization::network::hosts::Entry::ipv6_local_net(None),
    cfw::containerization::network::hosts::Entry::ipv6_multicast_prefix(None),
    cfw::containerization::network::hosts::Entry::ipv6_all_nodes(None),
    cfw::containerization::network::hosts::Entry::ipv6_all_routers(None),
  ]
  .into_iter()
  .collect::<Result<Vec<_>, _>>()
  .expect("entries");

  assert_eq!(entries, cfw::containerization::network::Hosts::default().entries);
}

#[test]
fn dates_a_new_exit_status_now() {
  let before = SystemTime::now();
  let status = cfw::containerization::process::ExitStatus::new(3).expect("an exit status");

  assert_eq!(status.exit_code, 3);
  assert!(status.exited_at >= before - Duration::from_secs(1), "{status:?}");
  assert!(
    status.exited_at <= SystemTime::now() + Duration::from_secs(1),
    "{status:?}"
  );
}

#[test]
fn parses_rlimit_kinds_by_their_oci_names() {
  let kind = cfw::containerization::process::linux_rlimit::Kind::parse("RLIMIT_NOFILE").expect("a kind");

  assert_eq!(kind, cfw::containerization::process::linux_rlimit::Kind::OpenFiles);
  assert_eq!(kind.to_string(), "RLIMIT_NOFILE");

  let error = cfw::containerization::process::linux_rlimit::Kind::parse("nofile")
    .err()
    .expect("a name without its prefix");
  assert!(error.to_string().contains("nofile"), "{error}");
}

#[test]
fn makes_capability_sets() {
  let chown = cfw::containerization_os::linux::CapabilityName::Chown;
  let capabilities = cfw::containerization::process::LinuxCapabilities::with_capabilities(vec![chown]);

  assert_eq!(
    capabilities,
    cfw::containerization::process::LinuxCapabilities {
      bounding: vec![chown],
      effective: vec![chown],
      permitted: vec![chown],
      ..Default::default()
    }
  );

  let all = cfw::containerization::process::LinuxCapabilities::all_capabilities();
  assert_eq!(all.ambient, cfw::containerization_os::linux::CapabilityName::ALL_CASES);
}

#[test]
fn configures_a_process_from_an_image() {
  let config = cfw::containerization_oci::image::ImageConfig {
    user: Some("nobody".into()),
    env: Some(strings(&["PATH=/bin", "LANG=C"])),
    entrypoint: Some(strings(&["/bin/sh", "-c"])),
    cmd: Some(strings(&["echo hello"])),
    working_dir: Some("/srv".into()),
    ..Default::default()
  };

  let process =
    cfw::containerization::process::LinuxProcessConfiguration::from_image_config(&config).expect("a process");

  assert_eq!(process.arguments, ["/bin/sh", "-c", "echo hello"]);
  assert_eq!(process.environment_variables, ["PATH=/bin", "LANG=C"]);
  assert_eq!(process.working_directory, "/srv");
  assert_eq!(process.user.username, "nobody");
  assert_eq!(
    process.capabilities,
    cfw::containerization::process::LinuxCapabilities::default_oci_capabilities()
  );
}

#[test]
fn configures_a_process_from_an_empty_image() {
  let process = cfw::containerization::process::LinuxProcessConfiguration::from_image_config(&Default::default())
    .expect("a process");

  assert!(process.arguments.is_empty());
  assert!(process.environment_variables.is_empty(), "Swift drops the default PATH");
  assert_eq!(process.working_directory, "/");
  assert_eq!(process.user, cfw::containerization_oci::runtime::User::default());
}

#[test]
fn sets_a_processs_terminal() {
  let (parent, _child) = cfw::containerization_os::Terminal::create(None).expect("a pty");
  let mut process = cfw::containerization::process::LinuxProcessConfiguration {
    stderr: Some(2),
    ..cfw::containerization::process::LinuxProcessConfiguration::new(&["/bin/sh"])
  };

  process.set_terminal_io(&parent).expect("a terminal");

  assert!(process.terminal);
  assert_eq!(
    process.environment_variables,
    [
      format!(
        "PATH={}",
        cfw::containerization::process::LinuxProcessConfiguration::DEFAULT_PATH
      ),
      "TERM=xterm".to_string(),
    ]
  );
  assert_eq!(process.stdin, Some(parent.handle()));
  assert_eq!(process.stdout, Some(parent.handle()));
  assert_eq!(process.stderr, Some(2), "Swift leaves stderr alone");
  assert_eq!(process.arguments, ["/bin/sh"]);
}

#[test]
fn parses_signals_as_swift_does() {
  let parse = cfw::containerization::process::Signal::parse;

  assert_eq!(
    parse("SIGTERM").expect("a name"),
    cfw::containerization::process::Signal::TERM
  );
  assert_eq!(
    parse("kill").expect("a name"),
    cfw::containerization::process::Signal::KILL
  );
  assert_eq!(
    parse("10").expect("a number"),
    cfw::containerization::process::signal::linux::USR1
  );
  assert_eq!(
    parse("RTMIN+1").expect("a real-time signal"),
    cfw::containerization::process::signal::linux::rtmin(1)
  );

  let error = parse("SIGNOPE").err().expect("an unknown signal");
  assert!(error.to_string().contains("SIGNOPE"), "{error}");
  assert!(parse("65").is_err(), "a number Linux lacks");
}

#[test]
fn parses_signals_from_a_map() {
  let darwin = cfw::containerization::process::Signal::platform().expect("the host's signals");

  assert_eq!(
    cfw::containerization::process::Signal::parse_from("USR1", &darwin).expect("a name"),
    cfw::containerization::process::signal::darwin::USR1
  );
  assert!(cfw::containerization::process::Signal::parse_from("USR1", &BTreeMap::new()).is_err());
}

#[test]
fn lists_and_names_signals() {
  let linux = cfw::containerization::process::Signal::linux().expect("Linux's signals");
  assert_eq!(linux.get("STKFLT"), Some(&16));

  let platform = cfw::containerization::process::Signal::platform().expect("the host's signals");
  assert_eq!(platform.get("INFO"), Some(&29));

  assert_eq!(
    cfw::containerization::process::Signal::TERM
      .platform_name()
      .expect("a name")
      .as_deref(),
    Some("TERM")
  );
  assert_eq!(
    cfw::containerization::process::Signal::platform_name_of(1000).expect("no name"),
    None
  );
}

#[test]
fn converts_host_signals_to_linuxs() {
  assert_eq!(
    cfw::containerization::process::signal::darwin::USR1
      .linux_signal()
      .expect("a signal"),
    Some(cfw::containerization::process::signal::linux::USR1)
  );
  assert_eq!(
    cfw::containerization::process::signal::darwin::INFO
      .linux_signal()
      .expect("no signal"),
    None,
    "Linux has no INFO"
  );
  assert_eq!(
    cfw::containerization::process::Signal::from(15),
    cfw::containerization::process::Signal::TERM
  );
}

#[test]
fn renders_system_platforms_as_swift_does() {
  let platform = cfw::containerization::vm::SystemPlatform::LINUX_AMD;

  assert_eq!(platform.os.as_str(), "linux");
  assert_eq!(platform.architecture.as_str(), "amd64");
  assert_eq!(
    platform.oci_platform().expect("a platform"),
    cfw::containerization_oci::image::Platform::parse("linux/amd64").expect("a platform")
  );
}
