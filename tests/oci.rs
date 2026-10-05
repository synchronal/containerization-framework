#![cfg(feature = "integration")]

//! `ContainerizationOCI`'s value types, each asked of Swift.

mod support;

use cfw::containerization_oci as oci;
use containerization_framework as cfw;

fn platform(string: &str) -> oci::Platform {
  oci::Platform::parse(string).expect("a valid platform")
}

#[test]
fn parses_a_platform() {
  assert_eq!(
    platform("linux/arm64"),
    oci::Platform {
      architecture: "arm64".to_string(),
      os: "linux".to_string(),
      os_version: None,
      os_features: None,
      variant: Some("v8".to_string()),
    }
  );
  assert_eq!(platform("linux/x86_64").architecture, "amd64");
  assert_eq!(platform("linux/arm/v6").variant.as_deref(), Some("v6"));
}

#[test]
fn rejects_a_malformed_platform() {
  let error = oci::Platform::parse("plan9/amd64").expect_err("an unknown OS");
  assert!(error.is_code(cfw::containerization_error::Code::InvalidArgument));
  assert!(oci::Platform::parse("linux").is_err());
  assert!(oci::Platform::parse("linux/amd64/v3").is_err());
}

#[test]
fn describes_a_platform() {
  assert_eq!(
    platform("linux/arm/v7")
      .description()
      .expect("a description"),
    "linux/arm/v7"
  );
  assert_eq!(
    platform("linux/arm64")
      .description()
      .expect("a description"),
    "linux/arm64"
  );
}

#[test]
fn compares_platforms_as_swift_does() {
  let arm64 = |variant: Option<&str>, os_version: Option<&str>| oci::Platform {
    architecture: "arm64".to_string(),
    os: "linux".to_string(),
    os_version: os_version.map(str::to_string),
    os_features: None,
    variant: variant.map(str::to_string),
  };

  assert_eq!(arm64(None, None), arm64(Some("v8"), None), "arm64 defaults to v8");
  assert_eq!(
    arm64(Some("v8"), Some("1")),
    arm64(Some("v8"), None),
    "the OS version is ignored"
  );
  assert_ne!(arm64(Some("v8"), None), platform("linux/amd64"));
}

#[test]
fn matches_platforms_as_swift_does() {
  let matches = |lhs: &str, rhs: &str| platform(lhs).matches(&platform(rhs)).expect("an answer");

  assert!(matches("linux/arm/v6", "linux/arm/v7"));
  assert!(!matches("linux/arm/v7", "linux/arm/v6"));
  assert!(matches("linux/amd64", "linux/amd64"));
  assert!(!matches("linux/amd64", "linux/arm64"));
}

