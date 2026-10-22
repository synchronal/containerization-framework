//! Generates the FFI glue, builds the Swift package, and tells cargo how to
//! link it.
//!
//! The package is staged into `OUT_DIR`, but SwiftPM's build directory lives
//! at `target/<profile>/containerization-framework-swift`. `OUT_DIR` is keyed
//! by this script's fingerprint, and editing this file or changing the
//! rustflags yields a new one. The build directory holds the compiled
//! dependency graph, so a copy per fingerprint would cost about 2 GiB and a
//! full rebuild of Containerization.
//!
//! macOS only. On other platforms the bridge is a stand-in
//! (`src/platform/unsupported`) that compiles, but fails every call that would
//! reach Swift.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// swift-bridge's generated header directory; `bridging-header.h` imports it.
const BRIDGE: &str = "containerization-bridge";
const PACKAGE: &str = "ContainerizationBridge";
/// The files holding the `#[swift_bridge::bridge]` modules: every `.rs` file
/// directly in `src/bridge`.
///
/// Their glue is concatenated in this order, and the C header must declare an
/// enum before a function uses it, so `mod.rs`, which declares every shared
/// enum, comes first. The rest are sorted.
///
/// A new module needs a `mod` line in `mod.rs`, which reruns this script, so
/// the directory itself isn't watched.
fn bridge_modules() -> Vec<PathBuf> {
  let bridge = manifest_dir().join("src").join("bridge");
  let mut modules: Vec<PathBuf> = std::fs::read_dir(&bridge)
    .expect("src/bridge should be readable")
    .map(|entry| entry.expect("a readable directory entry").path())
    .filter(|path| path.is_file() && path.extension().is_some_and(|extension| extension == "rs"))
    .collect();

  modules.sort_by_key(|path| (!path.ends_with("mod.rs"), path.clone()));
  modules
}

/// Never copied into the staged package: SwiftPM's build directory, and glue
/// this script regenerates.
const NOT_INPUTS: [&str; 2] = [".build", "generated"];

fn manifest_dir() -> PathBuf {
  PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR"))
}

fn out_dir() -> PathBuf {
  PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"))
}

fn source_package_dir() -> PathBuf {
  manifest_dir()
    .join("swift")
    .canonicalize()
    .expect("the swift package is committed inside this crate")
}

fn source_sources_dir() -> PathBuf {
  source_package_dir().join("Sources").join(PACKAGE)
}

fn staged_package_dir() -> PathBuf {
  out_dir().join("swift")
}

/// SwiftPM's build directory, one per profile.
///
/// `OUT_DIR` is `<target>/<profile>/build/<crate>-<fingerprint>/out`, so the
/// profile directory is its third ancestor. Cargo documents no part of that
/// layout, so a path that doesn't match it falls back to `OUT_DIR`.
fn scratch_dir() -> PathBuf {
  let out = out_dir();
  let profile = out.ancestors().nth(3).filter(|_| {
    out
      .ancestors()
      .nth(2)
      .is_some_and(|dir| dir.ends_with("build"))
  });

  profile
    .unwrap_or(&out)
    .join("containerization-framework-swift")
}

fn is_release() -> bool {
  std::env::var("PROFILE").as_deref() == Ok("release")
}

/// Makes swift-bridge's generated `@_cdecl` shims public.
///
/// They are emitted internal. A release build optimises whole-module, sees no
/// caller inside the module (the callers are across the C ABI), and strips
/// them, leaving two undefined `__swift_bridge__$…` symbols at link time.
/// Debug builds don't optimise, so `cargo test` never shows it.
fn publish_bridge_shims(generated: &Path) {
  let path = generated.join(BRIDGE).join(format!("{BRIDGE}.swift"));
  let source = std::fs::read_to_string(&path).expect("swift-bridge should have just written the glue");
  let published = source.replace("\nfunc __swift_bridge__", "\npublic func __swift_bridge__");

  // A silent no-op would resurface as an unexplained release-only link failure.
  if published == source {
    panic!(
      "no `func __swift_bridge__` to publish in {}; swift-bridge's output has changed shape",
      path.display()
    );
  }

  std::fs::write(&path, published).expect("the generated glue should be writable");
}

/// Mirrors `from` onto `to`, removing whatever `from` doesn't have.
///
/// Unchanged files aren't rewritten, because SwiftPM decides what to rebuild
/// from mtimes. Entries named in `skip` are neither copied nor removed, since
/// something else owns them.
fn mirror(from: &Path, to: &Path, skip: &[&str]) {
  std::fs::create_dir_all(to).expect("the destination should be creatable");

  for entry in std::fs::read_dir(to).expect("the destination was just created") {
    let entry = entry.expect("a readable directory entry");
    let name = entry.file_name();

    if skip.iter().any(|skipped| name == *skipped) || from.join(&name).exists() {
      continue;
    }

    let stale = entry.path();
    let removed = if entry.file_type().expect("a stat-able entry").is_dir() {
      std::fs::remove_dir_all(&stale)
    } else {
      std::fs::remove_file(&stale)
    };

    removed.expect("a stale staged file should be removable");
  }

  for entry in std::fs::read_dir(from).expect("the source directory should be readable") {
    let entry = entry.expect("a readable directory entry");
    let name = entry.file_name();

    if skip.iter().any(|skipped| name == *skipped) {
      continue;
    }

    let destination = to.join(&name);

    if entry.file_type().expect("a stat-able entry").is_dir() {
      mirror(&entry.path(), &destination, skip);
      continue;
    }

    let fresh = std::fs::read(entry.path()).expect("a readable source file");

    if std::fs::read(&destination).is_ok_and(|current| current == fresh) {
      continue;
    }

    std::fs::write(&destination, fresh).expect("the staged file should be writable");
  }
}

