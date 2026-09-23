#![cfg(feature = "integration")]

//! Booting a container and running processes in it.
//!
//! What no host-side test can show: that a built image boots, that a guest's
//! exit code comes back as its own, and that the descriptors a caller attaches
//! are the ones the guest writes to.
//!
//! One container serves all of it. Nextest runs each test in a process of its
//! own and a container dies with the process that booted it, so every test is a
//! VM.

mod support;

use support::Container;

#[test]
fn boots_and_runs_processes() {
  let container = Container::boot("cfw-test-session");

  assert!(
    container.session().is_running(container.name()),
    "the process that booted a container should see it running"
  );
  assert!(
    !container.session().is_running("cfw-test-never-booted"),
    "and should answer for that container alone"
  );

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
    container
      .capture("marker", &["/bin/cat", support::MARKER_PATH])
      .trim(),
    support::MARKER,
    "the step the image was built with should be in a container booted from it"
  );
}
