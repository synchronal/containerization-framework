#![cfg(feature = "integration")]

//! Creating a container and running processes in it.
//!
//! What no host-side test can show: that an image boots, that a guest's exit
//! code comes back as its own, and that the descriptors a caller attaches are
//! the ones the guest writes to.
//!
//! Nextest runs each test in a process of its own and a container dies with
//! the process that created it, so every container is a VM.

mod support;

use containerization_framework as cfw;
use support::container::Container;

#[test]
fn creates_and_runs_processes() {
  let container = Container::boot("cfw-test-container");

  assert_eq!(container.exec("true", &["/bin/true"]), 0);
  assert_eq!(container.exec("false", &["/bin/false"]), 1);
  assert_eq!(
    container.exec("three", &["/bin/sh", "-c", "exit 3"]),
    3,
    "an arbitrary exit code comes back whole"
  );

  assert_eq!(
    container.capture("echo", &["/bin/echo", "hello"]).trim(),
    "hello",
    "the guest writes to the descriptor it was handed, not through this process"
  );

  assert_eq!(
    container.sh("hostname", "hostname").trim(),
    container.container().id(),
    "an unnamed host should take the container's id"
  );
  assert!(
    container
      .sh("pid-one", "tr '\\0' ' ' < /proc/1/cmdline")
      .starts_with("/bin/sh "),
    "a container not asked for an init should run its first process as PID 1"
  );
  assert!(
    container.directory().join("bootlog.log").is_file(),
    "the manager should seed a boot log in the container's directory"
  );
}

#[test]
fn boots_from_copies_of_one_unpacked_rootfs() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let unpacked = support::store::unpack(&directory.path().join("rootfs.ext4"));
  let copy = |name: &str| {
    let path = directory.path().join(name);
    std::fs::copy(&unpacked.source, &path).expect("the rootfs should copy");

    cfw::containerization::container::Mount {
      source: path.display().to_string(),
      ..unpacked.clone()
    }
  };

  let first = Container::boot_from_rootfs("cfw-test-rootfs-first", copy("first.ext4"));
  let second = Container::boot_from_rootfs("cfw-test-rootfs-second", copy("second.ext4"));

  first.sh("write", "echo first > /marker");

  assert_eq!(first.sh("read", "cat /marker").trim(), "first");
  assert_eq!(
    second.exec("look", &["/bin/sh", "-c", "test -e /marker"]),
    1,
    "a copy doesn't see what another copy's container wrote"
  );
}

#[test]
fn reports_what_it_was_made_with() {
  let booted = Container::boot("cfw-test-container-getters");
  let container = booted.container();
  let config = container.config();

  assert!(
    container.rootfs().is_block(),
    "the manager unpacks the image to a block"
  );
  assert_eq!(container.writable_layer(), None);
  assert_eq!(container.vm(), support::vm());
  assert_eq!(config.cpus, support::TEST_CPUS);
  assert_eq!(config.memory_in_bytes, support::TEST_MEMORY_IN_BYTES);
  assert_eq!(container.cpus(), config.cpus);
  assert_eq!(container.memory_in_bytes(), config.memory_in_bytes);
  assert_eq!(container.interfaces(), config.interfaces);
  assert_eq!(
    config.interfaces.len(),
    1,
    "the suite gives each container one interface"
  );
}

#[test]
fn execs_with_a_closure_on_swifts_defaults() {
  let booted = Container::boot("cfw-test-container-exec-with");
  let container = booted.container();
  let seen = std::sync::Arc::new(std::sync::Mutex::new(None));
  let slot = std::sync::Arc::clone(&seen);

  let process = container
    .exec_with("defaults", move |configuration| {
      *slot.lock().unwrap() = Some(configuration.clone());
      configuration.arguments = vec!["/bin/true".into()];
      Ok(())
    })
    .expect("exec_with should exec");

  assert_eq!(process.owning_container(), Some(container.id()));
  process.start().expect("the process should start");
  assert_eq!(
    process
      .wait(None)
      .expect("the process should finish")
      .exit_code,
    0
  );
  let _ = process.delete();

  assert_eq!(
    seen.lock().unwrap().take(),
    Some(cfw::containerization::process::LinuxProcessConfiguration::default()),
    "the closure should change a LinuxProcessConfiguration()"
  );

  let thrown = cfw::Error::failed("configure throws", "on purpose");
  let returned = container
    .exec_with("throws", {
      let thrown = thrown.clone();
      move |_| Err(thrown)
    })
    .expect_err("a closure that throws should fail the exec");

  assert_eq!(returned, thrown, "the closure's own error should come back");
}

#[test]
fn reports_statistics_and_refuses_a_port_nothing_listens_on() {
  let booted = Container::boot("cfw-test-container-statistics");
  let container = booted.container();

  let statistics = container
    .statistics(cfw::containerization::container::StatCategory::MEMORY)
    .expect("the container should report statistics");

  assert_eq!(statistics.id, container.id());
  assert!(statistics.memory.expect("memory was asked for").usage_bytes > 0);
  assert_eq!(statistics.process, None, "only memory was asked for");

  let statistics = container
    .statistics(cfw::containerization::container::StatCategory::ALL)
    .expect("the container should report every category");

  assert!(statistics.process.is_some());
  assert!(statistics.cpu.is_some());
  assert!(statistics.memory_events.is_some());

  assert!(
    container.dial_vsock(9999).is_err(),
    "nothing in the guest listens on the port"
  );
}

#[test]
fn copies_files_in_and_out_and_freezes_the_filesystem() {
  let booted = Container::boot("cfw-test-container-copy");
  let container = booted.container();
  let directory = tempfile::tempdir().expect("a temporary directory");
  let source = directory.path().join("in.txt");
  let destination = directory.path().join("out").join("copied.txt");
  std::fs::write(&source, "copied across\n").expect("the source should write");

  container
    .copy_in(
      &source,
      std::path::Path::new("/tmp/copied.txt"),
      cfw::containerization::container::linux_container::CopyInOptions::default(),
    )
    .expect("the file should copy in");
  assert_eq!(booted.sh("cat", "cat /tmp/copied.txt"), "copied across\n");

  container
    .copy_out(
      std::path::Path::new("/tmp/copied.txt"),
      &destination,
      cfw::containerization::container::linux_container::CopyOutOptions::default(),
    )
    .expect("the file should copy out, creating its parent");
  assert_eq!(
    std::fs::read_to_string(&destination).expect("the copy should read"),
    "copied across\n"
  );

  container
    .filesystem_operation(cfw::containerization::container::FilesystemOperation::Freeze, "/")
    .expect("the root filesystem should freeze");
  container
    .filesystem_operation(cfw::containerization::container::FilesystemOperation::Thaw, "/")
    .expect("the root filesystem should thaw");
}
