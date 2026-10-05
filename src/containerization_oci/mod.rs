//! Containerization's `ContainerizationOCI` module.
//!
//! Each type wraps the Swift type of the same name, and its methods are the
//! Swift methods written in snake case.

/// A Swift enum backed by strings: Rust's cases, each with its `rawValue`.
macro_rules! raw_values {
  ($(#[$meta:meta])* $name:ident { $($case:ident = $raw:literal),* $(,)? }) => {
    $(#[$meta])*
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub enum $name {
      $(#[doc = concat!("`", $raw, "`.")] $case),*
    }

    impl $name {
      /// Every case, in the order the bridge lists Swift's raw values.
      #[cfg(all(test, target_os = "macos"))]
      pub(crate) const ALL: &[Self] = &[$(Self::$case),*];

      /// `rawValue`.
      pub fn raw_value(self) -> &'static str {
        match self {
          $(Self::$case => $raw),*
        }
      }

      /// `init?(rawValue:)`.
      pub fn from_raw_value(raw_value: &str) -> Option<Self> {
        match raw_value {
          $($raw => Some(Self::$case),)*
          _ => None,
        }
      }

      /// A raw value from Swift. It panics if Swift has a case Rust lacks.
      /// `ContainerState` never uses it.
      #[allow(dead_code)]
      pub(crate) fn from_swift(raw_value: &str) -> Self {
        Self::from_raw_value(raw_value)
          .unwrap_or_else(|| panic!("Swift's {} has a case Rust lacks: {raw_value:?}", stringify!($name)))
      }
    }
  };
}

mod annotation_keys;
mod bundle;
mod content;
mod content_writer;
mod descriptor;
mod image_config;
mod index;
mod local_content_store;
mod manifest;
mod media_types;
mod parsed_digest;
mod platform;
pub mod reference;
mod spec;
mod state;
mod user;
mod version;

pub use self::annotation_keys::AnnotationKeys;
pub use self::bundle::Bundle;
pub use self::content::Content;
pub use self::content_writer::ContentWriter;
pub use self::descriptor::Descriptor;
pub use self::image_config::History;
pub use self::image_config::Image;
pub use self::image_config::ImageConfig;
pub use self::image_config::Rootfs;
pub use self::index::Index;
pub use self::local_content_store::LocalContentStore;
pub use self::manifest::Manifest;
pub use self::media_types::MediaTypes;
pub use self::parsed_digest::ParsedDigest;
pub use self::platform::Platform;
pub use self::reference::Reference;
pub use self::spec::Arch;
pub use self::spec::Box;
pub use self::spec::Hook;
pub use self::spec::Hooks;
pub use self::spec::Linux;
pub use self::spec::LinuxBlockIO;
pub use self::spec::LinuxBlockIODevice;
pub use self::spec::LinuxCPU;
pub use self::spec::LinuxCapabilities;
pub use self::spec::LinuxDevice;
pub use self::spec::LinuxDeviceCgroup;
pub use self::spec::LinuxHugepageLimit;
pub use self::spec::LinuxIDMapping;
pub use self::spec::LinuxInterfacePriority;
pub use self::spec::LinuxMemory;
pub use self::spec::LinuxNamespace;
pub use self::spec::LinuxNamespaceType;
pub use self::spec::LinuxNetwork;
pub use self::spec::LinuxPersonality;
pub use self::spec::LinuxPersonalityDomain;
pub use self::spec::LinuxPids;
pub use self::spec::LinuxRdma;
pub use self::spec::LinuxResources;
pub use self::spec::LinuxSeccomp;
pub use self::spec::LinuxSeccompAction;
pub use self::spec::LinuxSeccompArg;
pub use self::spec::LinuxSeccompFlag;
pub use self::spec::LinuxSeccompOperator;
pub use self::spec::LinuxSyscall;
pub use self::spec::LinuxThrottleDevice;
pub use self::spec::LinuxWeightDevice;
pub use self::spec::Mount;
pub use self::spec::POSIXRlimit;
pub use self::spec::Process;
pub use self::spec::Root;
pub use self::spec::Spec;
pub use self::state::ContainerProcessState;
pub use self::state::ContainerState;
pub use self::state::SECCOMP_FD_NAME;
pub use self::state::State;
pub use self::user::User;
pub use self::version::RuntimeSpecVersion;
