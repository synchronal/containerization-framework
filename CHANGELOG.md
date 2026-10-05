# Changelog

## Unreleased

Where a Swift method has defaulted arguments, the Rust method takes them in an
options struct whose `Default` matches Swift, such as
`image_store::PullOptions`.

- Added the address types from `ContainerizationExtras`, such as
  `IPv4Address`, `CIDRv4` and `MACAddress`, along with
  `proxy_utils::proxy_from_environment`. Swift does all the parsing, so these
  calls return a `Result`.
- `Error::Failed` now carries the code of a thrown `ContainerizationError`,
  which you can check with `Error::is_code`.
- Added the OCI image types, including `Index`, `Manifest`, `Reference` and
  `ParsedDigest`. `containerization_oci::Image` is an image's config, and is a
  different type from `containerization::Image`.
- `Platform` can be parsed, printed and matched, and its `==` now follows
  Swift's: an `arm64` platform with no variant equals `arm64/v8`, and the OS
  version and features are ignored. Off macOS, every field must match.
- Added the OCI runtime spec, including `Spec`, `State` and `Bundle`, and
  conversions into it such as `Process::from_image_config`. Seccomp profiles
  are read with `LinuxSeccomp::decode`, which rejects Docker's profile format
  instead of silently turning its conditional rules into unconditional allows.
- Images can be pushed to registries and saved to disk through `ImageStore`.
  `RegistryClient`, `Authentication` and `KeychainHelper` are also available,
  although `RegistryClient` can't take Swift's TLS configuration or logger.
- `Image` exposes its descriptor, index, manifest and config. `Image`,
  `InitImage` and the new `KernelImage` can be created directly.
- `LocalContentStore` supports ingest sessions, and `ProgressEvent` exposes its
  event and value.
- Archives can be written, read and extracted through the new
  `containerization_archive` module, and `Ext4Unpacker::unpack_archive`
  unpacks one into an ext4 filesystem.

### Breaking

- `ImageStore::pull` takes a `PullOptions`, and `ImageStore::get_init_image`
  takes `auth` and `progress` arguments. Pass `Default::default()` or `None`
  to keep the old behavior.
- `Mount::share`, `Mount::block` and `Mount::any` take a `runtime_options`
  argument after `options`. Pass `&[]` to keep the old behavior.
- `NatInterface` holds parsed address types instead of strings, so a malformed
  address fails when you parse it rather than when the container is created.
  Its IPv4 gateway is now optional, as it is in Swift.
- `SeccompProfile::Profile` holds a `LinuxSeccomp` instead of JSON. Read the
  JSON with `LinuxSeccomp::decode`.
- `Error::Failed` has a new `code` field, so a pattern that names its fields
  needs `..`.

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
