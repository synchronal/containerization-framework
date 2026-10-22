# containerization-framework

Rust bindings for Apple's [Containerization](https://github.com/apple/containerization)
framework, which runs each Linux container in its own lightweight VM.

The Rust API mirrors Containerization's Swift API as closely as it can.
Modules are named after the Swift modules, types after the Swift types, and
methods after the Swift methods, in snake case. Types are grouped into modules
of related types, such as `containerization::container` and
`containerization_oci::image`. A Swift type nested in another, like
`LinuxContainer.Configuration`, is in a module named after its parent, so its
path is `containerization::container::linux_container::Configuration`.

```rust,no_run
use containerization_framework as cfw;
use cfw::containerization as cz;
use cfw::containerization_extras as cz_extras;

let store = cz::image::ImageStore::new("/Users/me/.cache/containers".as_ref())?;
let kernel = cz::vm::Kernel::new("/Users/me/.cache/vmlinux", cz::vm::SystemPlatform::LINUX_ARM);
let mut manager = cz::container::ContainerManager::with_initfs_reference(
    &kernel,
    cfw::versions::INIT_IMAGE,
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
println!("{status:?}");
Ok::<(), cfw::Error>(())
```

Swift's `async` methods block until they finish, and errors they throw are
returned as `cfw::Error`. A container belongs to the process that created it,
and stops when that process exits.

## Requirements

- macOS 26 on Apple silicon, and Xcode 26 to build. On macOS the crate builds
  only on and for `aarch64-apple-darwin`, and any other target fails with a
  message saying so.
- A network connection on the first build. The build script compiles the
  bundled Swift package, and SwiftPM fetches Containerization and its
  dependencies at the versions pinned in the `Package.resolved` that ships
  with this crate.

On other platforms this crate compiles, so a cross-platform workspace builds,
but every call that would reach Swift returns `Error::Unavailable`. Values
built purely in Rust, such as `IPv4Address::new` or `Descriptor::new`, work as
usual.

## Codesigning

**A binary using this crate must carry the `com.apple.security.virtualization`
entitlement.** Without it Virtualization.framework refuses to start a VM, and
`LinuxContainer::create` fails.

This crate ships a `containerization.entitlements` file. A rebuild drops the
signature, so pass that file, or a copy of it, to `codesign` after every build:

```sh
cargo build --release
codesign --force --sign - --entitlements containerization.entitlements \
  target/release/your-binary
```

Signing ad hoc (`--sign -`) grants the entitlement, but gives the binary a new
code identity on every rebuild, so anything keyed to that identity, such as
Keychain access, prompts again. Sign with a development identity to keep it
stable.

## Linking

This crate links the Swift runtime dynamically, as
`@rpath/libswift_Concurrency.dylib`, which dyld finds in `/usr/lib/swift`.
Every binary that links this crate, including your crates' test binaries,
needs that rpath, or it builds and then dies in dyld at launch.

A build script's link arguments reach only its own package's targets, so set
the rpath in your `.cargo/config.toml`, where a rustflag covers every target:

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
  `LinuxCapabilities`, `ExitStatus`.
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
- `containerization_os`: `Terminal`, `sysctl`, `file`, `file_descriptor_ops`,
  `keychain::KeychainQuery`, and in `linux`, `CapabilityName`, `CapabilitySet`,
  `ProcNetSnmp` and `binfmt`.
- `cloud_hypervisor`: in `device`, `DiskConfig` and `ImageType`, which
  `Mount::ch_disk_config` returns.

A few things work differently because Rust can't express them the way Swift
does:

- Rust has no default arguments, so `ContainerManager.create`'s optional
  arguments are fields of `container::container_manager::CreateOptions`, whose
  `Default` uses Swift's default values. An initializer's options struct is
  named after its type, such as `ContainerManagerOptions`, and a method's is
  named after the Rust method, such as `CreateWithRootfsOptions`. Only a
  function with two or more defaulted arguments takes an options struct. One
  with a single defaulted argument takes it as a plain parameter, and its doc
  gives Swift's default.
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
  `ProgressHandler`, and a `ProgressHandler` must also be `Sync`. Each
  method's docs say whether its closure may call back into this crate.
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
builds against (0.53.0, in `swift/Package.swift`): they share a protocol, and a
mismatch fails at runtime rather than at build time. As in Containerization,
the caller chooses the kernel and the init image.

The `versions` module says which releases to download. `INIT_IMAGE` is the
init image Apple publishes for the pinned release. `KERNEL_ARCHIVE_URL` is the
Kata Containers archive whose kernel Containerization's Makefile uses, and
`KERNEL_IN_ARCHIVE` is the kernel's path inside it.

## Development

Besides Xcode 26, working on this crate needs a few tools. The `Brewfile`
installs [medic](https://github.com/synchronal/medic-rs), and `medic doctor`
then checks for the rest: the Rust toolchain that `rust-toolchain.toml` pins,
`cargo-nextest`, `cargo-audit`, `cargo-outdated`, and the
`aarch64-unknown-linux-gnu` target, which checks that the crate still compiles
off macOS. `bin/dev/test-integration` also needs `python3`, and the Swift
scripts need the `swift-format` that ships with Xcode.

`medic audit` runs the checks, lints and formatters, and `medic test` runs the
unit tests and the doctests. Nextest doesn't run doctests, so
`cargo test --doc` runs them separately.

Setting `DEVELOPMENT_TEAM` to an Apple developer team ID makes
`bin/dev/sign` sign test binaries with that team's identity instead of ad hoc.
An ad hoc signature changes with every build, and the keychain asks again for
each new one.

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
The keychain and registry tests add items to the login keychain and remove
them afterward, so macOS may ask to allow the test binary access to it. Signing
with `DEVELOPMENT_TEAM` set lets an allowed binary stay allowed across builds.
The tests that need a vmnet network pass without checking anything on a host
whose process lacks the vmnet entitlement, and say so in their output.

## License

MIT. Containerization itself is Apache-2.0 and is fetched at build time, not
vendored here.
