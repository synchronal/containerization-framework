//! Stand-ins for `VolumeSubpath.swift`.

use super::CzOutcome;

taken!(volume_subpath_extracted -> (String, Vec<String>),);

failing!(
  cz_volume_subpath_is_subpath_mount(Vec<String>),
  cz_volume_subpath_extract(Vec<String>),
);
