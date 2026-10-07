#![cfg(feature = "integration")]

//! The container manager's store and network, and the vmnet network that hands
//! out interfaces.
//!
//! vmnet may refuse a process that lacks its entitlement. The tests that need
//! a network say so and pass rather than fail on such a host.

mod support;

use containerization_framework as cfw;
use support::container::Container;

/// A shared network, or `None` with the reason printed when vmnet refuses.
fn vmnet_network(test: &str) -> Option<cfw::containerization::network::VmnetNetwork> {
  cfw::containerization::network::VmnetNetwork::new(
    cfw::containerization::network::vmnet_network::Mode::Shared,
    None,
    None,
  )
  .inspect_err(|error| eprintln!("skipping {test}: vmnet made no network: {error}"))
  .ok()
}

#[test]
fn hands_out_interfaces_from_its_subnet() {
  let Some(mut network) = vmnet_network("hands_out_interfaces_from_its_subnet") else {
    return;
  };
  let subnet = network.subnet();
  let gateway = network.ipv4_gateway();

  assert_eq!(subnet.gateway().expect("Swift finds the subnet's gateway"), gateway);

  let interface = network
    .create_interface("first")
    .expect("an interface should be created")
    .expect("a vmnet network always makes an interface");

  assert!(subnet.contains(interface.ipv4_address().address()).unwrap());
  assert_eq!(interface.ipv4_address().prefix(), subnet.prefix());
  assert_eq!(interface.ipv4_gateway(), Some(gateway));
  assert_eq!(interface.ipv6_gateway(), network.ipv6_gateway());
  assert_eq!(network.ipv6_gateway().is_some(), network.prefix_v6().is_some());
  assert_eq!(
    interface.ipv6_address().is_some(),
    network.prefix_v6().is_some(),
    "an interface has an IPv6 address when its network has a prefix"
  );
  assert_eq!(interface.mac_address(), None, "the guest picks its own");
  assert_eq!(interface.mtu(), 1500);
  assert_eq!(interface.clone(), interface);
  assert!(
    network.create_interface("first").is_err(),
    "an id holds one allocation at a time"
  );

  let without_gateway = network
    .create_interface_without_gateway("second")
    .expect("an interface should be created")
    .expect("a vmnet network always makes an interface");

  assert_eq!(without_gateway.ipv4_gateway(), None);
  assert_ne!(without_gateway.ipv4_address(), interface.ipv4_address());

  let with_mtu = network
    .create_interface_with_mtu("third", 1400)
    .expect("an interface should be created")
    .expect("a vmnet network always makes an interface");

  assert_eq!(with_mtu.mtu(), 1400);

  network
    .release_interface("first")
    .expect("the interface should release");
  assert!(
    network.create_interface("first").is_ok(),
    "a released id can be allocated again"
  );
}

#[test]
fn routes_a_container_through_its_network() {
  let Some(network) = vmnet_network("routes_a_container_through_its_network") else {
    return;
  };
  let gateway = network.ipv4_gateway();
  let booted = Container::boot_on("cfw-test-manager-vmnet", network);
  let interfaces = booted.container().interfaces();

  let [cfw::containerization::network::Interface::Vmnet(interface)] = interfaces.as_slice() else {
    panic!("the manager should give the container one vmnet interface, not {interfaces:?}");
  };
  assert_eq!(interface.ipv4_gateway(), Some(gateway));
  assert_eq!(
    booted.container().config().dns.map(|dns| dns.nameservers),
    Some(vec![gateway.description().unwrap()]),
    "the manager resolves through the gateway"
  );

  let routes = booted.sh("routes", "ip route");

  assert!(
    routes.contains(&format!("default via {}", gateway.description().unwrap())),
    "the guest should route through the gateway: {routes}"
  );
}

#[test]
fn reaches_its_store_and_creates_from_a_reference() {
  let store = support::store::image_store();
  let _ = support::store::image(&store);
  let mut manager = support::store::manager(&store);
  let name = "cfw-test-manager-reference";

  assert_eq!(manager.image_store().path(), store.path());

  // Left by a run that died before deleting its container.
  let _ = manager.delete(name);

  let events = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
  let counted = std::sync::Arc::clone(&events);
  let options = cfw::containerization::container::container_manager::CreateOptions {
    rootfs_size_in_bytes: support::TEST_ROOTFS_SIZE_IN_BYTES,
    networking: false,
    progress: Some(Box::new(move |batch| {
      counted.fetch_add(batch.len(), std::sync::atomic::Ordering::Relaxed);
    })),
    ..Default::default()
  };

  let container = manager
    .create_from_reference(name, support::store::IMAGE, options, |_| {})
    .expect("the manager should create from a reference it has");

  assert_eq!(container.id(), name);
  assert!(
    events.load(std::sync::atomic::Ordering::Relaxed) > 0,
    "unpacking the rootfs reports progress"
  );
  manager
    .release_network(name)
    .expect("a manager with no network has nothing to release");
  manager.delete(name).expect("the container should delete");
}

#[test]
fn opens_a_store_at_its_root() {
  let kernel = support::store::kernel();
  let _lock = support::store::initfs_lock();

  let manager = cfw::containerization::container::ContainerManager::at_root_with_initfs_reference(
    &kernel,
    support::store::INITFS_REFERENCE,
    Some(&support::store::root()),
    Default::default(),
  )
  .expect("a manager should open the suite's store");

  assert_eq!(manager.image_store().path(), support::store::image_store().path());
}
