//! `ImageStore`, and the options for its `pull`, `platforms`, `push`es and
//! `getInitImage`: [`PullOptions`], [`PlatformsOptions`], [`PushOptions`],
//! [`PushAllOptions`] and [`GetInitImageOptions`].

use super::Description;
use super::Image;
use super::InitImage;
use crate::containerization::strings;
use crate::containerization_extras;
use crate::containerization_oci;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;

/// `ImageStore.pull(reference:...)`'s defaulted arguments. [`Default`] uses
/// Swift's defaults: three concurrent downloads, not insecure, and every
/// other argument `nil`.
pub struct PullOptions {
  pub platform: Option<containerization_oci::image::Platform>,
  pub insecure: bool,
  pub auth: Option<containerization_oci::client::Authentication>,
  pub progress: Option<containerization_extras::ProgressHandler>,
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

/// Leaves out the progress handler, which is a closure.
impl fmt::Debug for PullOptions {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("PullOptions")
      .field("platform", &self.platform)
      .field("insecure", &self.insecure)
      .field("auth", &self.auth)
      .field("max_concurrent_downloads", &self.max_concurrent_downloads)
      .finish_non_exhaustive()
  }
}

/// `ImageStore.platforms(reference:...)`'s defaulted arguments. [`Default`]
/// uses Swift's defaults: not insecure, and no `auth`.
#[derive(Debug, Default)]
pub struct PlatformsOptions {
  pub insecure: bool,
  pub auth: Option<containerization_oci::client::Authentication>,
}

/// `ImageStore.push(reference:...)`'s defaulted arguments. [`Default`] uses
/// Swift's defaults: not insecure, and every other argument `nil`.
#[derive(Default)]
pub struct PushOptions {
  pub platform: Option<containerization_oci::image::Platform>,
  pub insecure: bool,
  pub auth: Option<containerization_oci::client::Authentication>,
  pub progress: Option<containerization_extras::ProgressHandler>,
}

/// Leaves out the progress handler, which is a closure.
impl fmt::Debug for PushOptions {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("PushOptions")
      .field("platform", &self.platform)
      .field("insecure", &self.insecure)
      .field("auth", &self.auth)
      .finish_non_exhaustive()
  }
}

/// `ImageStore.push(references:...)`'s defaulted arguments. [`Default`] uses
/// Swift's defaults: three concurrent uploads, not insecure, and every other
/// argument `nil`.
pub struct PushAllOptions {
  pub platform: Option<containerization_oci::image::Platform>,
  pub insecure: bool,
  pub auth: Option<containerization_oci::client::Authentication>,
  pub max_concurrent_uploads: usize,
  pub progress: Option<containerization_extras::ProgressHandler>,
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

/// Leaves out the progress handler, which is a closure.
impl fmt::Debug for PushAllOptions {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("PushAllOptions")
      .field("platform", &self.platform)
      .field("insecure", &self.insecure)
      .field("auth", &self.auth)
      .field("max_concurrent_uploads", &self.max_concurrent_uploads)
      .finish_non_exhaustive()
  }
}

/// `ImageStore.getInitImage(reference:...)`'s defaulted arguments.
/// [`Default`] uses Swift's defaults: no `auth` and no `progress`.
#[derive(Default)]
pub struct GetInitImageOptions {
  pub auth: Option<containerization_oci::client::Authentication>,
  pub progress: Option<containerization_extras::ProgressHandler>,
}

/// Leaves out the progress handler, which is a closure.
impl fmt::Debug for GetInitImageOptions {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("GetInitImageOptions")
      .field("auth", &self.auth)
      .finish_non_exhaustive()
  }
}

/// `ImageStore`.
pub struct ImageStore {
  pub(crate) handle: ffi::CzImageStore,
}

opaque_debug!(ImageStore);

// Swift's `ImageStore` is an actor.
unsafe impl Send for ImageStore {}
unsafe impl Sync for ImageStore {}

impl ImageStore {
  /// `ImageStore(path:)`, with Swift's default `contentStore` of `nil`, which
  /// makes a content store in `path`'s `content` directory.
  pub fn new(path: &Path) -> Result<Self, Error> {
    let outcome = platform::outcome(
      ffi::cz_image_store_new(platform::path(path)?),
      format!("open the image store at {}", path.display()),
    )?;

    Ok(Self {
      handle: outcome.image_store(),
    })
  }

  /// `ImageStore.default`, which lives in the user's Application Support
  /// directory. If Swift can't make it, Swift crashes the process instead of
  /// throwing.
  pub fn default_store() -> Result<Self, Error> {
    platform::outcome(ffi::cz_image_store_default(), "open the default image store").map(|outcome| Self {
      handle: outcome.image_store(),
    })
  }