/// Compiles the staged package to a static library.
///
/// The bridging header is set in the package's `swiftSettings`, not via
/// `-Xswiftc`, which SwiftPM would apply to every target in the graph. It
/// resolves against `#filePath`, so it follows the package to `OUT_DIR`.
///
/// The scratch path holds every dependency's checkout as well as the compiled
/// output, hydrated from SwiftPM's cache.
fn compile_swift() {
  let mut command = Command::new("swift");

  command
    .arg("build")
    .arg("--package-path")
    .arg(staged_package_dir())
    .arg("--scratch-path")
    .arg(scratch_dir());

  if is_release() {
    command.args(["-c", "release"]);
  }

  let output = command.output().expect("swift should be on PATH on macOS");

  if !output.status.success() {
    panic!(
      "swift build failed\n--- stdout ---\n{}\n--- stderr ---\n{}",
      String::from_utf8_lossy(&output.stdout),
      String::from_utf8_lossy(&output.stderr),
    );
  }
}

/// Where `swift build` leaves the static library.
fn swift_build_dir() -> PathBuf {
  scratch_dir().join(if is_release() { "release" } else { "debug" })
}

/// System libraries Containerization's `CArchive` target links against.
///
/// Its `linkerSettings` only apply when SwiftPM links; without repeating them
/// here every `archive_*` symbol is undefined in the Rust binary.
const SYSTEM_LIBRARIES: [&str; 5] = ["archive", "z", "bz2", "lzma", "iconv"];

/// Where the dynamic Swift runtime lives.
const SWIFT_RUNTIME_DIR: &str = "/usr/lib/swift";

/// Links what the static library depends on but does not carry: the Swift
/// runtime, system frameworks, and `SYSTEM_LIBRARIES`.
fn link_swift_runtime() {
  let developer = Command::new("xcode-select")
    .arg("--print-path")
    .output()
    .ok()
    .filter(|output| output.status.success())
    .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
    .unwrap_or_else(|| "/Applications/Xcode.app/Contents/Developer".to_string());

  // The first is Xcode's, and the second the Command Line Tools'. A search
  // path that doesn't exist is ignored.
  println!("cargo:rustc-link-search={developer}/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx/");
  println!("cargo:rustc-link-search={developer}/usr/lib/swift/macosx/");
  println!("cargo:rustc-link-search={SWIFT_RUNTIME_DIR}");

  println!("cargo:rustc-link-lib=framework=Virtualization");
  // Dependencies such as swift-certificates and NIOTransportServices import
  // Security.
  println!("cargo:rustc-link-lib=framework=Security");

  for library in SYSTEM_LIBRARIES {
    println!("cargo:rustc-link-lib={library}");
  }
}

/// The Containerization release `Package.swift` pins, which
/// `versions::INIT_IMAGE` names, so that the pin has one home.
fn pinned_containerization() -> String {
  const PIN: &str = "let containerization: Version = \"";

  let path = source_package_dir().join("Package.swift");
  let manifest = std::fs::read_to_string(&path).expect("the swift package's manifest should be readable");

  manifest
    .lines()
    .find_map(|line| line.strip_prefix(PIN)?.strip_suffix('"'))
    .unwrap_or_else(|| panic!("{} should pin Containerization with `{PIN}…\"`", path.display()))
    .to_string()
}

fn main() {
  println!("cargo:rerun-if-changed=build.rs");

  let modules = bridge_modules();

  for module in &modules {
    println!("cargo:rerun-if-changed={}", module.display());
  }

  println!(
    "cargo:rerun-if-changed={}",
    source_package_dir().join("Package.swift").display()
  );
  println!(
    "cargo:rustc-env=CFW_CONTAINERIZATION_VERSION={}",
    pinned_containerization()
  );

  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
    return;
  }

  // `swift build` builds for the machine it runs on, which is the one this
  // script was compiled for. Any other architecture would leave every bridge
  // symbol undefined at link time.
  let target = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

  if target != "aarch64" || !cfg!(target_arch = "aarch64") {
    panic!(
      "containerization-framework builds only on and for Apple silicon (aarch64-apple-darwin), as Containerization \
       runs only there, but this build runs on {} and targets {target}",
      std::env::consts::ARCH
    );
  }

  println!("cargo:rerun-if-changed={}", source_sources_dir().display());
  println!(
    "cargo:rerun-if-changed={}",
    source_package_dir().join("Package.resolved").display()
  );

  let staged = staged_package_dir();

  mirror(&source_package_dir(), &staged, &NOT_INPUTS);

  let glue = out_dir().join("swift-bridge");

  // `OUT_DIR` survives between builds, so glue from the last run is cleared.
  let _ = std::fs::remove_dir_all(&glue);

  swift_bridge_build::parse_bridges(modules).write_all_concatenated(&glue, BRIDGE);
  publish_bridge_shims(&glue);
  mirror(&glue, &staged.join("Sources").join(PACKAGE).join("generated"), &[]);

  compile_swift();

  // The unit tests check `versions` against Containerization's Makefile.
  println!(
    "cargo:rustc-env=CFW_CONTAINERIZATION_CHECKOUT={}",
    scratch_dir()
      .join("checkouts")
      .join("containerization")
      .display()
  );
  println!("cargo:rustc-link-lib=static={PACKAGE}");
  println!("cargo:rustc-link-search={}", swift_build_dir().display());

  link_swift_runtime();
}
