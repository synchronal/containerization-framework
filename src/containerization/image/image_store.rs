//! `ImageStore`, and the options for its `pull` and `push`es:
//! [`PullOptions`], [`PushOptions`] and [`PushAllOptions`].

use super::Description;
use super::Image;
use super::InitImage;
use crate::containerization_extras::ProgressHandler;
use crate::containerization_oci;
use crate::containerization_oci::client::Authentication;
use crate::containerization_oci::client::authentication;
use crate::containerization_oci::content::LocalContentStore;
use crate::containerization_oci::image::Platform;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::path::Path;
use std::path::PathBuf;

/// `ImageStore.pull(reference:...)`'s defaulted arguments. [`Default`] is
/// Swift's defaults.
pub struct PullOptions {
  pub platform: Option<Platform>,
  pub insecure: bool,
  pub auth: Option<Authentication>,
  pub progress: Option<ProgressHandler>,
  pub max_concurrent_downloads: usize,
}

impl Default for PullOptions {
  fn default() -> Self {
    Self {
      platform: None,
      insecure: false,
      auth: None,
      progress: None,
      max_concurrent_downloads: 3,
    }
  }
}

/// `ImageStore.push(reference:...)`'s defaulted arguments. [`Default`] is
/// Swift's defaults.
#[derive(Default)]
pub struct PushOptions {
  pub platform: Option<Platform>,
  pub insecure: bool,
  pub auth: Option<Authentication>,
  pub progress: Option<ProgressHandler>,
}

/// `ImageStore.push(references:...)`'s defaulted arguments. [`Default`] is
/// Swift's defaults.
pub struct PushAllOptions {
  pub platform: Option<Platform>,
  pub insecure: bool,
  pub auth: Option<Authentication>,
  pub max_concurrent_uploads: usize,
  pub progress: Option<ProgressHandler>,
}

impl Default for PushAllOptions {
  fn default() -> Self {
    Self {
      platform: None,
      insecure: false,
      auth: None,
      max_concurrent_uploads: 3,
      progress: None,
    }
  }
}

/// `ImageStore`.
pub struct ImageStore {
  pub(crate) handle: ffi::CzImageStore,
}

// Swift's `ImageStore` is an actor.
unsafe impl Send for ImageStore {}
unsafe impl Sync for ImageStore {}

impl ImageStore {
  /// `ImageStore(path:)`.
  pub fn new(path: &Path) -> Result<Self, Error> {
    let outcome = platform::outcome(
      ffi::cz_image_store_new(&path.display().to_string()),
      format!("open the image store at {}", path.display()),
    )?;

    Ok(Self {
      handle: outcome.image_store(),
    })
  }

  /// `ImageStore.default`, in the user's Application Support directory.
  /// Swift's crashes, rather than throwing, if it can't be made.
  pub fn default_store() -> Result<Self, Error> {
    platform::outcome(ffi::cz_image_store_default(), "open the default image store").map(|outcome| Self {
      handle: outcome.image_store(),
    })
  }

  /// `ImageStore(path:contentStore:)`.
  pub fn with_content_store(path: &Path, content_store: &LocalContentStore) -> Result<Self, Error> {
    let outcome = platform::outcome(
      content_store
        .handle
        .image_store(&path.display().to_string()),
      format!("open the image store at {}", path.display()),
    )?;

    Ok(Self {
      handle: outcome.image_store(),
    })
  }

  /// `ImageStore.path`.
  pub fn path(&self) -> PathBuf {
    PathBuf::from(self.handle.path())
  }

  /// `ImageStore.get(reference:pull:)`.
  pub fn get(&self, reference: &str, pull: bool) -> Result<Image, Error> {
    platform::outcome(self.handle.get(reference, pull), format!("get {reference}")).map(|outcome| Image {
      handle: outcome.image(),
    })
  }

  /// `ImageStore.list()`.
  pub fn list(&self) -> Result<Vec<Image>, Error> {
    let images = platform::outcome(self.handle.list(), "list images")?.images();

    Ok(
      (0..images.len())
        .map(|index| Image {
          handle: images.at(index),
        })
        .collect(),
    )
  }

  /// `ImageStore.delete(reference:performCleanup:)`.
  pub fn delete(&self, reference: &str, perform_cleanup: bool) -> Result<(), Error> {
    platform::outcome(
      self.handle.delete(reference, perform_cleanup),
      format!("delete {reference}"),
    )
    .map(|_| ())
  }

  /// `ImageStore.tag(existing:new:)`.
  pub fn tag(&self, existing: &str, new: &str) -> Result<Image, Error> {
    platform::outcome(self.handle.tag(existing, new), format!("tag {existing} as {new}")).map(|outcome| Image {
      handle: outcome.image(),
    })
  }

