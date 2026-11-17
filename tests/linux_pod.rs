#![cfg(feature = "integration")]

//! A pod of containers sharing one VM, and a volume.

mod support;

use containerization_framework as cfw;

/// Runs a `/bin/sh` script in `container`, returning its exit code and what
/// it wrote to stdout.
fn sh(pod: &cfw::containerization::container::LinuxPod, container: &str, id: &str, script: &str) -> (i32, String) {
  let script = script.to_string();

  support::container::captured(&format!("{id} in {container}"), |stdout| {
    pod.exec_in_container(container, id, move |process| {
      process.arguments = vec!["/bin/sh".into(), "-c".into(), script];
      process.stdout = Some(stdout);
      Ok(())
    })
  })
}

#[test]
fn reads_back_the_longest_nbd_timeout() {
  let vmm = support::store::vmm();
  let volume = cfw::containerization::container::linux_pod::PodVolume {
    name: "remote".to_string(),
    source: cfw::containerization::container::linux_pod::pod_volume::Source::Nbd {
      url: "nbd://localhost:10809/remote".to_string(),
      timeout: Some(std::time::Duration::MAX),
      read_only: false,
    },
    format: "ext4".to_string(),
  };
  let configured = volume.clone();

  let pod =
    cfw::containerization::container::LinuxPod::new("cfw-test-nbd-pod", &vmm, support::vm(), move |configuration| {
      configuration.volumes = vec![configured];
      Ok(())
    })
    .expect("a pod should be made");

  assert_eq!(pod.config().volumes, [volume]);
}

#[test]
fn shares_a_host_directory_as_a_read_only_volume() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let shared = directory.path().join("shared");
  std::fs::create_dir(&shared).expect("the shared directory");
  std::fs::write(shared.join("greeting"), "hello\n").expect("the shared file");
  let vmm = support::store::vmm();
  let volume = cfw::containerization::container::linux_pod::PodVolume {
    name: "host".to_string(),
    source: cfw::containerization::container::linux_pod::pod_volume::Source::Virtiofs {
      path: shared.clone(),
      read_only: true,
    },
    format: "virtiofs".to_string(),
  };
  let configured = volume.clone();

  let pod = cfw::containerization::container::LinuxPod::new(
    "cfw-test-virtiofs-pod",
    &vmm,
    support::vm(),
    move |configuration| {
      configuration.volumes = vec![configured];
      Ok(())
    },
  )
  .expect("a pod should be made");

  assert_eq!(pod.config().volumes, [volume]);

  let rootfs = support::store::unpack(&directory.path().join("reader.ext4"));
  pod
    .add_container("reader", rootfs, |configuration| {
      support::container::keep_alive_in_pod(configuration)?;
      configuration
        .mounts
        .push(cfw::containerization::container::Mount::shared_mount(
          "host",
          "/host",
          Default::default(),
        ));
      Ok(())
    })
    .expect("the reader should be added");

  pod.create().expect("the pod should create");
  pod
    .start_container("reader")
    .expect("the reader should start");

  assert_eq!(
    sh(&pod, "reader", "read", "cat /host/greeting"),
    (0, "hello\n".to_string()),
    "the container sees the host's file"
  );
  assert_ne!(
    sh(&pod, "reader", "write", "echo bye > /host/greeting").0,
    0,
    "a read-only volume refuses a write"
  );

  pod
    .stop_container("reader")
    .expect("the reader should stop");
  pod.stop().expect("the pod should stop");
  assert_eq!(
    std::fs::read_to_string(shared.join("greeting")).expect("the shared file"),
    "hello\n"
  );
}

/// Runs busybox `sh` scripts in the pod's VM root rather than in a container,
/// as upstream's `PodRootProcessTests` does. The VM root has no dynamic
/// loader, so busybox runs through the loader in `container`'s rootfs, and
/// `$B` runs a busybox applet.
struct RootShell {
  agent: cfw::containerization::vm::Vminitd,
  loader: String,
  busybox: String,
}

impl RootShell {
  fn new(pod: &cfw::containerization::container::LinuxPod, container: &str) -> Self {
    let rootfs = format!("/run/container/{container}/rootfs");
    Self {
      agent: pod
        .with_virtual_machine_instance(|instance| instance.dial_agent())
        .expect("the pod's guest agent should answer"),
      loader: format!("{rootfs}/lib/ld-musl-aarch64.so.1"),
      busybox: format!("{rootfs}/bin/busybox"),
    }
  }

