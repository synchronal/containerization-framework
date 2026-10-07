//! Containerization's `Containerization` module.
//!
//! Each type wraps the Swift type of the same name, and its methods are the
//! Swift methods written in snake case. The types are grouped by what they
//! work on: images, VMs, networks, containers and processes. A Swift type
//! nested in another, like `LinuxContainer.Configuration`, is in a module
//! named after its parent, inside the parent's group:
//! `container::linux_container::Configuration`.

pub mod container;
pub mod image;
pub mod network;
pub mod process;
pub mod vm;

const MIB: u64 = 1024 * 1024;
const GIB: u64 = 1024 * MIB;

fn strings(values: &[&str]) -> Vec<String> {
  values.iter().map(|value| value.to_string()).collect()
}