  /// `ImageStore.pull(reference:platform:insecure:auth:progress:maxConcurrentDownloads:)`.
  pub fn pull(&self, reference: &str, options: PullOptions) -> Result<Image, Error> {
    let action = format!("pull {reference}");
    let (has_platform, platform) = containerization_oci::image::platform::crossing(options.platform.as_ref());
    let outcome = self.handle.pull(
      reference,
      has_platform,
      platform,
      options.insecure,
      authentication::crossing(options.auth.as_ref())?,
      platform::Progress(options.progress),
      options.max_concurrent_downloads,
    );

    platform::outcome(outcome, action).map(|outcome| Image {
      handle: outcome.image(),
    })
  }

  /// `ImageStore.push(reference:platform:insecure:auth:progress:)`.
  pub fn push(&self, reference: &str, options: PushOptions) -> Result<(), Error> {
    let action = format!("push {reference}");
    let (has_platform, platform) = containerization_oci::image::platform::crossing(options.platform.as_ref());
    let outcome = self.handle.push(
      reference,
      has_platform,
      platform,
      options.insecure,
      authentication::crossing(options.auth.as_ref())?,
      platform::Progress(options.progress),
    );

    platform::outcome(outcome, action).map(drop)
  }

  /// `ImageStore.push(references:platform:insecure:auth:maxConcurrentUploads:progress:)`.
  /// Rust has no overloading, and the suffix tells it apart from
  /// [`Self::push`].
  pub fn push_all(&self, references: &[&str], options: PushAllOptions) -> Result<(), Error> {
    let (has_platform, platform) = containerization_oci::image::platform::crossing(options.platform.as_ref());
    let outcome = self.handle.push_all(
      references
        .iter()
        .map(|reference| reference.to_string())
        .collect(),
      has_platform,
      platform,
      options.insecure,
      authentication::crossing(options.auth.as_ref())?,
      options.max_concurrent_uploads,
      platform::Progress(options.progress),
    );

    platform::outcome(outcome, format!("push {}", references.join(", "))).map(drop)
  }

  /// `ImageStore.getInitImage(reference:auth:progress:)`.
  pub fn get_init_image(
    &self,
    reference: &str,
    auth: Option<&Authentication>,
    progress: Option<ProgressHandler>,
  ) -> Result<InitImage, Error> {
    platform::outcome(
      self
        .handle
        .get_init_image(reference, authentication::crossing(auth)?, platform::Progress(progress)),
      format!("get init image {reference}"),
    )
    .map(|outcome| InitImage {
      handle: outcome.init_image(),
    })
  }

  /// `ImageStore.save(references:out:platform:)`.
  pub fn save(&self, references: &[&str], out: &Path, platform: Option<&Platform>) -> Result<(), Error> {
    let (has_platform, platform) = containerization_oci::image::platform::crossing(platform);

    platform::outcome(
      self.handle.save(
        references
          .iter()
          .map(|reference| reference.to_string())
          .collect(),
        &out.display().to_string(),
        has_platform,
        platform,
      ),
      format!("save {} to {}", references.join(", "), out.display()),
    )
    .map(drop)
  }

  /// `ImageStore.create(description:)`.
  pub fn create(&self, description: &Description) -> Result<Image, Error> {
    platform::outcome(
      self.handle.create(description.clone()),
      format!("create {}", description.reference),
    )
    .map(|outcome| Image {
      handle: outcome.image(),
    })
  }

  /// `ImageStore.load(from:progress:)`.
  pub fn load(&self, directory: &Path, progress: Option<ProgressHandler>) -> Result<Vec<Image>, Error> {
    let images = platform::outcome(
      self
        .handle
        .load(&directory.display().to_string(), platform::Progress(progress)),
      format!("load images from {}", directory.display()),
    )?
    .images();

    Ok(
      (0..images.len())
        .map(|index| Image {
          handle: images.at(index),
        })
        .collect(),
    )
  }

  /// `ImageStore.cleanUpOrphanedBlobs()`: the digests deleted, and the bytes
  /// freed.
  pub fn clean_up_orphaned_blobs(&self) -> Result<(Vec<String>, u64), Error> {
    let outcome = platform::outcome(self.handle.clean_up_orphaned_blobs(), "clean up orphaned blobs")?;

    Ok((outcome.strings(), outcome.number()))
  }

  /// `ImageStore.calculateOrphanedBlobsSize()`.
  pub fn calculate_orphaned_blobs_size(&self) -> Result<u64, Error> {
    platform::outcome(self.handle.calculate_orphaned_blobs_size(), "size orphaned blobs")
      .map(|outcome| outcome.number())
  }
}
