#![cfg(feature = "integration")]

//! `EXT4Unpacker`, and `EXT4.EXT4Reader` reading back what it unpacked.

mod support;

use containerization_framework as cfw;
use std::sync::Arc;
use std::sync::Mutex;

const CAPACITY_IN_BYTES: u64 = 512 * 1024 * 1024;

/// Every event a handler is called with, in order.
fn recorded() -> (
  Arc<Mutex<Vec<cfw::containerization_extras::ProgressEvent>>>,
  cfw::containerization_extras::ProgressHandler,
) {
  let events = Arc::new(Mutex::new(Vec::new()));
  let recording = Arc::clone(&events);

  (
    events,
    Box::new(move |batch| {
      recording
        .lock()
        .expect("a recording")
        .extend_from_slice(batch)
    }),
  )
}

#[test]
fn unpacks_an_image_and_reports_its_progress() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let at = directory.path().join("rootfs.ext4");
  let image = support::store::image(&support::store::image_store());
  let platform = cfw::containerization_oci::Platform::current().expect("the current platform");
  let (events, handler) = recorded();

  let mount = cfw::containerization::Ext4Unpacker::new(CAPACITY_IN_BYTES, None)
    .unpack(&image, &platform, &at, Some(handler))
    .expect("the image should unpack");

  assert_eq!(mount.r#type, "ext4");
  assert!(mount.source.ends_with("/rootfs.ext4"), "{}", mount.source);
  assert!(
    std::fs::metadata(&at).expect("the filesystem's file").len() >= CAPACITY_IN_BYTES,
    "the filesystem holds at least its capacity"
  );

  let events = events.lock().expect("the recording");
  let total: isize = events
    .iter()
    .map(|event| match event {
      cfw::containerization_extras::ProgressEvent::AddTotalItems(items) => *items,
      _ => 0,
    })
    .sum();
  let unpacked: isize = events
    .iter()
    .map(|event| match event {
      cfw::containerization_extras::ProgressEvent::AddItems(items) => *items,
      _ => 0,
    })
    .sum();

  assert!(total > 0, "{events:?}");
  assert_eq!(unpacked, total, "every item counted is unpacked");
}

#[test]
fn unpacks_with_a_journal() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let at = directory.path().join("journaled.ext4");
  let image = support::store::image(&support::store::image_store());
  let platform = cfw::containerization_oci::Platform::current().expect("the current platform");
  let journal = cfw::containerization_ext4::ext4::JournalConfig {
    size: None,
    default_mode: Some(cfw::containerization_ext4::ext4::journal_config::JournalMode::Ordered),
  };

  cfw::containerization::Ext4Unpacker::new(CAPACITY_IN_BYTES, Some(journal))
    .unpack(&image, &platform, &at, None)
    .expect("the image should unpack");

  assert!(at.is_file(), "{}", at.display());
}

#[test]
fn exports_an_unpacked_filesystem_as_a_tar() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let at = directory.path().join("rootfs.ext4");
  let archive = directory.path().join("rootfs.tar");
  support::store::unpack(&at);

  cfw::containerization_ext4::ext4::Ext4Reader::new(&at)
    .expect("the unpacked filesystem should read")
    .export(&archive)
    .expect("the filesystem should export");

  let listing = std::process::Command::new("tar")
    .arg("-tf")
    .arg(&archive)
    .output()
    .expect("tar should run");

  assert!(listing.status.success(), "{}", String::from_utf8_lossy(&listing.stderr));
  assert!(
    String::from_utf8_lossy(&listing.stdout)
      .lines()
      .any(|path| path.trim_start_matches("./") == "etc/os-release"),
    "the archive should hold the image's files"
  );
}

#[test]
fn says_which_block_device_it_could_not_read() {
  let error = cfw::containerization_ext4::ext4::Ext4Reader::new("/nowhere/rootfs.ext4".as_ref())
    .err()
    .expect("a filesystem that doesn't exist");

  assert!(error.to_string().contains("/nowhere/rootfs.ext4"), "{error}");
}

#[test]
fn reads_the_current_platform() {
  let platform = cfw::containerization_oci::Platform::current().expect("the current platform");

  assert_eq!(platform.os, "linux");
  assert_eq!(platform.architecture, "arm64");
}
