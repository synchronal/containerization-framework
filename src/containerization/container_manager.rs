//! `ContainerManager`, and [`CreateOptions`] for its `create`.

use super::GIB;
use super::Image;
use super::ImageStore;
use super::Kernel;
use super::LinuxContainer;
use super::Mount;
use super::VmResources;
use super::linux_container;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;

/// `ContainerManager.create(_:image:...)`'s defaulted arguments, between the
/// image and the configuration. [`Default`] is Swift's defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CreateOptions {
  pub rootfs_size_in_bytes: u64,
  pub writable_layer_size_in_bytes: Option<u64>,
  pub read_only: bool,
  pub networking: bool,
  pub vm: VmResources,
}

impl Default for CreateOptions {
  fn default() -> Self {
    Self {
      rootfs_size_in_bytes: 8 * GIB,
      writable_layer_size_in_bytes: None,
      read_only: false,
      networking: true,
      vm: VmResources::default(),
    }
  }
}

/// `ContainerManager`, with no `Network`.
pub struct ContainerManager {
  handle: ffi::CzContainerManager,
}

// Swift's `ContainerManager` is `Sendable`; `create` and `delete` take
// `&mut self` as Swift's are `mutating`.
unsafe impl Send for ContainerManager {}
unsafe impl Sync for ContainerManager {}

impl ContainerManager {
  /// `ContainerManager(kernel:initfs:imageStore:rosetta:nestedVirtualization:)`.
  pub fn new(
    kernel: &Kernel,
    initfs: &Mount,
    image_store: &ImageStore,
    rosetta: bool,
    nested_virtualization: bool,
  ) -> Result<Self, Error> {
    let outcome = platform::outcome(
      image_store
        .handle
        .container_manager(kernel.clone(), initfs.clone(), rosetta, nested_virtualization),
      "make a container manager",
    )?;

    Ok(Self {
      handle: outcome.container_manager(),
    })
  }

  /// `ContainerManager(kernel:initfsReference:imageStore:rosetta:nestedVirtualization:)`.
  pub fn with_initfs_reference(
    kernel: &Kernel,
    initfs_reference: &str,
    image_store: &ImageStore,
    rosetta: bool,
    nested_virtualization: bool,
  ) -> Result<Self, Error> {
    let outcome = platform::outcome(
      image_store.handle.container_manager_with_initfs_reference(
        kernel.clone(),
        initfs_reference,
        rosetta,
        nested_virtualization,
      ),
      format!("make a container manager booting {initfs_reference}"),
    )?;

    Ok(Self {
      handle: outcome.container_manager(),
    })
  }

  /// `ContainerManager.create(_:image:rootfsSizeInBytes:writableLayerSizeInBytes:readOnly:networking:vm:configuration:)`.
  ///
  /// `configuration` runs on another thread, as Swift's closure does, against
  /// the configuration the manager seeded.
  pub fn create(
    &mut self,
    id: &str,
    image: &Image,
    options: CreateOptions,
    configuration: impl FnOnce(&mut linux_container::Configuration) + Send + 'static,
  ) -> Result<LinuxContainer, Error> {
    let outcome = self.handle.create(
      id,
      image.handle.duplicate(),
      options,
      linux_container::Configuration::default(),
      Box::new(move |mut seeded: linux_container::Configuration| {
        configuration(&mut seeded);
        seeded
      }),
    );

    platform::outcome(outcome, format!("create {id}")).map(|outcome| LinuxContainer {
      handle: outcome.linux_container(),
    })
  }

  /// `ContainerManager.delete(_:)`.
  pub fn delete(&mut self, id: &str) -> Result<(), Error> {
    platform::outcome(self.handle.delete(id), format!("delete {id}")).map(|_| ())
  }
}
