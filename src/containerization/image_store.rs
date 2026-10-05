use super::Image;
use super::InitImage;
use super::image;
use crate::containerization_extras::ProgressHandler;
use crate::containerization_oci::LocalContentStore;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::path::Path;
use std::path::PathBuf;

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

  /// `ImageStore.pull(reference:)`, its other arguments at their defaults.
  pub fn pull(&self, reference: &str) -> Result<Image, Error> {
    platform::outcome(self.handle.pull(reference), format!("pull {reference}")).map(|outcome| Image {
      handle: outcome.image(),
    })
  }

  /// `ImageStore.getInitImage(reference:)`.
  pub fn get_init_image(&self, reference: &str) -> Result<InitImage, Error> {
    platform::outcome(
      self.handle.get_init_image(reference),
      format!("get init image {reference}"),
    )
    .map(|outcome| InitImage {
      handle: outcome.init_image(),
    })
  }

  /// `ImageStore.create(description:)`.
  pub fn create(&self, description: &image::Description) -> Result<Image, Error> {
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
