# Changelog

## Unreleased

- Every public type implements `Debug`.
- `VZVirtualMachineInstance::dial_agent` returns the new
  `containerization::vm::Vminitd`, the client of the agent running in the
  guest. It can set up the guest, manage processes, mount filesystems,
  configure networking, DNS and hosts, stat paths, relay sockets and read
  container statistics. Its `writeFile` and `copy` aren't bound, because
  Swift's `WriteFileFlags` has no public initializer and `copy` takes a
  generated protobuf type.
- `containerization_extras` has `InterfaceAddress`, `LinkRoute` and
  `DefaultRoute`, which `Vminitd` configures an interface with.
- `containerization_os::linux::binfmt` has `Entry`, which `Vminitd::setup_emulator`
  registers in the guest.
- `containerization_ext4::ext4::FileXattrsState::read` binds Swift's
  `EXT4.FileXattrsState.read(buffer:start:offset:)`. The rest of
  `FileXattrsState` isn't bound, because Swift's initializer is internal and
  nothing public returns one.

### Breaking changes

- `CapabilityName`, `CapabilitySet` and `binfmt` have moved from the root of
  `containerization_os` to `containerization_os::linux`, as they are in
  Swift's `Linux` directory.
- `ExtendedAttribute` no longer implements `PartialEq` or `Eq`. Swift's type
  isn't `Equatable`, and since its fields are internal, every value compared
  equal to every other.
- Types that mirror a Swift type now take Swift's exact name.
  `BlockIoDevice`, `BlockIoStatistics`, `Cidr`, `CpuStatistics`, `Dns`,
  `Ext4Reader`, `Ext4Unpacker`, `IpAddress`, `NatInterface`,
  `VmConfiguration`, `VmResources`, `VzVirtualMachineInstance`,
  `VzVirtualMachineManager` and `system_platform::Os` are now
  `BlockIODevice`, `BlockIOStatistics`, `CIDR`, `CPUStatistics`, `DNS`,
  `EXT4Reader`, `EXT4Unpacker`, `IPAddress`, `NATInterface`,
  `VMConfiguration`, `VMResources`, `VZVirtualMachineInstance`,
  `VZVirtualMachineManager` and `OS`.
- The structs that stand in for an initializer's default arguments are named
  after its type. `container_manager::ManagerOptions` is now
  `ContainerManagerOptions`, `vz_virtual_machine_manager::ManagerOptions` is
  `VZVirtualMachineManagerOptions`, `reference::NewOptions` is
  `ReferenceOptions`, `registry_client::HostOptions` is
  `RegistryClientOptions`, and `archive_writer::FileOptions` is
  `ArchiveWriterOptions`. A method's struct is named after the method, so
  `RootfsCreateOptions` is now `CreateWithRootfsOptions`.
- `ContainerManager::create_from_reference` is now `create_with_reference`,
  and `LinuxCapabilities::with` is now `with_capabilities`.
- Configuration closures return `Result<(), Error>`, as Swift's can throw.
  An error the closure returns is the one the call returns. This covers
  `ContainerManager`'s `create` methods, `LinuxContainer::new_with` and
  `exec_with`, and `LinuxPod`'s `new`, `add_container` and
  `exec_in_container`.
- Off macOS, comparing two `Platform`s or ordering two addresses now panics,
  since only Swift can answer. Before, `Platform` compared every field, and
  addresses had no order.
- `Error` is `#[non_exhaustive]`, and `Error::unavailable` is no longer
  public. `Error` now implements `Clone`, `PartialEq` and `Eq`.
- `UnixSocketConfiguration` now holds the Swift value instead of public
  fields, so it keeps the `id` that Swift gives it. It has `id`, `source`,
  `destination`, `permissions` and `direction` methods, and a `set_` method
  for each field but `id`. `UnixSocketConfiguration::new` returns a `Result`,
  because it can't reach Swift off macOS. Permissions are a `u16`, as Swift's
  `CModeT` is. Two configurations are equal when their `id`s and other fields
  match, so a configuration equals only itself or an unchanged clone.
- `UnixSocketConfiguration::set_source` and `set_destination` now return
  `Result<(), Error>`, and `EXT4Reader::exists` now returns
  `Result<bool, Error>`. Each fails if its path isn't valid UTF-8 (see
  Fixes).

### Fixes

