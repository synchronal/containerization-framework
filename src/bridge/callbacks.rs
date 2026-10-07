//! The closures Rust hands Swift, each in an opaque type with a `call` method.

use crate::containerization::container::linux_container::Configuration as RustLinuxContainerConfiguration;
use crate::containerization::container::linux_pod::Configuration as RustPodConfiguration;
use crate::containerization::container::linux_pod::ContainerConfiguration as RustPodContainerConfiguration;
use crate::containerization::process::LinuxProcessConfiguration as RustLinuxProcessConfiguration;
use crate::platform::ConfigureContainer as RustConfigure;
use crate::platform::ConfigurePod as RustConfigurePod;
use crate::platform::ConfigurePodContainer as RustConfigurePodContainer;
use crate::platform::ConfigureProcess as RustConfigureProcess;
use crate::platform::Progress as RustProgressHandler;

use super::ffi::ProgressKind;

#[swift_bridge::bridge]
mod ffi {
  #[swift_bridge(already_declared)]
  enum ProgressKind {}

  extern "Rust" {
    #[swift_bridge(already_declared)]
    type RustLinuxContainerConfiguration;
    #[swift_bridge(already_declared)]
    type RustLinuxProcessConfiguration;
    #[swift_bridge(already_declared)]
    type RustPodConfiguration;
    #[swift_bridge(already_declared)]
    type RustPodContainerConfiguration;

    // A `(inout LinuxContainer.Configuration) -> Void`: Swift calls it once,
    // with the configuration it seeded.
    type RustConfigure;
    fn call(self: &RustConfigure, configuration: &mut RustLinuxContainerConfiguration);

    // A `(inout LinuxProcessConfiguration) -> Void`: Swift calls it once,
    // with the configuration it filled.
    type RustConfigureProcess;
    fn call(self: &RustConfigureProcess, process: &mut RustLinuxProcessConfiguration);

    // A `(inout LinuxPod.Configuration) -> Void`, which takes what Swift
    // filled, and a `(inout LinuxPod.ContainerConfiguration) -> Void`, which
    // Swift calls once on Rust's default, which is Swift's.
    type RustConfigurePod;
    fn call(self: &RustConfigurePod, configuration: &mut RustPodConfiguration);
    type RustConfigurePodContainer;
    fn call(self: &RustConfigurePodContainer, configuration: &mut RustPodContainerConfiguration);

    // A `ProgressHandler?`. Swift asks `is_some` before calling it.
    type RustProgressHandler;
    #[swift_bridge(swift_name = "isSome")]
    fn is_some(self: &RustProgressHandler) -> bool;
    fn call(self: &RustProgressHandler, kinds: Vec<ProgressKind>, values: Vec<i64>);
  }
}