  /// Returns the script's exit code.
  fn run(
    &self,
    id: &str,
    script: &str,
    capabilities: &[cfw::containerization_os::linux::CapabilityName],
    resources: Option<cfw::containerization_oci::runtime::LinuxResources>,
  ) -> i32 {
    let process = cfw::containerization_oci::runtime::Process {
      args: support::strings(&[&self.loader, &self.busybox, "sh", "-c", script]),
      env: vec![format!("B={} {}", self.loader, self.busybox)],
      capabilities: (!capabilities.is_empty()).then(|| {
        cfw::containerization::process::LinuxCapabilities::with_capabilities(capabilities.to_vec())
          .to_oci()
          .expect("OCI capabilities")
      }),
      ..Default::default()
    };
    let spec = cfw::containerization_oci::runtime::Spec {
      process: Some(process),
      linux: resources.map(|resources| cfw::containerization_oci::runtime::Linux {
        resources: Some(resources),
        ..Default::default()
      }),
      ..Default::default()
    };

    self
      .agent
      .create_process(id, None, None, None, None, None, &spec, None)
      .unwrap_or_else(|error| panic!("{id} should be created: {error}"));
    self
      .agent
      .start_process(id, None)
      .unwrap_or_else(|error| panic!("{id} should start: {error}"));
    let status = self
      .agent
      .wait_process(id, None, None)
      .unwrap_or_else(|error| panic!("{id} should exit: {error}"));
    self
      .agent
      .delete_process(id, None)
      .unwrap_or_else(|error| panic!("{id} should be deleted: {error}"));
    status.exit_code
  }
}

#[test]
fn runs_processes_in_its_vms_root() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let vmm = support::store::vmm();
  let nameserver = "192.0.2.53";
  let pod =
    cfw::containerization::container::LinuxPod::new("cfw-test-root-pod", &vmm, support::vm(), |configuration| {
      configuration.dns = Some(cfw::containerization::network::DNS {
        nameservers: vec![nameserver.to_string()],
        ..Default::default()
      });
      Ok(())
    })
    .expect("a pod should be made");
  let rootfs = support::store::unpack(&directory.path().join("sleeper.ext4"));
  pod
    .add_container("sleeper", rootfs, support::container::keep_alive_in_pod)
    .expect("the sleeper should be added");
  pod.create().expect("the pod should create");
  pod
    .start_container("sleeper")
    .expect("the sleeper should start");
  let shell = RootShell::new(&pod, "sleeper");

  assert_eq!(
    shell.run(
      "root-dns",
      &format!("$B grep -qx 'nameserver {nameserver}' /etc/resolv.conf"),
      &[],
      None,
    ),
    0,
    "the VM root's resolv.conf names the pod's nameserver"
  );

  assert_eq!(
    shell.run(
      "root-namespaces",
      r#"
      c=$($B pidof sleep) || exit 10
      for n in cgroup ipc mnt pid uts net; do
        [ "$($B readlink /proc/self/ns/$n)" = "$($B readlink /proc/1/ns/$n)" ] || exit 11
      done
      [ "$($B readlink /proc/$c/ns/mnt)" != "$($B readlink /proc/self/ns/mnt)" ] || exit 12
      [ -f /proc/$c/root/etc/alpine-release ] || exit 13
      "#,
      &[cfw::containerization_os::linux::CapabilityName::SysPtrace],
      None,
    ),
    0,
    "10: no container process; 11: not in PID 1's namespaces; 12: shares the container's mount namespace; 13: can't read the container's root"
  );

  let mountpoint = "/run/root-process-probe";
  let mount = format!("$B mount -t tmpfs tmpfs {mountpoint}");
  shell
    .agent
    .mkdir(mountpoint, true, 0o755)
    .expect("the agent should make the mountpoint");
  let unmounted = shell
    .agent
    .stat("/", mountpoint)
    .expect("the mountpoint's metadata")
    .dev;
  assert_ne!(
    shell.run("root-unprivileged", &mount, &[], None),
    0,
    "a root process without CAP_SYS_ADMIN can't mount"
  );
  assert_eq!(
    shell.run(
      "root-privileged",
      &mount,
      &[cfw::containerization_os::linux::CapabilityName::SysAdmin],
      None,
    ),
    0,
    "a root process with CAP_SYS_ADMIN can mount"
  );
  assert_ne!(
    shell
      .agent
      .stat("/", mountpoint)
      .expect("the mountpoint's metadata")
      .dev,
    unmounted,
    "the agent sees the root process's mount"
  );
  shell
    .agent
    .umount(mountpoint, 0)
    .expect("the agent should unmount it");

  let limit: i64 = 64 * 1024 * 1024;
  assert_eq!(
    shell.run(
      "root-limited",
      &format!(
        r#"
        [ "$($B cat /proc/self/cgroup)" = "0::/vminitd-root/root-limited" ] || exit 10
        [ "$($B cat /sys/fs/cgroup/vminitd-root/root-limited/memory.max)" = "{limit}" ] || exit 11
        "#
      ),
      &[],
      Some(cfw::containerization_oci::runtime::LinuxResources {
        memory: Some(cfw::containerization_oci::runtime::LinuxMemory {
          limit: Some(limit),
          ..Default::default()
        }),
        ..Default::default()
      }),
    ),
    0,
    "10: not in its own cgroup; 11: the wrong memory limit"
  );

  shell
    .agent
    .close()
    .expect("the agent's connection should close");
  pod.stop().expect("the pod should stop");
}