- A path that isn't valid UTF-8 now fails the call with `Error::Failed`
  before it reaches Swift. Before, its invalid bytes were replaced with
  U+FFFD, so a call such as `Bundle::delete` or
  `ArchiveReader::extract_contents` could act on a different file. A kernel,
  boot log or pod disk image path that isn't UTF-8 also fails the call that
  hands its configuration to Swift.
- A process's `exited_at` and a keychain `RegistryInfo`'s dates now keep
  dates before 1970 instead of becoming 1970.
- `VsockListener`'s iterator panics if the bridge fails. Before, the failure
  looked like the end of the connections.
- `Vminitd::stop_socket_relay` now stops the relay that `relay_socket` started
  with the same configuration. Before, each call made a new Swift
  configuration with a new `id`, so the guest never found the relay to stop.
  Sockets in a container's configuration also keep their `id`s when it
  crosses to Swift and back, as do those in a pod container's configuration
  when it crosses to Swift.
- The build script no longer rewrites the staged FFI glue on every run, so
  SwiftPM doesn't recompile an unchanged bridge. A change to
  `swift/Package.resolved` now reruns the build script.

## v0.4.0

This release binds most of the rest of Containerization's Swift API. Each
module's types are now grouped into submodules by what they work on, such as
`containerization::image` and `containerization::container`, and the entries
below name each new type's group. Where a Swift method has defaulted
arguments, the Rust method takes them in an options struct whose `Default`
matches Swift's, such as `image::image_store::PullOptions`.

### Images and content

- `ImageStore` can push images to a registry and save them to disk. `Image`
  exposes its descriptor, index, manifest and config, and `Image`,
  `InitImage` and the new `KernelImage` can be created directly.
- `LocalContentStore` supports ingest sessions.
- The new `containerization_oci::client` module has `RegistryClient`,
  `Authentication` and `KeychainHelper`. `RegistryClient` can't take Swift's
  TLS configuration or logger.

### Containers and processes

- `LinuxContainer` exposes what it was made with, such as its `config`,
  `rootfs` and `vm`. It can run a process configured by a closure with
  `exec_with`, report `ContainerStatistics`, freeze, thaw or trim a
  filesystem, copy files in and out, and dial a vsock port.
- `ContainerManager` can open a store at a root directory, create a container
  from an image reference, report unpacking progress, and release a
  container's network. Given a `VmnetNetwork`, it gives each container it
  creates an interface on that network.
- The new `LinuxPod` runs several containers in one VM, and they can share
  the pod's volumes.
- The configuration types have the rest of their Swift API, such as
  `LinuxProcessConfiguration::from_image_config`, `Mount::tag_hash`,
  `Dns::resolv_conf` and `Hosts::hosts_file`. `LinuxProcess` exposes its
  `owning_container`.
- `Signal` can be parsed from a name or a number, and every Linux and Darwin
  signal is a constant in `signal::linux` and `signal::darwin`.

### Virtual machines

- The new `containerization::vm::VzVirtualMachineManager` boots VMs without a
  container manager. The `VzVirtualMachineInstance` it creates can be
  started, paused, dialed over vsock and listened on.
- A `LinuxContainer` can be made directly on a VM manager, and its VM can be
  reached with `with_virtual_machine_instance`. `ContainerManager::with_vmm`
  makes a container manager on one.

### OCI types

- `containerization_oci::image` has the image format's types, such as
  `Index`, `Manifest` and `Reference`. Its `Image` is an image's config, not
  `containerization::image::Image`.
- `containerization_oci::runtime` has the runtime spec, such as `Spec`,
  `State` and `Bundle`. `LinuxSeccomp::decode` reads a seccomp profile, and
  rejects Docker's profile format rather than turning its conditional rules
  into unconditional allows.
- `containerization_oci::content` has `ParsedDigest`.
- `Platform` can be parsed, printed and matched. Its `==` follows Swift's, so
  an `arm64` platform with no variant equals `arm64/v8`, and the OS version
  and features are ignored. Off macOS, every field must match.

### Other modules

- `containerization_extras` has `proxy_utils::proxy_from_environment`, and
  its `address` module has network address types such as `IPv4Address`,
  `CIDRv4` and `MACAddress`. Swift does the parsing, so these calls return a
  `Result`. `ProgressEvent` exposes its event and value.
- `containerization_os` has `Terminal`, `CapabilityName`, `CapabilitySet`,
  `Stat`, `sysctl::by_name` and `file::info`, and its `keychain` module has
  `KeychainQuery`.
