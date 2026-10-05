use super::ArchiveReader;
use crate::platform::ffi;
use std::io;
use std::marker::PhantomData;

/// `ArchiveEntryReader`, which reads the current entry's data from the archive
/// it came from. Its `read(_:maxLength:)` is [`io::Read::read`].
pub struct ArchiveEntryReader<'a> {
  pub(crate) handle: ffi::CzArchiveEntryReader,
  pub(crate) reader: PhantomData<&'a ArchiveReader>,
}

impl io::Read for ArchiveEntryReader<'_> {
  /// `ArchiveEntryReader.read(_:maxLength:)`. Swift's `-1` is an error.
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    let read = self.handle.read(buf.len());
    if let Some(message) = read.error() {
      return Err(io::Error::other(message));
    }

    let bytes = read.bytes();
    buf[..bytes.len()].copy_from_slice(&bytes);
    Ok(bytes.len())
  }
}
