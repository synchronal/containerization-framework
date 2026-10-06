#![cfg(feature = "integration")]

//! `ContainerizationOCI`'s runtime spec, each value crossing to Swift and
//! back.

use cfw::containerization_oci as oci;
use containerization_framework as cfw;
use std::collections::BTreeMap;

fn strings(values: &[&str]) -> Vec<String> {
  values.iter().map(|value| value.to_string()).collect()
}

fn map(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
  entries
    .iter()
    .map(|(key, value)| (key.to_string(), value.to_string()))
    .collect()
}

/// A `Spec` with every field set, and none at its default.
fn full_spec() -> oci::Spec {
  let hook = |path: &str| oci::Hook::new(path, strings(&[path, "--flag"]), strings(&["TOKEN=secret"]), Some(5));
  let mapping = oci::LinuxIDMapping::new(0, 100_000, 65_536);
  let throttle = |rate| oci::LinuxThrottleDevice::new(8, 0, rate);

  oci::Spec {
    version: "1.0.2-dev".to_string(),
    hooks: Some(oci::Hooks::new(
      vec![hook("/prestart")],
      vec![hook("/create-runtime")],
      vec![hook("/create-container")],
      vec![hook("/start-container")],
      vec![hook("/poststart")],
      vec![hook("/poststop")],
    )),
    process: Some(oci::Process {
      cwd: "/work".to_string(),
      env: strings(&["PATH=/bin", "HOME=/root"]),
      console_size: Some(oci::Box::new(24, 80)),
      selinux_label: "system_u:system_r:container_t".to_string(),
      no_new_privileges: true,
      command_line: "sh -c true".to_string(),
      oom_score_adj: Some(-500),
      capabilities: Some(oci::LinuxCapabilities {
        bounding: Some(strings(&["CAP_CHOWN", "CAP_KILL"])),
        effective: Some(strings(&["CAP_CHOWN"])),
        inheritable: Some(Vec::new()),
        permitted: Some(strings(&["CAP_KILL"])),
        ambient: None,
      }),
      apparmor_profile: "docker-default".to_string(),
      user: oci::User {
        uid: 1000,
        gid: 1000,
        umask: Some(0o022),
        additional_gids: vec![10, 20],
        username: "user".to_string(),
      },
      rlimits: vec![oci::POSIXRlimit::new("RLIMIT_NOFILE", 4096, 1024)],
      args: strings(&["/bin/sh", "-c", "true"]),
      terminal: true,
    }),
    hostname: "host".to_string(),
    domainname: "example.com".to_string(),
    mounts: vec![oci::Mount {
      r#type: "bind".to_string(),
      source: "/src".to_string(),
      destination: "/dst".to_string(),
      options: strings(&["rbind", "ro"]),
      uid_mappings: Some(vec![mapping]),
      gid_mappings: None,
    }],
    annotations: Some(map(&[("org.example/a", "1"), ("org.example/b", "2")])),
    root: Some(oci::Root::new("rootfs", true)),
    linux: Some(oci::Linux {
      uid_mappings: vec![mapping],
      gid_mappings: vec![mapping],
      sysctl: Some(map(&[("net.ipv4.ip_forward", "1")])),
      resources: Some(oci::LinuxResources {
        devices: vec![oci::LinuxDeviceCgroup::new(
          false,
          "a",
          None,
          Some(3),
          Some("rwm".to_string()),
        )],
        memory: Some(oci::LinuxMemory {
          limit: Some(1 << 30),
          reservation: Some(1 << 29),
          swap: Some(-1),
          kernel: None,
          kernel_tcp: Some(1 << 20),
          swappiness: Some(60),
          disable_oom_killer: Some(false),
          use_hierarchy: None,
          check_before_update: Some(true),
        }),
        cpu: Some(oci::LinuxCPU {
          shares: Some(1024),
          quota: Some(50_000),
          burst: None,
          period: Some(100_000),
          realtime_runtime: Some(-1),
          realtime_period: None,
          cpus: "0-3".to_string(),
          mems: "0".to_string(),
          idle: Some(0),
        }),
        pids: Some(oci::LinuxPids::new(512)),
        block_io: Some(oci::LinuxBlockIO::new(
          Some(500),
          None,
          vec![oci::LinuxWeightDevice::new(8, 0, Some(300), Some(200))],
          vec![throttle(1)],
          vec![throttle(2)],
          vec![throttle(3)],
          vec![throttle(4)],
        )),
        hugepage_limits: vec![oci::LinuxHugepageLimit::new("2MB", 1 << 21)],
        network: Some(oci::LinuxNetwork::new(
          Some(0x10001),
          vec![oci::LinuxInterfacePriority::new("eth0", 5)],
        )),
        rdma: Some(
          [
            ("mlx5_0".to_string(), oci::LinuxRdma::new(Some(3), Some(10_000))),
            ("mlx5_1".to_string(), oci::LinuxRdma::new(None, Some(1))),
          ]
          .into(),
        ),
        unified: Some(map(&[("memory.high", "max")])),
      }),
      cgroups_path: "/container".to_string(),
      namespaces: vec![
        oci::LinuxNamespace::new(oci::LinuxNamespaceType::Pid),
        oci::LinuxNamespace {
          r#type: oci::LinuxNamespaceType::Network,
          path: "/var/run/netns/one".to_string(),
        },
      ],
      devices: vec![oci::LinuxDevice::new(
        "/dev/fuse",
        "c",
        10,
        229,
        Some(0o666),
        Some(0),
        None,
      )],
      seccomp: Some(oci::LinuxSeccomp::new(
        oci::LinuxSeccompAction::ActErrno,
        Some(1),
        vec![oci::Arch::ArchAARCH64, oci::Arch::ArchX86_64],
        vec![oci::LinuxSeccompFlag::FlagLog],
        "/run/listener.sock",
        "metadata",
        vec![oci::LinuxSyscall::new(
          strings(&["personality"]),
          oci::LinuxSeccompAction::ActAllow,
          None,
          vec![oci::LinuxSeccompArg::new(
            0,
            0xffff_ffff,
            8,
            oci::LinuxSeccompOperator::OpMaskedEqual,
          )],
        )],
      )),
      rootfs_propagation: "rslave".to_string(),
      masked_paths: strings(&["/proc/kcore"]),
      readonly_paths: strings(&["/proc/sys"]),
      mount_label: "label".to_string(),
      personality: Some(oci::LinuxPersonality::new(
        oci::LinuxPersonalityDomain::PerLinux32,
        strings(&["flag"]),
      )),
    }),
  }
}