- `ext4::Formatter` builds an ext4 filesystem file by file, or unpacks an
  archive into one. `Ext4Reader` can read the superblock, stat paths, list
  directories and read files.
- The new `containerization_archive` module writes, reads and extracts
  archives, and `Ext4Unpacker::unpack_archive` unpacks an archive into an
  ext4 filesystem.
- The new `containerization_io` module has `ReadStream`, whose `data_stream`
  is an iterator over the chunks it reads.
- `Error::Failed` carries the code of a thrown `ContainerizationError`, which
  `Error::is_code` checks.

### Fixes

- `LocalContentStore::ingest` no longer intermittently crashes after its
  closure returns.

### Breaking

- The types that were at the root of `containerization` and
  `containerization_oci` have moved into the new groups, along with the
  modules for their nested types:
  - `containerization::image` has `Image`, `ImageStore`, `InitImage` and
    `Ext4Unpacker`, so `containerization::ImageStore` is now
    `containerization::image::ImageStore`. `image::Description` hasn't moved.
  - `containerization::vm` has `Kernel`, `SystemPlatform`, `BootLog` and
    `VmResources`, and the `kernel` and `system_platform` modules.
  - `containerization::network` has `Dns`, `Hosts`, `NatInterface` and the
    `hosts` module.
  - `containerization::container` has `ContainerManager`, `LinuxContainer`,
    `Mount` and `UnixSocketConfiguration`, and the `container_manager`,
    `linux_container`, `mount` and `unix_socket_configuration` modules. So
    `containerization::linux_container::Configuration` is now
    `containerization::container::linux_container::Configuration`.
  - `containerization::process` has `LinuxProcess`,
    `LinuxProcessConfiguration`, `LinuxCapabilities`, `LinuxRLimit`, `Signal`
    and `ExitStatus`, and the `linux_rlimit` module.
  - In `containerization_oci`, `Descriptor` and `Platform` are in `image`,
    `Content`, `ContentWriter` and `LocalContentStore` are in `content`, and
    `User` is in `runtime`.
- `ImageStore::pull` takes a `PullOptions`, and `ImageStore::get_init_image`
  takes `auth` and `progress` arguments. Pass `Default::default()` or `None`
  to keep the old behavior.
- `Mount::share`, `Mount::block` and `Mount::any` take a `runtime_options`
  argument after `options`. Pass `&[]` to keep the old behavior.
- `linux_container::Configuration::interfaces` holds `Interface`s, so wrap a
  `NatInterface` in `Interface::Nat`. `NatInterface` holds parsed address
  types instead of strings, so a malformed address fails when you parse it
  rather than when the container is created. `NatInterface::new` takes its
  IPv4 gateway as an `Option`, as Swift does.
- `SeccompProfile::Profile` holds a `LinuxSeccomp` instead of JSON. Read the
  JSON with `LinuxSeccomp::decode`.
- `Error::Failed` has a new `code` field, so a pattern that names its fields
  needs `..`.
- `LinuxCapabilities`' sets hold `CapabilityName`s instead of strings, so a
  misspelled capability is a compile error.
- `ContainerManager::new` and `ContainerManager::with_initfs_reference` take
  a `ManagerOptions` in place of `rosetta` and `nested_virtualization`. Pass
  `Default::default()` to keep the old behavior. `CreateOptions` holds an
  optional `ProgressHandler`, so it is no longer `Copy`, `Clone`, `Debug` or
  `PartialEq`.

## v0.3.0

- The Rust API now mirrors Containerization's Swift API. Modules are named
  after the Swift modules (`containerization`, `containerization_oci`,
  `containerization_os`), types after the Swift types, and methods after their
  Swift methods. A Swift type nested in another, like
  `LinuxContainer.Configuration`, is found in a module named after its parent:
  `linux_container::Configuration`.
- Added wrappers for `ImageStore`, `Image`, `InitImage`, `Kernel`,
  `SystemPlatform`, `ContainerManager`, `LinuxContainer`, `LinuxProcess`,
  `ExitStatus`, `Signal`, `LocalContentStore` and `Content`.
- Processes can now be given rlimits, capabilities, `noNewPrivileges` and a
  terminal. Mounts can use `Mount.RuntimeOptions.shared`, and a boot log can
  write to a file descriptor.
- `ContainerManager::create` takes Swift's optional arguments in a
  `container_manager::CreateOptions` struct, whose defaults match Swift's. Its
  closure receives the configuration the manager prepared and can change it.
