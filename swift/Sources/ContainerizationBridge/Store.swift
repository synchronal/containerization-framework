//===----------------------------------------------------------------------===//
// Where containers live in the image store, for sessions and builds alike. The
// Rust side's `Store::container_dir` names the same directory.
//===----------------------------------------------------------------------===//

import Foundation

/// The store's `containers` directory, shared by sessions and builders.
func containers(in root: URL) -> URL {
  root.appending(path: "containers")
}

/// A container's directory (its rootfs and the library's boot log) and the
/// rootfs within it.
func container(_ name: String, in root: URL) -> (directory: URL, rootfs: URL) {
  let directory = containers(in: root).appending(path: name)

  return (directory, directory.appending(path: "rootfs.ext4"))
}
