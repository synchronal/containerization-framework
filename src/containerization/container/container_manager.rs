//! `ContainerManager`, the options for its initializers:
//! [`ContainerManagerOptions`], and the options for its `create`s:
//! [`CreateOptions`] and [`CreateWithRootfsOptions`].

use super::LinuxContainer;
use super::Mount;
use super::linux_container;
use crate::containerization::GIB;
use crate::containerization::image;
use crate::containerization::network;
use crate::containerization::vm;
use crate::containerization_extras;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::fmt;
use std::path::Path;

/// The defaulted arguments of `ContainerManager`'s initializers, after the
/// image store or root. [`Default`] is Swift's defaults.
#[derive(Debug, Default)]
pub struct ContainerManagerOptions {
  /// Swift's `Network?`, of which `VmnetNetwork` is the conforming type on
  /// macOS.
  pub network: Option<network::VmnetNetwork>,
  pub rosetta: bool,
  pub nested_virtualization: bool,
}

/// `ContainerManager.create(_:image:...)`'s defaulted arguments, between the
/// image and the configuration, which `create(_:reference:...)` shares.
/// [`Default`] is Swift's defaults.
pub struct CreateOptions {
  pub rootfs_size_in_bytes: u64,
  pub writable_layer_size_in_bytes: Option<u64>,
  pub read_only: bool,
  pub networking: bool,
  pub vm: vm::VMResources,
  pub progress: Option<containerization_extras::ProgressHandler>,
}

impl Default for CreateOptions {
  fn default() -> Self {
    Self {
      rootfs_size_in_bytes: 8 * GIB,
      writable_layer_size_in_bytes: None,
      read_only: false,
      networking: true,
      vm: vm::VMResources::default(),
      progress: None,
    }
  }
}

/// Leaves out the progress handler, which is a closure.
impl fmt::Debug for CreateOptions {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("CreateOptions")
      .field("rootfs_size_in_bytes", &self.rootfs_size_in_bytes)
      .field("writable_layer_size_in_bytes", &self.writable_layer_size_in_bytes)
      .field("read_only", &self.read_only)
      .field("networking", &self.networking)
      .field("vm", &self.vm)
      .finish_non_exhaustive()
  }
}

/// `ContainerManager.create(_:image:rootfs:...)`'s defaulted arguments, between
/// the rootfs and the configuration. [`Default`] is Swift's defaults.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateWithRootfsOptions {
  pub writable_layer: Option<Mount>,
  pub networking: bool,
  pub vm: vm::VMResources,
}

impl Default for CreateWithRootfsOptions {
  fn default() -> Self {
    Self {
      writable_layer: None,
      networking: true,
      vm: vm::VMResources::default(),
    }
  }
}

/// `ContainerManager`.
pub struct ContainerManager {
  handle: ffi::CzContainerManager,
}

impl fmt::Debug for ContainerManager {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("ContainerManager")
      .finish_non_exhaustive()
  }
}

// Swift's `ContainerManager` is `Sendable`; `create`, `release_network` and
// `delete` take `&mut self` as Swift's are `mutating`.
unsafe impl Send for ContainerManager {}
unsafe impl Sync for ContainerManager {}

impl ContainerManager {
  /// `ContainerManager(kernel:initfs:imageStore:network:rosetta:nestedVirtualization:)`.
  pub fn new(
    kernel: &vm::Kernel,
    initfs: &Mount,
    image_store: &image::ImageStore,
    options: ContainerManagerOptions,
  ) -> Result<Self, Error> {
    Self::made(
      image_store.handle.container_manager(
        kernel.clone(),
        initfs.clone(),
        network_crossing(options.network),
        options.rosetta,
        options.nested_virtualization,
      ),
      "make a container manager",
    )
  }

  /// `ContainerManager(kernel:initfsReference:imageStore:network:rosetta:nestedVirtualization:)`.
  pub fn with_initfs_reference(
    kernel: &vm::Kernel,
    initfs_reference: &str,
    image_store: &image::ImageStore,
    options: ContainerManagerOptions,
  ) -> Result<Self, Error> {
    Self::made(
      image_store.handle.container_manager_with_initfs_reference(
        kernel.clone(),
        initfs_reference,
        network_crossing(options.network),
        options.rosetta,
        options.nested_virtualization,
      ),
      format!("make a container manager booting {initfs_reference}"),
    )
  }

  /// `ContainerManager(kernel:initfs:root:network:rosetta:nestedVirtualization:)`.
  /// A `None` root is `ImageStore.default`'s.
  pub fn at_root(
    kernel: &vm::Kernel,
    initfs: &Mount,
    root: Option<&Path>,
    options: ContainerManagerOptions,
  ) -> Result<Self, Error> {
    Self::made(
      ffi::cz_container_manager_at_root(
        kernel.clone(),
        initfs.clone(),
        root.map(platform::path).transpose()?.map(str::to_string),
        network_crossing(options.network),
        options.rosetta,
        options.nested_virtualization,
      ),
      "make a container manager",
    )
  }