  /// `ImageStore(path:contentStore:)`.
  pub fn with_content_store(
    path: &Path,
    content_store: &containerization_oci::content::LocalContentStore,
  ) -> Result<Self, Error> {
    let outcome = platform::outcome(
      content_store.handle.image_store(platform::path(path)?),
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

  /// `ImageStore.get(reference:pull:)`. Swift's `pull` defaults to `false`.
  pub fn get(&self, reference: &str, pull: bool) -> Result<Image, Error> {
    platform::outcome(self.handle.get(reference, pull), format!("get {reference}")).map(|outcome| Image {
      handle: outcome.image(),
    })
  }

  /// `ImageStore.getOrNil(reference:)`: the image, or `None` if the store
  /// has no image by that reference.
  pub fn get_or_nil(&self, reference: &str) -> Result<Option<Image>, Error> {
    let outcome = platform::outcome(self.handle.get_or_nil(reference), format!("get {reference}"))?;

    Ok(outcome.is_some().then(|| Image {
      handle: outcome.image(),
    }))
  }

  /// `ImageStore.list()`.
  pub fn list(&self) -> Result<Vec<Image>, Error> {
    platform::outcome(self.handle.list(), "list images").map(|outcome| images(&outcome))
  }

  /// `ImageStore.delete(reference:performCleanup:)`. Swift's
  /// `performCleanup` defaults to `false`.
  pub fn delete(&self, reference: &str, perform_cleanup: bool) -> Result<(), Error> {
    platform::outcome(
      self.handle.delete(reference, perform_cleanup),
      format!("delete {reference}"),
    )
    .map(drop)
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
    let (has_platform, platform) = containerization_oci::image::Platform::crossing(options.platform.as_ref());
    let auth = containerization_oci::client::Authentication::crossing(options.auth.as_ref())?;
    let outcome = platform::progressing(options.progress, |progress| {
      self.handle.pull(
        reference,
        has_platform,
        platform,
        options.insecure,
        auth,
        progress,
        options.max_concurrent_downloads,
      )
    });

    platform::outcome(outcome, action).map(|outcome| Image {
      handle: outcome.image(),
    })
  }

  /// `ImageStore.platforms(reference:insecure:auth:)`: the platforms an image
  /// runs on, read from the store's index when it has the image and from the
  /// registry otherwise, without pulling layers.
  pub fn platforms(
    &self,
    reference: &str,
    options: PlatformsOptions,
  ) -> Result<Vec<containerization_oci::image::Platform>, Error> {
    let auth = containerization_oci::client::Authentication::crossing(options.auth.as_ref())?;

    platform::outcome(
      self.handle.platforms(reference, options.insecure, auth),
      format!("read the platforms of {reference}"),
    )
    .map(|outcome| outcome.platforms())
  }

  /// `ImageStore.push(reference:platform:insecure:auth:progress:)`.
  pub fn push(&self, reference: &str, options: PushOptions) -> Result<(), Error> {
    let action = format!("push {reference}");
    let (has_platform, platform) = containerization_oci::image::Platform::crossing(options.platform.as_ref());
    let auth = containerization_oci::client::Authentication::crossing(options.auth.as_ref())?;
    let outcome = platform::progressing(options.progress, |progress| {
      self
        .handle
        .push(reference, has_platform, platform, options.insecure, auth, progress)
    });

    platform::outcome(outcome, action).map(drop)
  }

  /// `ImageStore.push(references:platform:insecure:auth:maxConcurrentUploads:progress:)`.
  /// Rust has no overloading, and the suffix tells it apart from
  /// [`Self::push`].
  pub fn push_all(&self, references: &[&str], options: PushAllOptions) -> Result<(), Error> {
    let (has_platform, platform) = containerization_oci::image::Platform::crossing(options.platform.as_ref());
    let auth = containerization_oci::client::Authentication::crossing(options.auth.as_ref())?;
    let outcome = platform::progressing(options.progress, |progress| {
      self.handle.push_all(
        strings(references),
        has_platform,
        platform,
        options.insecure,
        auth,
        options.max_concurrent_uploads,
        progress,
      )
    });

    platform::outcome(outcome, format!("push {}", references.join(", "))).map(drop)
  }

  /// `ImageStore.getInitImage(reference:auth:progress:)`.
  pub fn get_init_image(&self, reference: &str, options: GetInitImageOptions) -> Result<InitImage, Error> {
    let auth = containerization_oci::client::Authentication::crossing(options.auth.as_ref())?;

    platform::outcome(
      platform::progressing(options.progress, |progress| {
        self.handle.get_init_image(reference, auth, progress)
      }),
      format!("get init image {reference}"),
    )
    .map(|outcome| InitImage {
      handle: outcome.init_image(),
    })
  }

  /// `ImageStore.save(references:out:platform:)`. Swift's `platform`
  /// defaults to `nil`.
  pub fn save(
    &self,
    references: &[&str],
    out: &Path,
    platform: Option<&containerization_oci::image::Platform>,
  ) -> Result<(), Error> {
    let (has_platform, platform) = containerization_oci::image::Platform::crossing(platform);

    platform::outcome(
      self
        .handle
        .save(strings(references), platform::path(out)?, has_platform, platform),
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

  /// `ImageStore.load(from:progress:)`. Swift's `progress` defaults to
  /// `nil`.
  pub fn load(
    &self,
    directory: &Path,
    progress: Option<containerization_extras::ProgressHandler>,
  ) -> Result<Vec<Image>, Error> {
    let path = platform::path(directory)?;

    platform::outcome(
      platform::progressing(progress, |progress| self.handle.load(path, progress)),
      format!("load images from {}", directory.display()),
    )
    .map(|outcome| images(&outcome))
  }

  /// `ImageStore.cleanUpOrphanedBlobs()`, which returns the names of the blobs it
  /// deleted and the bytes it freed.
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

/// Returns the `[Image]` that an outcome holds.
fn images(outcome: &ffi::CzOutcome) -> Vec<Image> {
  let images = outcome.images();

  (0..images.len())
    .map(|index| Image {
      handle: images.at(index),
    })
    .collect()
}
