//! Containerization's `Containerization` module.
//!
//! Each type wraps the Swift type of the same name, and its methods are the
//! Swift methods written in snake case. A Swift type nested in another, like
//! `LinuxContainer.Configuration`, is in a module named after its parent:
//! `linux_container::Configuration`.

mod boot_log;
pub mod container_manager;
mod dns;
mod exit_status;
mod ext4_unpacker;
pub mod hosts;
pub mod image;
pub mod image_store;
mod init_image;
pub mod kernel;
mod kernel_image;
mod linux_capabilities;
pub mod linux_container;
mod linux_process;
mod linux_process_configuration;
pub mod linux_rlimit;
pub mod mount;
mod nat_interface;
pub mod signal;
pub mod system_platform;
pub mod unix_socket_configuration;
mod vm_resources;

pub use self::boot_log::BootLog;
pub use self::container_manager::ContainerManager;
pub use self::dns::Dns;
pub use self::exit_status::ExitStatus;
pub use self::ext4_unpacker::Ext4Unpacker;
pub use self::hosts::Hosts;
pub use self::image::Image;
pub use self::image_store::ImageStore;
pub use self::init_image::InitImage;
pub use self::kernel::Kernel;
pub use self::kernel_image::KernelImage;
pub use self::linux_capabilities::LinuxCapabilities;
pub use self::linux_container::LinuxContainer;
pub use self::linux_process::LinuxProcess;
pub use self::linux_process_configuration::LinuxProcessConfiguration;
pub use self::linux_rlimit::LinuxRLimit;
pub use self::mount::Mount;
pub use self::nat_interface::NatInterface;
pub use self::signal::Signal;
pub use self::system_platform::SystemPlatform;
pub use self::unix_socket_configuration::UnixSocketConfiguration;
pub use self::vm_resources::VmResources;

const MIB: u64 = 1024 * 1024;
const GIB: u64 = 1024 * MIB;

fn strings(values: &[&str]) -> Vec<String> {
  values.iter().map(|value| value.to_string()).collect()
}
