#![cfg(feature = "integration")]

//! ContainerizationOS: `VolumeSubpath`, which carries a subpath mount's
//! subpath as a mount option.

use containerization_framework as cfw;

/// What `VolumeSubpath` threw, from an error that should carry it.
fn thrown(error: cfw::Error) -> cfw::containerization_os::volume_subpath::Error {
  match error {
    cfw::Error::VolumeSubpath { error, .. } => error,
    other => panic!("VolumeSubpath should have thrown, not {other:?}"),
  }
}

#[test]
fn tells_a_subpath_mount_by_its_options() {
  let option = cfw::containerization_os::volume_subpath::to_mount_option("data");
  let is_subpath_mount = cfw::containerization_os::volume_subpath::is_subpath_mount;

  assert!(!is_subpath_mount(&["bind", "ro"]).expect("plain options"));
  assert!(is_subpath_mount(&["bind", &option]).expect("a bind with a subpath"));
  assert_eq!(
    thrown(is_subpath_mount(&["rbind", &option]).expect_err("a recursive bind")),
    cfw::containerization_os::volume_subpath::Error::NotBind
  );
  assert_eq!(
    thrown(is_subpath_mount(&[&option]).expect_err("no bind")),
    cfw::containerization_os::volume_subpath::Error::NotBind
  );
}

#[test]
fn takes_the_subpath_from_the_options() {
  let option = cfw::containerization_os::volume_subpath::to_mount_option("data/cache");

  assert_eq!(
    cfw::containerization_os::volume_subpath::extract(&["bind", &option, "ro"]).expect("a subpath"),
    ("data/cache".to_string(), vec!["bind".to_string(), "ro".to_string()])
  );
}

#[test]
fn fails_without_exactly_one_subpath() {
  let extract = cfw::containerization_os::volume_subpath::extract;
  let option = cfw::containerization_os::volume_subpath::to_mount_option("data");

  assert_eq!(
    thrown(extract(&["bind", "ro"]).expect_err("no subpath")),
    cfw::containerization_os::volume_subpath::Error::Missing {
      options: vec!["bind".to_string(), "ro".to_string()]
    }
  );
  assert_eq!(
    thrown(extract(&[&option, &option]).expect_err("two subpaths")),
    cfw::containerization_os::volume_subpath::Error::Invalid {
      subpath: "data".to_string(),
      reason: "it is given more than once".to_string()
    }
  );
}