#[test]
fn round_trips_a_spec_through_a_bundle() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let path = directory.path().join("bundle");
  let spec = full_spec();

  let bundle = oci::Bundle::create(&path, &spec).expect("a bundle");
  assert_eq!(bundle.path(), path);
  assert_eq!(bundle.config_path().expect("a path"), path.join("config.json"));
  assert_eq!(bundle.rootfs_path().expect("a path"), path.join("rootfs"));
  assert!(path.join("rootfs").is_dir());

  assert_eq!(bundle.load_config().expect("the spec"), spec);
  assert_eq!(
    oci::Bundle::load(&path)
      .expect("the bundle")
      .load_config()
      .expect("the spec"),
    spec
  );

  bundle.delete().expect("deleted");
  assert!(!path.exists());
  assert!(oci::Bundle::load(&path).is_err());
}

#[test]
fn round_trips_a_spec_of_defaults() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let bundle = oci::Bundle::create(directory.path(), &oci::Spec::default()).expect("a bundle");

  assert_eq!(bundle.load_config().expect("the spec"), oci::Spec::default());
}

#[test]
fn writes_a_bundles_config_as_given() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let bundle = oci::Bundle::create_from_data(directory.path(), b"not a spec").expect("a bundle");

  assert_eq!(
    std::fs::read(bundle.config_path().expect("a path")).expect("the config"),
    b"not a spec"
  );
  assert!(bundle.load_config().is_err());
}

#[test]
fn makes_a_process_from_an_image_config() {
  let config = oci::ImageConfig {
    user: Some("nobody".to_string()),
    env: Some(strings(&["PATH=/bin"])),
    entrypoint: Some(strings(&["/bin/sh", "-c"])),
    cmd: Some(strings(&["true"])),
    working_dir: Some("/work".to_string()),
    ..Default::default()
  };

  let process = oci::Process::from_image_config(&config).expect("a process");
  assert_eq!(process.args, strings(&["/bin/sh", "-c", "true"]));
  assert_eq!(process.cwd, "/work");
  assert_eq!(process.env, strings(&["PATH=/bin"]));
  assert_eq!(process.user.username, "nobody");

  assert_eq!(
    oci::Process::from_image_config(&oci::ImageConfig::default()).expect("a process"),
    oci::Process::default()
  );
}

