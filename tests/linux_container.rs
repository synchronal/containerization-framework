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
use std::io::Write;
use std::os::fd::AsRawFd;
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

/// The bridge duplicates each descriptor a process is given. Once the process
/// is deleted, the duplicates must stop reading the caller's input and stop
/// holding their output open, though the caller still holds the process.
#[test]
fn lets_go_of_the_callers_descriptors_once_a_process_is_deleted() {
  let booted = Container::boot("cfw-test-container-descriptors");
  let container = booted.container();
  let (mut stdin, mut input) = std::io::pipe().expect("a pipe should be creatable");
  let (mut read, output) = std::io::pipe().expect("a pipe should be creatable");

  let process = container
    .exec(
      "both",
      cfw::containerization::process::LinuxProcessConfiguration {
        stdin: Some(stdin.as_raw_fd()),
        stdout: Some(output.as_raw_fd()),
        stderr: Some(output.as_raw_fd()),
        ..cfw::containerization::process::LinuxProcessConfiguration::new(&["/bin/sh", "-c", "echo out; echo err >&2"])
      },
    )
    .expect("the process should exec");
  assert_eq!(support::container::finish(&process, "both"), 0);

  drop(output);
  let written = support::container::read_to_end(&mut read);
  assert!(
    written.contains("out\n") && written.contains("err\n"),
    "stdout and stderr should share the descriptor, but wrote {written:?}"
  );

  input
    .write_all(b"still ours")
    .expect("the pipe should take input");
  drop(input);
  // Give a duplicate that is still reading stdin time to take the input first.
  std::thread::sleep(std::time::Duration::from_secs(1));
  assert_eq!(
    support::container::read_to_end(&mut stdin),
    "still ours",
    "the deleted process should no longer read stdin"
  );

  drop(process);
}

/// Reading the caller's stdin can fail rather than reach end of file, as a
/// pty does once its terminal window has closed. That has to end the
/// process's input, as end of file does, not abort the process the container
/// lives in. A TCP connection its peer reset fails every read
/// (ECONNRESET), so it stands in for the hung-up terminal.
#[test]
fn ends_input_when_reading_stdin_fails() {
  let booted = Container::boot("cfw-test-container-stdin-error");
  let container = booted.container();
  let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback listener should bind");
  let stdin = std::net::TcpStream::connect(listener.local_addr().expect("the listener has an address"))
    .expect("the listener should take a connection");
  let (peer, _) = listener
    .accept()
    .expect("the connection should be accepted");
  let reset = libc::linger {
    l_onoff: 1,
    l_linger: 0,
  };
  // SAFETY: `peer` is ours, and `reset` is a linger of the size passed.
  let set = unsafe {
    libc::setsockopt(
      peer.as_raw_fd(),
      libc::SOL_SOCKET,
      libc::SO_LINGER,
      (&reset as *const libc::linger).cast(),
      std::mem::size_of::<libc::linger>() as libc::socklen_t,
    )
  };
  assert_eq!(set, 0, "SO_LINGER should set: {}", std::io::Error::last_os_error());
  // Closing with a zero linger resets the connection instead of ending it.
  drop(peer);
  let (mut read, output) = std::io::pipe().expect("a pipe should be creatable");

  let process = container
    .exec(
      "unreadable",
      cfw::containerization::process::LinuxProcessConfiguration {
        stdin: Some(stdin.as_raw_fd()),
        stdout: Some(output.as_raw_fd()),
        ..cfw::containerization::process::LinuxProcessConfiguration::new(&[
          "/bin/sh",
          "-c",
          "cat >/dev/null; echo input ended",
        ])
      },
    )
    .expect("the process should exec");
  assert_eq!(support::container::finish(&process, "unreadable"), 0);

  drop(output);
  assert_eq!(support::container::read_to_end(&mut read), "input ended\n");
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
    statistics.network_protocols.is_some(),
    "the network category should report protocol traffic"
  );

  booted.sh("ping", "ping -c 1 -W 1 127.0.0.1 >/dev/null 2>&1; true");
  let protocols = container
    .statistics(cfw::containerization::container::StatCategory::NETWORK)
    .expect("the container should report network statistics")
    .network_protocols
    .expect("the network category should report protocol traffic");
  assert!(
    protocols.ip.received_bytes.is_some(),
    "IP keeps a byte counter: {protocols:?}"
  );
  assert_eq!(
    protocols.icmp.received_drops, None,
    "ICMP has no drop counter, which isn't the same as no drops"
  );
  assert!(
    protocols.icmp.received_packets > 0 && protocols.icmp.transmitted_packets > 0,
    "a ping should count as ICMP, not as {protocols:?}"
  );
  assert!(
    protocols.ip.received_packets >= protocols.icmp.received_packets,
    "every ICMP packet is also an IP packet: {protocols:?}"
  );

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
    .filesystem_operation(cfw::containerization::vm::FilesystemOperation::Freeze, "/")
    .expect("the root filesystem should freeze");
  container
    .filesystem_operation(cfw::containerization::vm::FilesystemOperation::Thaw, "/")
    .expect("the root filesystem should thaw");
}