  /// `ContainerManager(kernel:initfsReference:root:network:rosetta:nestedVirtualization:)`.
  /// A `None` root is `ImageStore.default`'s.
  pub fn at_root_with_initfs_reference(
    kernel: &vm::Kernel,
    initfs_reference: &str,
    root: Option<&Path>,
    options: ContainerManagerOptions,
  ) -> Result<Self, Error> {
    Self::made(
      ffi::cz_container_manager_at_root_with_initfs_reference(
        kernel.clone(),
        initfs_reference,
        root.map(platform::path).transpose()?.map(str::to_string),
        network_crossing(options.network),
        options.rosetta,
        options.nested_virtualization,
      ),
      format!("make a container manager booting {initfs_reference}"),
    )
  }

  /// `ContainerManager(vmm:network:logger:)`, on `ImageStore.default`.
  pub fn with_vmm(vmm: &vm::VZVirtualMachineManager, network: Option<network::VmnetNetwork>) -> Result<Self, Error> {
    Self::made(
      ffi::cz_container_manager_with_vmm(vmm.handle.duplicate(), network_crossing(network)),
      "make a container manager",
    )
  }

  fn made(outcome: ffi::CzOutcome, action: impl Into<String>) -> Result<Self, Error> {
    platform::outcome(outcome, action).map(|outcome| Self {
      handle: outcome.container_manager(),
    })
  }

  /// `ContainerManager.imageStore`.
  pub fn image_store(&self) -> image::ImageStore {
    image::ImageStore {
      handle: self.handle.image_store(),
    }
  }

  /// `ContainerManager.create(_:reference:rootfsSizeInBytes:writableLayerSizeInBytes:readOnly:networking:vm:progress:configuration:)`.
  /// The manager's store pulls the image if it doesn't have it.
  ///
  /// `configuration` runs as [`Self::create`]'s does.
  pub fn create_with_reference(
    &mut self,
    id: &str,
    reference: &str,
    mut options: CreateOptions,
    configuration: impl FnOnce(&mut linux_container::Configuration) -> Result<(), Error> + Send + 'static,
  ) -> Result<LinuxContainer, Error> {
    let progress = platform::Progress(options.progress.take());

    platform::configured(
      configuration,
      |configure| {
        self.handle.create_from_reference(
          id,
          reference,
          options,
          progress,
          linux_container::Configuration::default(),
          configure,
        )
      },
      format!("create {id} from {reference}"),
    )
    .map(|outcome| LinuxContainer {
      handle: outcome.linux_container(),
    })
  }

  /// `ContainerManager.create(_:image:rootfsSizeInBytes:writableLayerSizeInBytes:readOnly:networking:vm:progress:configuration:)`.
  ///
  /// `configuration` runs on another thread, as Swift's closure does, against
  /// the configuration the manager seeded. An error it returns is thrown and
  /// returned by `create`.
  pub fn create(
    &mut self,
    id: &str,
    image: &image::Image,
    mut options: CreateOptions,
    configuration: impl FnOnce(&mut linux_container::Configuration) -> Result<(), Error> + Send + 'static,
  ) -> Result<LinuxContainer, Error> {
    let progress = platform::Progress(options.progress.take());

    platform::configured(
      configuration,
      |configure| {
        self.handle.create(
          id,
          image.handle.duplicate(),
          options,
          progress,
          linux_container::Configuration::default(),
          configure,
        )
      },
      format!("create {id}"),
    )
    .map(|outcome| LinuxContainer {
      handle: outcome.linux_container(),
    })
  }

  /// `ContainerManager.create(_:image:rootfs:writableLayer:networking:vm:configuration:)`.
  ///
  /// `configuration` runs on another thread, as Swift's closure does, against
  /// the configuration the manager seeded. Swift seeds a boot log in the
  /// manager's directory for `id`, but only the image `create` makes that
  /// directory, so set `boot_log` to somewhere that exists.
  pub fn create_with_rootfs(
    &mut self,
    id: &str,
    image: &image::Image,
    rootfs: Mount,
    options: CreateWithRootfsOptions,
    configuration: impl FnOnce(&mut linux_container::Configuration) -> Result<(), Error> + Send + 'static,
  ) -> Result<LinuxContainer, Error> {
    platform::configured(
      configuration,
      |configure| {
        self.handle.create_with_rootfs(
          id,
          image.handle.duplicate(),
          rootfs,
          options,
          linux_container::Configuration::default(),
          configure,
        )
      },
      format!("create {id}"),
    )
    .map(|outcome| LinuxContainer {
      handle: outcome.linux_container(),
    })
  }

  /// `ContainerManager.releaseNetwork(_:)`.
  pub fn release_network(&mut self, id: &str) -> Result<(), Error> {
    platform::outcome(self.handle.release_network(id), format!("release {id}'s network")).map(drop)
  }

  /// `ContainerManager.delete(_:)`.
  pub fn delete(&mut self, id: &str) -> Result<(), Error> {
    platform::outcome(self.handle.delete(id), format!("delete {id}")).map(drop)
  }
}

/// A `Network?`, as it crosses to Swift.
fn network_crossing(network: Option<network::VmnetNetwork>) -> ffi::CzNetwork {
  match network {
    Some(network) => network.handle.as_network(),
    None => ffi::cz_no_network(),
  }
}