#[test]
fn reads_a_pulled_images_index_manifest_and_config() {
  let store = support::store::image_store();
  let image = support::store::image(&store);
  let platform = platform("linux/arm64");

  let descriptor = image.descriptor();
  assert_eq!(descriptor.digest, image.digest());
  assert_eq!(image.description().descriptor, descriptor);

  let index = image.index().expect("the image's index");
  assert_eq!(index.schema_version, 2);
  let listed = index
    .manifests
    .iter()
    .find(|manifest| manifest.platform.as_ref() == Some(&platform))
    .expect("an arm64 manifest in the index");
  assert_eq!(
    &image
      .descriptor_for(&platform)
      .expect("the arm64 descriptor"),
    listed
  );

  let manifest = image.manifest(&platform).expect("the arm64 manifest");
  assert!(!manifest.layers.is_empty());
  let content_store = oci::LocalContentStore::new(&support::store::content_store_path()).expect("the content store");
  for layer in &manifest.layers {
    assert!(
      content_store
        .get(&layer.digest)
        .expect("a lookup")
        .is_some(),
      "{} should be in the content store",
      layer.digest
    );
  }

  let config = image.config(&platform).expect("the arm64 config");
  assert_eq!(config.architecture, "arm64");
  assert_eq!(config.os, "linux");
  assert_eq!(config.rootfs.r#type, "layers");
  assert_eq!(config.rootfs.diff_ids.len(), manifest.layers.len());
  let image_config = config.config.expect("alpine's config");
  assert!(
    image_config
      .env
      .expect("alpine's environment")
      .iter()
      .any(|variable| variable.starts_with("PATH="))
  );
}

#[test]
fn fails_for_a_platform_the_image_lacks() {
  let store = support::store::image_store();
  let image = support::store::image(&store);

  let error = image
    .descriptor_for(&platform("windows/amd64"))
    .expect_err("no Windows manifest");
  assert!(error.is_code(cfw::containerization_error::Code::InvalidArgument));
}

#[test]
fn parses_and_normalizes_a_reference() {
  let mut reference = oci::Reference::parse("docker.io/alpine").expect("a valid reference");

  assert_eq!(reference.domain().as_deref(), Some("docker.io"));
  assert_eq!(reference.resolved_domain().as_deref(), Some("registry-1.docker.io"));
  assert_eq!(reference.path(), "alpine");
  assert_eq!(reference.tag(), None);

  reference.normalize();
  assert_eq!(reference.path(), "library/alpine");
  assert_eq!(reference.tag().as_deref(), Some("latest"));
  assert_eq!(reference.description(), "docker.io/library/alpine:latest");

  let digest = format!("sha256:{}", "a".repeat(64));
  let pinned = reference
    .with_digest(&digest)
    .expect("a reference by digest");
  assert_eq!(pinned.digest(), Some(digest.clone()));
  assert_eq!(pinned.tag(), None);
  assert_eq!(pinned.description(), format!("docker.io/library/alpine@{digest}"));
  assert_eq!(
    pinned.with_tag("3").expect("a tagged reference").name(),
    "docker.io/library/alpine"
  );
}

#[test]
fn makes_a_reference_from_its_parts() {
  let reference = oci::Reference::new(
    "library/alpine",
    oci::reference::NewOptions {
      domain: Some("ghcr.io".to_string()),
      tag: Some("3".to_string()),
      ..Default::default()
    },
  )
  .expect("a reference");

  assert_eq!(reference.description(), "ghcr.io/library/alpine:3");
  assert_eq!(
    oci::Reference::resolve_domain("docker.io").expect("a domain"),
    "registry-1.docker.io"
  );
}

#[test]
fn rejects_a_malformed_reference() {
  let error = oci::Reference::parse("Alpine")
    .err()
    .expect("an uppercase path");
  assert!(error.is_code(cfw::containerization_error::Code::InvalidArgument));
  assert!(
    oci::Reference::parse("alpine")
      .expect("a reference")
      .with_tag("")
      .is_err()
  );
}

#[test]
fn parses_a_digest() {
  let encoded = "0123456789abcdef".repeat(4);
  let digest = oci::ParsedDigest::parse(&format!("sha256:{encoded}")).expect("a valid digest");

  assert_eq!(digest.encoded(), encoded);
  assert_eq!(
    digest.description().expect("a description"),
    format!("sha256:{encoded}")
  );
  assert_eq!(
    oci::ParsedDigest::parse_path_component(&encoded).expect("the hex digits alone"),
    digest
  );
  assert!(oci::ParsedDigest::parse(&encoded).is_err(), "parse needs the prefix");
  assert!(oci::ParsedDigest::is_valid(&encoded).expect("an answer"));
  assert!(
    !oci::ParsedDigest::is_valid(&encoded.to_uppercase()).expect("an answer"),
    "uppercase hex is rejected"
  );
}

#[test]
fn finds_a_digests_path_in_a_directory() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  // Not canonicalized: Swift's `resolvingSymlinksInPath` drops macOS's
  // `/private` prefix, which `canonicalize` adds.
  let root = directory.path();
  let encoded = "f".repeat(64);
  let digest = oci::ParsedDigest::parse_path_component(&encoded).expect("a valid digest");

  assert_eq!(digest.path(&root).expect("a path"), root.join(&encoded));
}

#[test]
fn makes_values_with_swifts_defaults() {
  let config = oci::Descriptor::new(oci::MediaTypes::IMAGE_CONFIG, "sha256:00", 2);

  let index = oci::Index::new(vec![config.clone()]);
  assert_eq!(index.schema_version, 2);
  assert_eq!(index.media_type, oci::MediaTypes::INDEX);

  let manifest = oci::Manifest::new(config, Vec::new());
  assert_eq!(manifest.schema_version, 2);
  assert_eq!(manifest.media_type.as_deref(), Some(oci::MediaTypes::IMAGE_MANIFEST));
}
