#![cfg(feature = "integration")]

//! Snapshots of an image's root filesystem, unpacked once and cloned for each
//! container.

mod support;

use containerization_framework as cfw;

#[test]
fn unpacks_an_image_once_clones_it_and_deletes_it_unless_kept() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let path = directory.path().join("snapshots");
  let store = support::store::image_store();
  let image = support::store::image(&store);
  let platform = cfw::containerization_oci::image::Platform::current().expect("the current platform");

  let snapshots = cfw::containerization::image::SnapshotManager::new(&path, support::TEST_ROOTFS_SIZE_IN_BYTES)
    .expect("a snapshot manager should be made");

  assert!(path.is_dir(), "the manager creates its directory");
  assert_eq!(snapshots.path(), path);

  let key = cfw::containerization::image::SnapshotManager::key(&image, None).expect("Swift keys the image's snapshot");
  assert_eq!(
    cfw::containerization::image::SnapshotManager::key(&image, Some(&platform))
      .expect("Swift keys the image's snapshot for a platform"),
    key,
    "no platform is the current platform"
  );
  assert_eq!(
    snapshots
      .get(&image, None)
      .expect("Swift looks for the snapshot"),
    None,
    "nothing is unpacked yet"
  );

  let (events, handler) = support::recorded();
  let unpacked = snapshots
    .unpack(
      &image,
      cfw::containerization::image::snapshot_manager::UnpackOptions {
        progress: Some(handler),
        ..Default::default()
      },
    )
    .expect("the image should unpack");

  assert_eq!(unpacked, path.join(format!("{}.ext4", key.encoded())));
  assert!(unpacked.is_file());
  assert!(
    !events.lock().expect("the recording").is_empty(),
    "the unpack reports progress"
  );
  assert_eq!(
    snapshots
      .get(&image, Some(&platform))
      .expect("Swift looks for the snapshot"),
    Some(unpacked.clone())
  );

  let (again, handler) = support::recorded();
  assert_eq!(
    snapshots
      .clone()
      .unpack(
        &image,
        cfw::containerization::image::snapshot_manager::UnpackOptions {
          platform: Some(platform.clone()),
          progress: Some(handler),
        },
      )
      .expect("an unpacked image returns its snapshot"),
    unpacked
  );
  assert!(
    again.lock().expect("the recording").is_empty(),
    "a second unpack of the same image unpacks nothing"
  );

  let destination = directory.path().join("rootfs.ext4");
  let mount = snapshots
    .clone_to(&image, None, &destination)
    .expect("the snapshot should clone");

  assert_eq!(mount.source, destination.display().to_string());
  assert_eq!(mount.destination, "/");
  assert!(destination.is_file());

  let error = snapshots
    .clone_to(&image, None, &destination)
    .expect_err("a clone doesn't replace a file");
  assert!(error.is_code(cfw::containerization_error::Code::Exists), "{error}");

  assert_eq!(
    snapshots
      .delete(std::slice::from_ref(&image))
      .expect("the snapshots should be pruned")
      .0,
    Vec::<String>::new(),
    "a kept image keeps its snapshot"
  );

  let (deleted, freed) = snapshots
    .delete(&[])
    .expect("the snapshots should be pruned");
  assert_eq!(deleted, [format!("sha256:{}", key.encoded())]);
  assert!(freed > 0, "deleting a snapshot frees its blocks");
  assert_eq!(
    snapshots
      .get(&image, None)
      .expect("Swift looks for the snapshot"),
    None
  );
  assert!(destination.is_file(), "a clone outlives its snapshot");
}
