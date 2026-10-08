# containerization-framework

Rust bindings for Apple's [Containerization](https://github.com/apple/containerization)
framework: Linux containers.

The Rust API mirrors Containerization's Swift API as much as possible.
Modules are named after the Swift modules, types after the Swift types, and
methods after their Swift methods, except for using snake case. A Swift type
nested in another, like `LinuxContainer.Configuration`, is found in a module
named after its parent: `linux_container::Configuration`. Types are grouped
into modules of related types, such as `containerization::container` and
`containerization_oci::image`, so a path reads
`containerization::container::linux_container::Configuration`.

```rust
use containerization_framework as cfw;
use cfw::containerization as cz;
use cfw::containerization_extras as cz_extras;

let store = cz::image::ImageStore::new("/Users/me/.cache/containers".as_ref())?;
let kernel = cz::vm::Kernel::new("/Users/me/.cache/vmlinux", cz::vm::SystemPlatform::LINUX_ARM);
let mut manager = cz::container::ContainerManager::with_initfs_reference(
    &kernel,
    "ghcr.io/apple/containerization/vminit:0.48.0",
    &store,
    Default::default(),
)?;

let image = store.get("docker.io/library/alpine:3", true)?;
let address = cz_extras::address::CIDRv4::parse("192.168.64.7/24")?;
let gateway = cz_extras::address::IPv4Address::parse("192.168.64.1")?;
let options = cz::container::container_manager::CreateOptions { networking: false, ..Default::default() };
let container = manager.create("example", &image, options, move |config| {
    config.process.arguments = vec!["/bin/sleep".into(), "infinity".into()];
    config.interfaces = vec![cz::network::Interface::Nat(cz::network::NATInterface::new(address, Some(gateway)))];
    config.dns = Some(cz::network::DNS { nameservers: vec!["192.168.64.1".into()], ..Default::default() });
    Ok(())
})?;
container.create()?;
container.start()?;

let process = container.exec("hello", cz::process::LinuxProcessConfiguration::new(&["/bin/echo", "hello"]))?;
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

On other platforms this crate compiles, so a cross-platform workspace builds,
but every call that would reach Swift returns `Error::Unavailable`. Values
built purely in Rust, such as `IPv4Address::new` or `Descriptor::new`, work as
usual.

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

- `containerization`, in five groups. `image`: `ImageStore`, `Image`,
  `image::Description`, `InitImage`, `KernelImage`, `EXT4Unpacker`. `vm`:
  `Kernel`, `SystemPlatform`, `VMConfiguration`, `VMResources`, `BootLog`,
  `VZVirtualMachineManager`, `VZVirtualMachineInstance`, `Vminitd`,
  `VsockListener`. `network`:
  `VmnetNetwork`, `NATInterface`, `Interface`, `DNS`, `Hosts`. `container`:
  `ContainerManager`, `LinuxContainer`, `LinuxPod`, `Mount`, and the
  configuration types they take (`linux_container::Configuration`, ...).
  `process`: `LinuxProcess`, `LinuxProcessConfiguration`, `Signal`,
  `LinuxCapabilities`, `ExitStatus`. Their defaults match Containerization's.
- `containerization_oci`, in four groups. `content`: `LocalContentStore`,
  `Content`, `ContentWriter`. `image`: `Descriptor`, `Platform`, `Reference`,
  `Manifest`. `client`: `RegistryClient`, `Authentication`, `KeychainHelper`.
  `runtime`: `Spec` and the types it holds, `Bundle`, `State`, `User`.
- `containerization_archive`: `ArchiveReader`, `ArchiveWriter`, `WriteEntry`,
  and the formats and filters they take.
- `containerization_error`: the `Code` that a thrown `ContainerizationError`
  carries, which `Error::is_code` checks.
- `containerization_ext4`: `ext4::Formatter`, `ext4::EXT4Reader`,
  `ext4::SuperBlock`, `ext4::Inode`, `ext4::JournalConfig`,
  `FileTimestamps`.
- `containerization_extras`: in `address`, `IPv4Address`, `IPv6Address`,
  `IPAddress`, `Prefix`, `CIDRv4`, `CIDRv6`, `CIDR` and `MACAddress`; at the
  root, `InterfaceAddress`, `LinkRoute`, `DefaultRoute`, `ProgressEvent` and
  `ProgressHandler`; and `proxy_utils`.
- `containerization_io`: `ReadStream`.
- `containerization_os`: `Terminal`, `CapabilityName`, `CapabilitySet`,
  `binfmt`, `sysctl`, `file`, and `keychain::KeychainQuery`.

A few things work differently because Rust can't express them the way Swift
does:

- Rust has no default arguments, so `ContainerManager.create`'s optional
  arguments are fields of `container::container_manager::CreateOptions`. Its
  `Default` uses the same values as Swift. An initializer's options struct is
  named after its type, such as `ContainerManagerOptions`, and a method's is
  named after the Rust method, such as `CreateWithRootfsOptions`.
- Rust has no overloading either. Where Swift overloads a name, the second
  Rust method adds a suffix naming the argument that tells them apart:
  `ContainerManager.create(_:image:rootfs:...)` is `create_with_rootfs`, and
  `ImageStore(path:contentStore:)` is `ImageStore::with_content_store`. Where
  the overloads differ only in taking a closure, as with
  `LinuxContainer.exec`, the one taking a closure ends in `_with`.
- Where Swift takes a `ReaderStream` or `Writer` for a process's `stdin`,
  `stdout` and `stderr`, Rust takes a file descriptor. Swift uses a duplicate
  of it, so you keep yours open and close it yourself.
- `ContainerManager.create` takes a Rust closure. It receives the
  configuration the manager has prepared and runs on a Swift thread, so it
  must be `Send + 'static`. So must `LocalContentStore.ingest`'s body and a
  `ProgressHandler`, and a `ProgressHandler` must also be `Sync`.
