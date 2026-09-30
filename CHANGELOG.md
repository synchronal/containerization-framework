# Changelog

## Unreleased

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