- Unpack an image once with `Ext4Unpacker::unpack`, then boot containers from
  copies of it with `ContainerManager::create_with_rootfs`. `boot_log` must
  point at an existing directory, because Swift doesn't create it.
- Turn an ext4 filesystem back into a tar archive with `Ext4Reader::export`.
- Write your own blobs into a store with `LocalContentStore::ingest` and
  `ContentWriter`. If your closure fails, nothing is added.
- Add images without pulling them: `ImageStore::create` tags blobs already in
  the store, and `ImageStore::load` imports an OCI layout directory.
  `ImageStore::with_content_store` lets an image store share a content store.
- Unpacking and loading accept a `ProgressHandler` to report progress.
- New supporting types: `Platform`, `Descriptor`, `image::Description` and
  `JournalConfig`.

### Breaking

The types this crate built on top of Containerization have been removed. Use
the Containerization types they were built from.

- `Session` is removed. Create containers with `ContainerManager::create`,
  then call `create`, `start` and `exec` on the `LinuxContainer` it returns.
  `exec` returns a `LinuxProcess`, which you `start` and `wait` on. There is
  no replacement for `is_running` or `is_unpacked`; keep the `LinuxContainer`
  you created. Each container now unpacks its own copy of the image, as
  Containerization does, instead of cloning one shared copy.
- `Session::version` and the kernel and init image constants (`KERNEL_URL`,
  `KERNEL_IN_ARCHIVE`, `KERNEL_VERSION`, `INITFS_REFERENCE`,
  `INITFS_VERSION`) are removed. Callers choose the kernel and init image
  themselves; the init image must match Containerization 0.48.0.
- `Builder`, `BuildPlan`, `BuildStep`, `Shell` and `CachePolicy` are removed,
  with no replacement. This crate no longer builds images or downloads
  kernels.
- `Store` and `StoreError` are replaced by `ImageStore`.
- `BootSpec` is replaced by the arguments to `ContainerManager::create`.
- `Stdio`, `lend`, `is_tty` and `UNATTACHED` are removed. Pass file
  descriptors in `LinuxProcessConfiguration`'s `stdin`, `stdout` and `stderr`.
- The `model` module is now `containerization`. Some types moved with their
  Swift names: `LinuxContainerConfiguration` is now
  `linux_container::Configuration`, `HostsEntry` is `hosts::Entry`,
  `RuntimeOptions` is `mount::RuntimeOptions`, `Direction` is
  `unix_socket_configuration::Direction`, and `User` is
  `containerization_oci::User`.
- `LinuxProcessConfiguration` now has the same fields and defaults as Swift's.
  `arguments`, `working_directory` and `user` are no longer `Option`s, and the
  environment defaults to just `PATH`. Only the container's first process
  starts from the image's settings, because `ContainerManager` fills them in.
- Processes started with `exec` no longer get `TERM=xterm`, or the image's
  user and environment. Set them in the process's configuration.
- DNS nameservers are no longer checked for hostnames before a container
  starts.
- `BootLog` is now an enum: `File { path, append }` or `FileHandle`.

## v0.2.2

- Update Containerization to 0.48.0

## v0.2.1

- Off macOS, errors name what was attempted (`cannot boot session-one:
  Containerization.framework is macOS only`) rather than a generic action
- Lend FFI accessors instead of returning owned clones.

## v0.2.0

- Update Containerization to 0.47.0
- Update configuration to match Containerization's types, names, and
  defaults in `model`. New: hostname, sysctls, full DNS, `/etc/hosts`, masked
  and read-only paths, init process, nested virtualization, boot log, OCI
  runtime, seccomp; block and `tmpfs` mounts; multiple interfaces with IPv6,
  MAC and MTU; process user as uid/gid/umask/groups
- The caller chooses where the kernel and unpacked init image live, and which
  init image to boot. The pins (`KERNEL_URL`, `KERNEL_IN_ARCHIVE`,
  `KERNEL_VERSION`, `INITFS_REFERENCE`, `INITFS_VERSION`) are public on every
  platform
- The model crosses to Swift as swift-bridge opaque types, not JSON

### Breaking

Container and process settings now use Containerization's types and defaults,
so code that boots containers or runs processes need updating.

#### Changes with no compile errors

- **Containers no longer get DNS automatically.** Previously every container
  resolved names through the network gateway. Now nothing is written to
  `/etc/resolv.conf` unless you set `configuration.dns`. To keep the old
  behavior, set it to your gateway:
  `Some(Dns { nameservers: vec![gateway.into()], ..Dns::default() })`.
