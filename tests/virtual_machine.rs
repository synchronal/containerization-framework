#![cfg(feature = "integration")]

//! VMs made by a VM manager of the test's own, without a container manager,
//! and the containers made on them.

mod support;

use containerization_framework as cfw;

/// Runs `/bin/true` in `container`, returning its exit code.
fn run_true(container: &cfw::containerization::container::LinuxContainer, id: &str) -> i32 {
  let process = container
    .exec(
      id,
      cfw::containerization::process::LinuxProcessConfiguration::new(&["/bin/true"]),
    )
    .expect("/bin/true should exec");

  support::container::finish(&process, "/bin/true")
}

#[test]
fn boots_a_virtual_machine_of_its_own() {
  let shared = tempfile::tempdir().expect("a temporary directory");
  let vmm = support::store::vmm();
  let config = cfw::containerization::vm::VMConfiguration {
    cpus: support::TEST_CPUS,
    memory_in_bytes: support::vm().memory_in_bytes,
    mounts_by_id: [(
      "shared".to_string(),
      vec![cfw::containerization::container::Mount::share(
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
    cfw::containerization::vm::VirtualMachineInstanceState::Stopped
  );
  assert_eq!(
    instance.virtiofs_layout(),
    cfw::containerization::vm::VirtiofsLayout::Unified
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
    cfw::containerization::vm::VirtualMachineInstanceState::Running
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

  let block = cfw::containerization::container::Mount::block("ext4", "/nonexistent.ext4", "/data", &[], &[]);
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
    cfw::containerization::vm::VirtualMachineInstanceState::Stopped
  );
}

#[test]
fn talks_to_the_guest_agent() {
  let vmm = support::store::vmm();
  let config = cfw::containerization::vm::VMConfiguration {
    cpus: support::TEST_CPUS,
    memory_in_bytes: support::vm().memory_in_bytes,
    ..Default::default()
  };
  let instance = vmm.create(&config).expect("a VM should be created");
  instance.start().expect("the VM should start");

  let agent = instance
    .dial_agent()
    .expect("the guest agent should answer");
  agent.standard_setup().expect("the guest should set up");

  agent
    .setenv("CFW_AGENT", "here")
    .expect("the agent should set a variable");
  assert_eq!(agent.getenv("CFW_AGENT").expect("the agent should read it"), "here");

  agent
    .mkdir("/tmp/cfw-agent/nested", true, 0o755)
    .expect("the agent should make a directory");
  let stat = agent
    .stat("/", "/tmp/cfw-agent/nested")
    .expect("the agent should stat the directory");
  assert_eq!(stat.mode & 0o170000, 0o040000, "{stat:?} should be a directory");
  assert_ne!(stat.ino, 0);

  let missing = agent
    .stat("/", "/tmp/cfw-agent/missing")
    .expect_err("a missing path has no metadata");
  assert!(
    missing.is_code(cfw::containerization_error::Code::NotFound),
    "{missing}"
  );

  agent
    .up("lo", None)
    .expect("the agent should bring the loopback up");
  agent.sync().expect("the agent should sync");
  agent
    .kill(1, 0)
    .expect("the agent should signal its own process");

  agent.close().expect("the agent's connection should close");
  instance.stop().expect("the VM should stop");
}

#[test]
fn pauses_and_resumes_a_container_on_its_own_manager() {
  let name = "cfw-test-vmm-container";
  let directory = tempfile::tempdir().expect("a temporary directory");
  let rootfs = support::store::unpack(&directory.path().join("rootfs.ext4"));
  let vmm = support::store::vmm();

  let container = cfw::containerization::container::LinuxContainer::new_with(
    name,
    rootfs,
    None,
    &vmm,
    support::vm(),
    support::container::keep_alive,
  )
  .expect("a container should be made on the VM manager");

  container.create().expect("the container should create");
  container.start().expect("the container should start");
  assert_eq!(run_true(&container, "before"), 0);

  container
    .with_virtual_machine_instance(|instance| {
      assert_eq!(
        instance.state(),
        cfw::containerization::vm::VirtualMachineInstanceState::Running
      );
      assert!(
        instance.mounts().contains_key(name),
        "the container's mounts are attached under its ID"
      );

      instance.pause()?;
      assert_eq!(
        instance.state(),
        cfw::containerization::vm::VirtualMachineInstanceState::Unknown,
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
  let rootfs = cfw::containerization::container::Mount::block("ext4", "/rootfs.ext4", "/", &[], &[]);
  let configuration = cfw::containerization::container::linux_container::Configuration {
    hostname: Some("configured".to_string()),
    ..Default::default()
  };

  let container = cfw::containerization::container::LinuxContainer::new(
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

  let not_a_block = cfw::containerization::container::Mount::any("tmpfs", "tmpfs", "/", &[], &[]);
  assert!(
    cfw::containerization::container::LinuxContainer::new(
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
  let manager = cfw::containerization::container::ContainerManager::with_vmm(&vmm, None)
    .expect("a manager should be made on the VM manager");

  assert_eq!(
    manager.image_store().path(),
    cfw::containerization::image::ImageStore::default_store()
      .expect("the default store should open")
      .path()
  );
}
