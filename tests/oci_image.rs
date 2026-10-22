#![cfg(feature = "integration")]

//! `ContainerizationOCI`'s image values and references, each asked of Swift.
//! None needs a store, so the suite reaches no registry.

mod support;

use cfw::containerization_oci as oci;
use containerization_framework as cfw;
use support::platform;

#[test]
fn parses_a_platform() {
  assert_eq!(
    platform("linux/arm64"),
    oci::image::Platform {
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
  let error = oci::image::Platform::parse("plan9/amd64").expect_err("an unknown OS");
  assert!(error.is_code(cfw::containerization_error::Code::InvalidArgument));
  assert!(oci::image::Platform::parse("linux").is_err());
  assert!(oci::image::Platform::parse("linux/amd64/v3").is_err());
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
  let arm64 = |variant: Option<&str>, os_version: Option<&str>| oci::image::Platform {
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
fn reads_the_current_platform() {
  let platform = oci::image::Platform::current().expect("the current platform");

  assert_eq!(platform.os, "linux");
  assert_eq!(platform.architecture, "arm64");
}

#[test]
fn parses_and_normalizes_a_reference() {
  let mut reference = oci::image::Reference::parse("docker.io/alpine").expect("a valid reference");

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
  let reference = oci::image::Reference::new(
    "library/alpine",
    oci::image::reference::ReferenceOptions {
      domain: Some("ghcr.io".to_string()),
      tag: Some("3".to_string()),
      ..Default::default()
    },
  )
  .expect("a reference");

  assert_eq!(reference.description(), "ghcr.io/library/alpine:3");
  assert_eq!(
    oci::image::Reference::resolve_domain("docker.io").expect("a domain"),
    "registry-1.docker.io"
  );
}

#[test]
fn rejects_a_malformed_reference() {
  let error = oci::image::Reference::parse("Alpine").expect_err("an uppercase path");
  assert!(error.is_code(cfw::containerization_error::Code::InvalidArgument));
  assert!(
    oci::image::Reference::parse("alpine")
      .expect("a reference")
      .with_tag("")
      .is_err()
  );
}

/// The index's own annotations, subject and artifact type don't change its
/// platforms. They are set so that every field crosses to Swift.
#[test]
fn lists_an_indexs_runnable_platforms() {
  let manifest_for = |platform: oci::image::Platform| oci::image::Descriptor {
    platform: Some(platform),
    ..oci::image::Descriptor::new(oci::image::MediaTypes::IMAGE_MANIFEST, "sha256:00", 2)
  };
  let manifest = |platform_string: &str| manifest_for(platform(platform_string));
  let unknown = oci::image::Platform {
    architecture: "unknown".to_string(),
    os: "unknown".to_string(),
    os_version: None,
    os_features: None,
    variant: None,
  };
  let attestation = oci::image::Descriptor {
    annotations: Some(
      [(
        "vnd.docker.reference.type".to_string(),
        "attestation-manifest".to_string(),
      )]
      .into(),
    ),
    ..manifest("linux/arm64")
  };
  let unplatformed = oci::image::Descriptor::new(oci::image::MediaTypes::IMAGE_MANIFEST, "sha256:00", 2);
  let index = oci::image::Index {
    annotations: Some([("org.opencontainers.image.ref.name".to_string(), "3".to_string())].into()),
    subject: Some(manifest("linux/arm64")),
    artifact_type: Some("application/example".to_string()),
    ..oci::image::Index::new(vec![
      manifest("linux/arm64"),
      attestation,
      manifest_for(unknown),
      unplatformed,
      manifest("linux/amd64"),
    ])
  };

  assert_eq!(
    index.runnable_platforms().expect("the index's platforms"),
    vec![platform("linux/arm64"), platform("linux/amd64")],
    "attestations, unknown/unknown and manifests without a platform are left out"
  );
}

#[test]
fn makes_values_with_swifts_defaults() {
  let config = oci::image::Descriptor::new(oci::image::MediaTypes::IMAGE_CONFIG, "sha256:00", 2);

  let index = oci::image::Index::new(vec![config.clone()]);
  assert_eq!(index.schema_version, 2);
  assert_eq!(index.media_type, oci::image::MediaTypes::INDEX);

  let manifest = oci::image::Manifest::new(config, Vec::new());
  assert_eq!(manifest.schema_version, 2);
  assert_eq!(
    manifest.media_type.as_deref(),
    Some(oci::image::MediaTypes::IMAGE_MANIFEST)
  );
}