- Swift's configuration closures can throw, so Rust's return
  `Result<(), Error>`. To throw, return `Err(Error::failed(...))`. An error
  the closure returns is the one the call returns.
- `Content.decode()` is generic over Swift's `Decodable`, which Rust can't
  call. Read `Content::data` and decode the bytes yourself.
- Swift computes everything about an address, from parsing it to its
  `description` and `isLoopback`. Each of those calls Swift and returns a
  `Result`, which is why addresses have a `description` method rather than
  `Display`. Their `PartialOrd` and `Platform`'s `PartialEq` call Swift's
  operators too, so off macOS, comparing them panics.
- Where Swift's initializer checks or changes a value, as with `Prefix`,
  `CIDRv4`, `CIDRv6` and `MACAddress`, only Swift makes one, so their fields
  are read through getters.

## OCI runtimes

An OCI runtime, and with it a seccomp profile, can be configured. Swift refuses
a seccomp profile without an OCI runtime path, and the runtime has to be in the
init image. The init image Apple publishes has no `runc`, so such a container
fails to start unless you build an init image with one.

## Versioning

The init image's `vminitd` must match the Containerization release this crate
builds against (0.48.0, in `swift/Package.swift`): they share a protocol, and a
mismatch fails at runtime rather than at build time. As in Containerization,
the caller chooses the kernel and the init image.

## Testing

`cargo nextest run` runs the unit tests. They also check Rust's copies of Swift
values, such as raw values, defaults and struct layouts, against Swift.

The suites in `tests/` call into the real framework, so they sit behind the
`integration` feature and run through `bin/dev/test-integration`. Some only ask
Swift for values, while others boot real containers and VMs. The script signs
each test binary with `containerization.entitlements` first, because the
entitlement is checked against the calling process. Run the suites only through
the script: a test binary that cargo rebuilds after the script signed it has no
entitlement, and its VMs fail to start.

Those tests share an image store at `~/.cache/containerization-framework-tests`,
which is kept between runs. Before the tests that boot a VM, nextest runs
`bin/dev/prepare-integration` as a setup script, which downloads the kernel into
the store. The tests then pull the init image and `alpine:3` themselves, and
the registry tests talk to Docker Hub. A first run therefore needs the network,
and later runs reuse the store. The store's directory can be deleted at any
time.

Two tests open Containerization's default image store, and in doing so create
`~/Library/Application Support/com.apple.containerization` if it is missing.
The tests that need a vmnet network pass without checking anything on a host
whose process lacks the vmnet entitlement, and say so in their output.

## License

MIT. Containerization itself is Apache-2.0 and is fetched at build time, not
vendored here.