#[test]
fn describes_a_process_and_a_hook_without_their_secrets() {
  let process = oci::Process {
    env: strings(&["TOKEN=secret"]),
    ..Default::default()
  };
  let description = process.description().expect("a description");
  assert!(description.contains("TOKEN=<redacted>"), "{description}");
  assert!(!description.contains("secret"), "{description}");

  let hook = oci::Hook::new("/hook", Vec::new(), strings(&["TOKEN=secret"]), None);
  let description = hook.description().expect("a description");
  assert!(description.contains("TOKEN=<redacted>"), "{description}");
  assert!(!description.contains("secret"), "{description}");
}

#[test]
fn decodes_a_seccomp_profile() {
  let profile = oci::LinuxSeccomp::decode(
    br#"{
      "defaultAction": "SCMP_ACT_ERRNO",
      "architectures": ["SCMP_ARCH_AARCH64"],
      "syscalls": [{"names": ["read"], "action": "SCMP_ACT_ALLOW"}]
    }"#,
  )
  .expect("a profile");

  assert_eq!(
    profile,
    oci::LinuxSeccomp::new(
      oci::LinuxSeccompAction::ActErrno,
      None,
      vec![oci::Arch::ArchAARCH64],
      Vec::new(),
      "",
      "",
      vec![oci::LinuxSyscall::new(
        strings(&["read"]),
        oci::LinuxSeccompAction::ActAllow,
        None,
        Vec::new()
      )],
    )
  );
}

#[test]
fn rejects_a_seccomp_profile_that_is_not_one() {
  let error = oci::LinuxSeccomp::decode(b"not json").expect_err("not JSON");
  assert!(error.is_code(cfw::containerization_error::Code::InvalidArgument));
}

/// A plain decoder would accept this, dropping `includes` to allow `mount`
/// unconditionally.
#[test]
fn rejects_a_docker_format_seccomp_profile() {
  let error = oci::LinuxSeccomp::decode(
    br#"{
      "defaultAction": "SCMP_ACT_ERRNO",
      "syscalls": [{"names": ["mount"], "action": "SCMP_ACT_ALLOW", "includes": {"caps": ["CAP_SYS_ADMIN"]}}]
    }"#,
  )
  .expect_err("Docker's format");

  assert!(error.is_code(cfw::containerization_error::Code::InvalidArgument));
  assert!(error.to_string().contains("Docker's format"), "{error}");
}

#[test]
fn makes_the_default_seccomp_profile_for_this_machine() {
  let arch = oci::Arch::current_verified().expect("a seccomp architecture");
  assert_eq!(oci::Arch::current().expect("an answer"), Some(arch));

  let profile = oci::LinuxSeccomp::default_profile(None, arch).expect("a profile");
  assert!(profile.architectures.contains(&arch));
  assert!(!profile.syscalls.is_empty());

  let capabilities = oci::LinuxCapabilities {
    bounding: Some(strings(&["CAP_SYS_ADMIN"])),
    ..Default::default()
  };
  let privileged = oci::LinuxSeccomp::default_profile(Some(&capabilities), arch).expect("a profile");
  assert_ne!(privileged, profile, "CAP_SYS_ADMIN allows more syscalls");
}

#[test]
fn reads_the_runtime_spec_version() {
  assert_eq!(
    oci::RuntimeSpecVersion::current().expect("a version"),
    oci::RuntimeSpecVersion::new(1, 0, 2, "-dev")
  );
}

#[test]
fn converts_containerizations_types_to_ocis() {
  let rlimit = cfw::containerization::LinuxRLimit::new(cfw::containerization::linux_rlimit::Kind::OpenFiles, 1024);
  assert_eq!(
    rlimit.to_oci().expect("an rlimit"),
    oci::POSIXRlimit::new("RLIMIT_NOFILE", 1024, 1024)
  );

  let capabilities = cfw::containerization::LinuxCapabilities {
    bounding: vec![cfw::containerization_os::CapabilityName::Chown],
    ..Default::default()
  };
  assert_eq!(
    capabilities.to_oci().expect("capabilities"),
    oci::LinuxCapabilities {
      bounding: Some(strings(&["CAP_CHOWN"])),
      ..Default::default()
    },
    "empty sets are left out"
  );

  assert_eq!(
    cfw::containerization::SystemPlatform::LINUX_ARM
      .oci_platform()
      .expect("a platform"),
    oci::Platform::parse("linux/arm64").expect("a platform")
  );
}