#[test]
fn resumes_a_panic_in_a_containers_configuration_on_the_callers_thread() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let vmm = support::store::vmm();
  let pod = cfw::containerization::container::LinuxPod::new("cfw-test-panic-pod", &vmm, support::vm(), |_| Ok(()))
    .expect("a pod should be made");
  let rootfs = support::store::unpack(&directory.path().join("panics.ext4"));

  let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
    pod.add_container("panics", rootfs, |_| panic!("the configuration gave up"))
  }))
  .expect_err("the closure's panic should reach the caller");

  assert_eq!(panicked.downcast_ref::<&str>(), Some(&"the configuration gave up"));
  assert!(pod.list_containers().is_empty());
}

#[test]
fn shares_a_volume_between_its_containers() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let vmm = support::store::vmm();
  let volume = cfw::containerization::container::linux_pod::PodVolume {
    name: "shared".to_string(),
    source: cfw::containerization::container::linux_pod::pod_volume::Source::Tmpfs { size_bytes: None },
    format: "tmpfs".to_string(),
  };
  let configured = volume.clone();

  let pod =
    cfw::containerization::container::LinuxPod::new("cfw-test-pod", &vmm, support::vm(), move |configuration| {
      configuration.hostname = Some("pod".to_string());
      configuration.volumes = vec![configured];
      Ok(())
    })
    .expect("a pod should be made");

  assert_eq!(pod.id(), "cfw-test-pod");
  assert_eq!(pod.vm(), support::vm());
  assert_eq!(pod.config().volumes, [volume]);
  assert_eq!(pod.config().hostname.as_deref(), Some("pod"));

  for name in ["writer", "reader"] {
    let rootfs = support::store::unpack(&directory.path().join(format!("{name}.ext4")));

    pod
      .add_container(name, rootfs, |configuration| {
        support::container::keep_alive_in_pod(configuration)?;
        configuration
          .mounts
          .push(cfw::containerization::container::Mount::shared_mount(
            "shared",
            "/shared",
            Default::default(),
          ));
        Ok(())
      })
      .unwrap_or_else(|error| panic!("{name} should be added: {error}"));
  }

  pod.create().expect("the pod should create");
  pod
    .start_container("writer")
    .expect("the writer should start");
  pod
    .start_container("reader")
    .expect("the reader should start");

  let mut containers = pod.list_containers();
  containers.sort();
  assert_eq!(containers, ["reader", "writer"]);

  assert_eq!(sh(&pod, "writer", "write", "echo hello > /shared/greeting").0, 0);
  assert_eq!(
    sh(&pod, "reader", "read", "cat /shared/greeting"),
    (0, "hello\n".to_string()),
    "the reader sees what the writer wrote"
  );
  assert_eq!(
    sh(&pod, "reader", "hostname", "hostname"),
    (0, "pod\n".to_string()),
    "a container takes the pod's hostname"
  );

  let statistics = pod
    .statistics(cfw::containerization::container::linux_pod::StatisticsOptions {
      container_ids: Some(&["writer"]),
      categories: cfw::containerization::container::StatCategory::PROCESS,
    })
    .expect("the pod should report statistics");
  let [writer] = statistics.as_slice() else {
    panic!("the pod should report the one container asked for, not {statistics:?}");
  };
  assert_eq!(writer.id, "writer");
  assert!(writer.process.is_some());

  pod
    .with_virtual_machine_instance(|instance| {
      assert_eq!(
        instance.state(),
        cfw::containerization::vm::VirtualMachineInstanceState::Running
      );
      Ok(())
    })
    .expect("the pod's VM should be reachable");

  pod
    .stop_container("reader")
    .expect("the reader should stop");
  pod.stop().expect("the pod should stop");
}

#[test]
fn writes_a_containers_unified_cgroup_files() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let vmm = support::store::vmm();
  let pod = cfw::containerization::container::LinuxPod::new("cfw-test-unified-pod", &vmm, support::vm(), |_| Ok(()))
    .expect("a pod should be made");
  let rootfs = support::store::unpack(&directory.path().join("unified.ext4"));

  pod
    .add_container("unified", rootfs, |configuration| {
      support::container::keep_alive_in_pod(configuration)?;
      configuration.unified = [("memory.oom.group".to_string(), "1".to_string())].into();
      Ok(())
    })
    .expect("the container should be added");
  pod.create().expect("the pod should create");
  pod
    .start_container("unified")
    .expect("the container should start");

  assert_eq!(
    sh(&pod, "unified", "oom-group", "cat /sys/fs/cgroup/memory.oom.group"),
    (0, "1\n".to_string())
  );

  pod.stop().expect("the pod should stop");
}
