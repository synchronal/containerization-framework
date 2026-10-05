# containerization-framework

Rust bindings for Apple's [Containerization](https://github.com/apple/containerization)
framework: Linux containers.

The Rust API mirrors Containerization's Swift API as much as possible.
Modules are named after the Swift modules, types after the Swift types, and
methods after their Swift methods, except for using snake case. A Swift type
nested in another, like `LinuxContainer.Configuration`, is found in a module
named after its parent: `linux_container::Configuration`.

```rust
use containerization_framework as cfw;
use cfw::containerization as cz;

let store = cz::ImageStore::new("/Users/me/.cache/containers".as_ref())?;
let kernel = cz::Kernel::new("/Users/me/.cache/vmlinux", cz::SystemPlatform::LINUX_ARM);
let mut manager = cz::ContainerManager::with_initfs_reference(
    &kernel,
    "ghcr.io/apple/containerization/vminit:0.48.0",
    &store,
    false,
    false,
)?;

let image = store.get("docker.io/library/alpine:3", true)?;
let options = cz::container_manager::CreateOptions { networking: false, ..Default::default() };
let container = manager.create("example", &image, options, |config| {
    config.process.arguments = vec!["/bin/sleep".into(), "infinity".into()];
    config.interfaces = vec![cz::NatInterface::new("192.168.64.7/24", Some("192.168.64.1".into()))];
    config.dns = Some(cz::Dns { nameservers: vec!["192.168.64.1".into()], ..Default::default() });
})?;
container.create()?;
container.start()?;

let process = container.exec("hello", cz::LinuxProcessConfiguration::new(&["/bin/echo", "hello"]))?;
process.start()?;
let status = process.wait(None)?;
process.delete()?;

container.stop()?;
manager.delete("example")?;
```

Swift's `async` methods block until they finish, and errors they throw are
returned as `cfw::Error`. A container belongs to the process that created it,
and stops when that process exits.

## Requirements

- macOS 26 on Apple silicon, and Xcode 26 to build.
- Network on a first build: the build script compiles the bundled Swift package,
  which resolves Containerization and its dependencies through SwiftPM. Versions
  are pinned by the `Package.resolved` that ships with this crate.

On non-macOS platforms, this crate compiles but returns errors on every call.

## Codesigning

**A binary using this crate must carry the `com.apple.security.virtualization`
entitlement.** Without it Virtualization.framework refuses to start a VM, and
`LinuxContainer::create` fails.

A `containerization.entitlements` file ships with this crate; binaries compiled
against `containerization-framework` should pass it, or a copy of it, to `codesign`
after compilation.

```sh
cargo build --release
codesign --force --sign - --entitlements containerization.entitlements \
  target/release/your-binary
```

Signing ad hoc (`--sign -`) satisfies the entitlement but gives the binary a new
code identity on every rebuild, so anything keyed to that identity — Keychain
access, for one — prompts again. Sign with a development identity to keep it
stable.

A rebuild drops the signature, so this runs after every build.

## Linking

The Swift runtime this links against is dynamic and referenced as
`@rpath/libswift_Concurrency.dylib`, which dyld resolves against `/usr/lib/swift`
in macOS. Anything that links this crate — a binary of yours, and the
test binaries of any crate of yours that links it — needs that rpath, or it
links and then dies in dyld at launch.

A build script's link arguments reach only its package's targets, so the rpath
belongs in `.cargo/config.toml`, where a rustflag covers every kind of target:

```toml
[target.'cfg(target_os = "macos")']
rustflags = ["-C", "link-arg=-Wl,-rpath,/usr/lib/swift"]
```

## Shape

- `containerization`: `ImageStore`, `Image`, `image::Description`,
  `InitImage`, `Ext4Unpacker`, `Kernel`, `ContainerManager`,
  `LinuxContainer`, `LinuxProcess`, and the configuration types they take
  (`linux_container::Configuration`, `LinuxProcessConfiguration`, `Mount`,
  `Dns`, `Hosts`, ...). Their defaults match Containerization's.
- `containerization_oci`: `LocalContentStore`, `Content`, `ContentWriter`,
  `Descriptor`, `Platform`, `User`.
- `containerization_ext4`: `ext4::Ext4Reader`, `ext4::JournalConfig`.
- `containerization_extras`: `IPv4Address`, `IPv6Address`, `IpAddress`,
  `Prefix`, `CIDRv4`, `CIDRv6`, `Cidr`, `MACAddress`, `ProgressEvent`,
  `ProgressHandler`.
- `containerization_os`: `terminal::Size`.

A few things work differently because Rust can't express them the way Swift
does:

- Rust has no default arguments, so `ContainerManager.create`'s optional
  arguments are fields of `container_manager::CreateOptions`. Its `Default`
  uses the same values as Swift.
- Rust has no overloading either. Where Swift overloads a name, the second
  Rust method adds a suffix naming the argument that tells them apart:
  `ContainerManager.create(_:image:rootfs:...)` is `create_with_rootfs`, and
  `ImageStore(path:contentStore:)` is `ImageStore::with_content_store`.
- Where Swift takes a `ReaderStream` or `Writer` for a process's `stdin`,
  `stdout` and `stderr`, Rust takes a file descriptor. Swift uses a duplicate
  of it, so you keep yours open and close it yourself.
- `ContainerManager.create` takes a Rust closure. It receives the
  configuration the manager has prepared and runs on a Swift thread, so it
  must be `Send + 'static`. So must `LocalContentStore.ingest`'s body and a
  `ProgressHandler`, and a `ProgressHandler` must also be `Sync`.
- `Content.decode()` is generic over Swift's `Decodable`, which Rust can't
  call. Read `Content::data` and decode the bytes yourself.
- Swift computes everything about an address, from parsing it to its
  `description` and `isLoopback`. Each of those calls Swift and returns a
  `Result`, which is why addresses have a `description` method rather than
  `Display`. For the same reason, their ordering is `PartialOrd`, which asks
  Swift's `<` and gives `None` where Swift can't be asked.
- Where Swift's initializer checks or changes a value, as with `Prefix`,
  `CIDRv4`, `CIDRv6` and `MACAddress`, only Swift makes one, so their fields
  are read through getters.

## Unimplemented

The framework is larger than these bindings. Not exposed: `LinuxPod`, a
`Network` for `ContainerManager`, `VZVirtualMachineManager` and
`LinuxContainer`'s own initializers, container statistics, filesystem
operations, file copy between host and guest, vsock, registry authentication,
push, and OCI layout save.

An OCI runtime (and so seccomp) is configurable, but requires an init image with
`runc`, which Apple does not publish.

## Versioning

The init image's `vminitd` must match the Containerization release this crate
builds against (0.48.0, in `swift/Package.swift`): they share a protocol, and a
mismatch fails at runtime rather than at build time. As in Containerization,
the caller chooses the kernel and the init image.

## Testing

`cargo nextest run` runs the unit tests.

The suite in `tests/` boots real containers, so it sits behind the `integration`
feature and runs through `bin/dev/test-integration`, which signs each test binary
with `containerization.entitlements` first — the entitlement is checked against the
calling process.

Those tests share an image store at `~/.cache/containerization-framework-tests`,
kept between runs. Before the tests that boot a VM, nextest runs
`bin/dev/prepare-integration` as a setup script, which downloads the kernel into
the store. The tests then pull the init image and `alpine:3` themselves. A first
run therefore needs the network, and later runs reuse the store. This directory
can be deleted.

## License

MIT. Containerization itself is Apache-2.0 and is fetched at build time, not
vendored here.
