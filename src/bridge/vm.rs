//! The values a VM or pod is configured with, and the emulator its guest
//! agent sets up, which Swift reads.

use crate::containerization::container::linux_pod::PodVolume as RustPodVolume;
use crate::containerization::vm::AttachedFilesystem as RustAttachedFilesystem;
use crate::containerization::vm::BootLog as RustBootLog;
use crate::containerization::vm::Kernel as RustKernel;
use crate::containerization::vm::SystemPlatform as RustSystemPlatform;
use crate::containerization_os::binfmt::Entry as RustBinfmtEntry;

use super::ffi::BootLogKind;
use super::ffi::PlatformArchitecture;
use super::ffi::PlatformOs;
use super::ffi::PodVolumeKind;

#[swift_bridge::bridge]
mod ffi {
  #[swift_bridge(already_declared)]
  enum BootLogKind {}

  #[swift_bridge(already_declared)]
  enum PlatformOs {}

  #[swift_bridge(already_declared)]
  enum PlatformArchitecture {}

  #[swift_bridge(already_declared)]
  enum PodVolumeKind {}

  extern "Rust" {
    type RustBootLog;
    fn kind(self: &RustBootLog) -> BootLogKind;
    fn path(self: &RustBootLog) -> String;
    fn append(self: &RustBootLog) -> bool;
    #[swift_bridge(swift_name = "fileHandle")]
    fn file_handle(self: &RustBootLog) -> i32;

    // `location` is an `nbd` source's URL or a `diskImage`'s path.
    type RustPodVolume;
    fn name(self: &RustPodVolume) -> &str;
    fn format(self: &RustPodVolume) -> &str;
    #[swift_bridge(swift_name = "sourceKind")]
    fn source_kind(self: &RustPodVolume) -> PodVolumeKind;
    fn location(self: &RustPodVolume) -> String;
    fn timeout(self: &RustPodVolume) -> Option<f64>;
    #[swift_bridge(swift_name = "readOnly")]
    fn read_only(self: &RustPodVolume) -> bool;
    #[swift_bridge(swift_name = "sizeBytes")]
    fn size_bytes(self: &RustPodVolume) -> Option<u64>;

    type RustAttachedFilesystem;
    #[swift_bridge(swift_name = "filesystemType")]
    fn filesystem_type(self: &RustAttachedFilesystem) -> &str;
    fn source(self: &RustAttachedFilesystem) -> &str;
    fn destination(self: &RustAttachedFilesystem) -> &str;
    #[swift_bridge(swift_name = "optionsLen")]
    fn options_len(self: &RustAttachedFilesystem) -> usize;
    #[swift_bridge(swift_name = "optionsAt")]
    fn options_at(self: &RustAttachedFilesystem, index: usize) -> &str;

    type RustSystemPlatform;
    fn os(self: &RustSystemPlatform) -> PlatformOs;
    fn architecture(self: &RustSystemPlatform) -> PlatformArchitecture;

    // The emulator `Vminitd.setupEmulator` registers. (`type` is a Swift
    // keyword.)
    type RustBinfmtEntry;
    fn name(self: &RustBinfmtEntry) -> &str;
    #[swift_bridge(swift_name = "entryType")]
    fn entry_type(self: &RustBinfmtEntry) -> &str;
    fn offset(self: &RustBinfmtEntry) -> &str;
    fn magic(self: &RustBinfmtEntry) -> &str;
    fn mask(self: &RustBinfmtEntry) -> &str;
    fn flags(self: &RustBinfmtEntry) -> &str;

    type RustKernel;
    fn path(self: &RustKernel) -> String;
    fn platform(self: &RustKernel) -> &RustSystemPlatform;
    #[swift_bridge(swift_name = "kernelArgsLen")]
    fn kernel_args_len(self: &RustKernel) -> usize;
    #[swift_bridge(swift_name = "kernelArgsAt")]
    fn kernel_args_at(self: &RustKernel, index: usize) -> &str;
    #[swift_bridge(swift_name = "initArgsLen")]
    fn init_args_len(self: &RustKernel) -> usize;
    #[swift_bridge(swift_name = "initArgsAt")]
    fn init_args_at(self: &RustKernel, index: usize) -> &str;
  }
}
