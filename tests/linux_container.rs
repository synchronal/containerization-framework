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

    cfw::containerization::Mount {
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
