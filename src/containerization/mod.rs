//! Containerization's `Containerization` module.
//!
//! Each type wraps the Swift type of the same name, and its methods are the
//! Swift methods written in snake case. A Swift type nested in another, like
//! `LinuxContainer.Configuration`, is in a module named after its parent:
//! `linux_container::Configuration`.

mod attached_filesystem;
mod boot_log;
pub mod container_manager;
pub mod container_statistics;
mod dns;
mod exit_status;
mod ext4_unpacker;
mod filesystem_operation;
pub mod hosts;
pub mod image;
pub mod image_store;
mod init_image;
mod interface;
pub mod kernel;
mod kernel_image;
mod linux_capabilities;
pub mod linux_container;
pub mod linux_pod;
mod linux_process;
mod linux_process_configuration;
pub mod linux_rlimit;
pub mod mount;
mod nat_interface;
pub mod signal;
mod stat_category;
pub mod system_platform;
pub mod unix_socket_configuration;
mod virtual_machine_instance;
mod vm_configuration;
mod vm_resources;
pub mod vmnet_network;
mod vsock_listener;
mod vz_virtual_machine_instance;
pub mod vz_virtual_machine_manager;

pub use self::attached_filesystem::AttachedFilesystem;
pub use self::boot_log::BootLog;
pub use self::container_manager::ContainerManager;
pub use self::container_statistics::ContainerStatistics;
pub use self::dns::Dns;
pub use self::exit_status::ExitStatus;
pub use self::ext4_unpacker::Ext4Unpacker;
pub use self::filesystem_operation::FilesystemOperation;
pub use self::hosts::Hosts;
pub use self::image::Image;
pub use self::image_store::ImageStore;
pub use self::init_image::InitImage;
pub use self::interface::Interface;
pub use self::kernel::Kernel;
pub use self::kernel_image::KernelImage;
pub use self::linux_capabilities::LinuxCapabilities;
pub use self::linux_container::LinuxContainer;
pub use self::linux_pod::LinuxPod;
pub use self::linux_process::LinuxProcess;
pub use self::linux_process_configuration::LinuxProcessConfiguration;
pub use self::linux_rlimit::LinuxRLimit;
pub use self::mount::Mount;
pub use self::nat_interface::NatInterface;
pub use self::signal::Signal;
pub use self::stat_category::StatCategory;
pub use self::system_platform::SystemPlatform;
pub use self::unix_socket_configuration::UnixSocketConfiguration;
pub use self::virtual_machine_instance::VirtiofsLayout;
pub use self::virtual_machine_instance::VirtualMachineInstanceState;
pub use self::vm_configuration::VmConfiguration;
pub use self::vm_resources::VmResources;
pub use self::vmnet_network::VmnetNetwork;
pub use self::vsock_listener::VsockListener;
pub use self::vz_virtual_machine_instance::VzVirtualMachineInstance;
pub use self::vz_virtual_machine_manager::VzVirtualMachineManager;

const MIB: u64 = 1024 * 1024;
const GIB: u64 = 1024 * MIB;

fn strings(values: &[&str]) -> Vec<String> {
  values.iter().map(|value| value.to_string()).collect()
}