- **VMs are no longer given extra room.** Previously the VM got one more CPU
  and 128 MiB more memory than the container's limits. Now the VM is exactly
  `vm`, which defaults to 4 CPUs and 1 GiB. If you size the VM to match the
  container, add `VmResources::GUEST_MEMORY_OVERHEAD` to its memory so the
  guest kernel has room.
- **Unset process settings come from the image.** A process with no arguments
  now runs the image's entrypoint and command, and one with no working
  directory starts in the image's, not `/`.
- **Relayed sockets are no longer world-writable by default.** Set
  `permissions: Some(0o666)` to keep the old mode.
- **Missing images are pulled.** Booting an image the store doesn't hold used to
  fail; now it downloads it.

#### Creating a store

`Store::at` now also takes the kernel's path, the init image's reference, and
where to unpack it; this crate no longer chooses them. Provisioning fills empty
paths and leaves existing files alone, so name paths for the pinned versions to
pick up upgrades:

```rust
let store = Store::at(
    &root,
    root.join(format!("vmlinux-{KERNEL_VERSION}")),
    INITFS_REFERENCE,
    root.join(format!("vminit-{INITFS_VERSION}.ext4")),
);
```

To reuse an existing store's files, pass `root.join("kernels/default.kernel-arm64")`
and `root.join("initfs.ext4")`.

#### Booting a container

`BootSpec::new` now takes only an id and an image; everything else is set on
its fields, which start with Containerization's defaults.

Before:

```rust
let spec = BootSpec {
    arguments: vec!["/bin/sleep".into(), "infinity".into()],
    ..BootSpec::new("example", "debian", Resources { cpus: 2, memory_in_bytes: 1 << 30 }, network)
};
```

After:

```rust
let mut spec = BootSpec::new("example", "debian");
spec.configuration.cpus = 2;
spec.configuration.memory_in_bytes = 1 << 30;
spec.vm = VmResources { cpus: 2, memory_in_bytes: (1 << 30) + VmResources::GUEST_MEMORY_OVERHEAD };
spec.configuration.process = LinuxProcessConfiguration::new(&["/bin/sleep", "infinity"]);
spec.configuration.interfaces = vec![NatInterface::new("192.168.64.7/24", "192.168.64.1")];
spec.configuration.dns = Some(Dns { nameservers: vec!["192.168.64.1".into()], ..Dns::default() });
```

Where each old field went:

| Before | After |
|---|---|
| `name` | `id` |
| `image` | `reference` |
| `rootfs_capacity_in_bytes` | `rootfs_size_in_bytes` |
| `resources` | `configuration.cpus` and `configuration.memory_in_bytes` for the container, `vm` for the VM |
| `arguments`, `environment`, `workdir` | `configuration.process` |
| `network` | `configuration.interfaces`, a list |
| `mounts` | `configuration.mounts` |
| `sockets` | `configuration.sockets` |

`configuration.mounts` starts with the standard mounts (`/proc`, `/dev`, ...);
add yours with `push`, don't replace the list. A read-only share that was
`Mount { readonly: true, source, target }` is now
`Mount::share(source, target, &["ro"])`.

### Running a process

`ExecRequest` is gone. `Session::exec` takes the container's name, a process
id, the same `LinuxProcessConfiguration` used for a container's first process,
and the stdio:

```rust
session.exec("example", "hello", &LinuxProcessConfiguration::new(&["/bin/echo", "hello"]), stdio)?;
```

- `user` is now a `User` rather than a string; `User::named("app")` does what
  `Some("app".into())` did.
- `term` is gone. A process on a terminal gets `TERM=xterm` unless you set
  `TERM` in its environment.

#### Building an image

`BuildPlan::new(name, base, tag, interface, base_key)` no longer takes
resources. Set `cpus`, `memory_in_bytes` and `vm` on the plan as for a
container. `network` is now `interface`, `rootfs_capacity_in_bytes` is
`rootfs_size_in_bytes`, and `mounts` holds `Mount`s (`BuildMount` is gone).

#### Moved types

`Mount`, `Direction` and the new configuration types live in `model`, not at the
crate root. `ExecRequest`, `Network`, `Resources`, `SocketRelay` and
`BuildMount` are gone; the table above shows their replacements.

## v0.1.0

- Initial release
