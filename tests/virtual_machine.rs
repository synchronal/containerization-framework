#![cfg(feature = "integration")]

//! VMs made by a VM manager of the test's own, without a container manager,
//! and the containers made on them.

mod support;

use containerization_framework as cfw;

/// Holds a container open; the image's own `Cmd` would exit at once.
const KEEPALIVE: [&str; 3] = ["/bin/sh", "-c", "while :; do sleep 86400; done"];

fn keep_alive(configuration: &mut cfw::containerization::linux_container::Configuration) {
  configuration.process.arguments = KEEPALIVE.map(String::from).to_vec();
  configuration.cpus = support::TEST_CPUS;
  configuration.memory_in_bytes = support::TEST_MEMORY_IN_BYTES;
}

/// Runs `/bin/true` in `container`, returning its exit code.
fn run_true(container: &cfw::containerization::LinuxContainer, id: &str) -> i32 {
  let process = container
    .exec(
      id,
      cfw::containerization::LinuxProcessConfiguration::new(&["/bin/true"]),
    )
    .expect("/bin/true should exec");

  process.start().expect("/bin/true should start");
  let status = process.wait(None).expect("/bin/true should finish");
  let _ = process.delete();

  status.exit_code
}

#[test]
fn boots_a_virtual_machine_of_its_own() {
  let shared = tempfile::tempdir().expect("a temporary directory");
  let vmm = support::store::vmm();
  let config = cfw::containerization::VmConfiguration {
    cpus: support::TEST_CPUS,
    memory_in_bytes: support::vm().memory_in_bytes,
    mounts_by_id: [(
      "shared".to_string(),
      vec![cfw::containerization::Mount::share(
        shared.path().display().to_string(),
        "/mnt/shared",
        &[],
        &[],
      )],
    )]
    .into(),
    ..Default::default()
  };

  let instance = vmm.create(&config).expect("a VM should be created");

  assert_eq!(
    instance.state(),
    cfw::containerization::VirtualMachineInstanceState::Stopped
  );
  assert_eq!(
    instance.virtiofs_layout(),
    cfw::containerization::VirtiofsLayout::Unified
  );

  let mounts = instance.mounts();
  let [share] = mounts["shared"].as_slice() else {
    panic!("the VM should attach the one share, not {mounts:?}");
  };
  assert_eq!(share.r#type, "virtiofs");
  assert_eq!(share.destination, "/mnt/shared");

  instance.start().expect("the VM should start");
  assert_eq!(
    instance.state(),
    cfw::containerization::VirtualMachineInstanceState::Running
  );

  let listener = instance.listen(0x2000).expect("the VM should listen");
  assert_eq!(listener.port(), 0x2000);

  std::thread::scope(|scope| {
    let waiting = scope.spawn(|| (&listener).next());

    // Long enough for the other thread to be waiting, which finishing ends.
    std::thread::sleep(std::time::Duration::from_millis(100));
    listener.finish().expect("the listener should finish");

    assert!(
      waiting.join().expect("the waiting thread").is_none(),
      "finishing ends a wait for a connection"
    );
  });
  listener.finish().expect("finishing again does nothing");
  assert!((&listener).next().is_none(), "a finished listener has no connections");

  let block = cfw::containerization::Mount::block("ext4", "/nonexistent.ext4", "/data", &[], &[]);
  assert!(
    instance.hotplug(block, "hotplugged").is_err(),
    "a VM without a hotplug provider can't hotplug"
  );
  instance
    .release_hotplug("hotplugged")
    .expect("without a hotplug provider, releasing does nothing");

  instance.stop().expect("the VM should stop");
  assert_eq!(
    instance.state(),
    cfw::containerization::VirtualMachineInstanceState::Stopped
  );
}

#[test]
fn pauses_and_resumes_a_container_on_its_own_manager() {
  let name = "cfw-test-vmm-container";
  let directory = tempfile::tempdir().expect("a temporary directory");
  let rootfs = support::store::unpack(&directory.path().join("rootfs.ext4"));
  let vmm = support::store::vmm();

  let container = cfw::containerization::LinuxContainer::new_with(name, rootfs, None, &vmm, support::vm(), keep_alive)
    .expect("a container should be made on the VM manager");

  container.create().expect("the container should create");
  container.start().expect("the container should start");
  assert_eq!(run_true(&container, "before"), 0);

  container
    .with_virtual_machine_instance(|instance| {
      assert_eq!(
        instance.state(),
        cfw::containerization::VirtualMachineInstanceState::Running
      );
      assert!(
        instance.mounts().contains_key(name),
        "the container's mounts are attached under its ID"
      );

      instance.pause()?;
      assert_eq!(
        instance.state(),
        cfw::containerization::VirtualMachineInstanceState::Unknown,
        "Swift has no paused state"
      );
      instance.resume()
    })
    .expect("the container's VM should pause and resume");

  assert_eq!(run_true(&container, "after"), 0, "an exec after resuming works");
  container.stop().expect("the container should stop");
}

#[test]
fn makes_a_container_from_a_configuration() {
  let vmm = support::store::vmm();
  let rootfs = cfw::containerization::Mount::block("ext4", "/rootfs.ext4", "/", &[], &[]);
  let configuration = cfw::containerization::linux_container::Configuration {
    hostname: Some("configured".to_string()),
    ..Default::default()
  };

  let container = cfw::containerization::LinuxContainer::new(
    "cfw-test-vmm-configured",
    rootfs.clone(),
    None,
    &vmm,
    support::vm(),
    configuration,
  )
  .expect("a container should be made from a configuration");

  assert_eq!(container.config().hostname.as_deref(), Some("configured"));
  assert_eq!(container.vm(), support::vm());
  assert!(
    container.with_virtual_machine_instance(|_| Ok(())).is_err(),
    "a container has no VM before it's created"
  );

  let not_a_block = cfw::containerization::Mount::any("tmpfs", "tmpfs", "/", &[], &[]);
  assert!(
    cfw::containerization::LinuxContainer::new(
      "cfw-test-vmm-not-a-block",
      rootfs,
      Some(not_a_block),
      &vmm,
      support::vm(),
      Default::default(),
    )
    .is_err(),
    "a writable layer must be a block"
  );
}

#[test]
fn makes_a_manager_on_the_default_store() {
  let vmm = support::store::vmm();
  let manager =
    cfw::containerization::ContainerManager::with_vmm(&vmm, None).expect("a manager should be made on the VM manager");

  assert_eq!(
    manager.image_store().path(),
    cfw::containerization::ImageStore::default_store()
      .expect("the default store should open")
      .path()
  );
}
