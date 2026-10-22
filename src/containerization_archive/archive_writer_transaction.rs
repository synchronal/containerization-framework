use super::ArchiveWriter;
use super::WriteEntry;
use crate::error::Error;
use crate::platform;
use crate::platform::ffi;
use std::marker::PhantomData;

/// `ArchiveWriterTransaction`. It writes through its writer's archive, so it
/// borrows the writer. Swift's is a class that isn't `Sendable`, so Rust's is
/// neither `Send` nor `Sync`.
///
/// ```compile_fail
/// # use containerization_framework as cfw;
/// # let configuration = cfw::containerization_archive::ArchiveWriterConfiguration::new(
/// #   cfw::containerization_archive::Format::Ustar,
/// #   cfw::containerization_archive::Filter::None,
/// # );
/// let writer = cfw::containerization_archive::ArchiveWriter::new(&configuration)?;
/// let transaction = writer.make_transaction_writer();
/// std::thread::scope(|scope| {
///   scope.spawn(move || transaction.finish());
/// });
/// # Ok::<(), cfw::Error>(())
/// ```
pub struct ArchiveWriterTransaction<'a> {
  pub(super) handle: ffi::CzArchiveWriterTransaction,
  pub(super) writer: PhantomData<&'a ArchiveWriter>,
}

opaque_debug!(ArchiveWriterTransaction<'_>);

impl ArchiveWriterTransaction<'_> {
  /// `ArchiveWriterTransaction.writeHeader(entry:)`.
  pub fn write_header(&self, entry: &WriteEntry) -> Result<(), Error> {
    platform::outcome(
      self.handle.write_header(entry.handle.duplicate()),
      format!("write the header of {} to an archive", entry.path().unwrap_or_default()),
    )
    .map(drop)
  }

  /// `ArchiveWriterTransaction.writeChunk(data:)`.
  pub fn write_chunk(&self, data: &[u8]) -> Result<(), Error> {
    platform::outcome(self.handle.write_chunk(data.to_vec()), "write a chunk to an archive").map(drop)
  }

  /// `ArchiveWriterTransaction.finish()`.
  pub fn finish(&self) -> Result<(), Error> {
    platform::outcome(self.handle.finish(), "finish an archive entry").map(drop)
  }
}
