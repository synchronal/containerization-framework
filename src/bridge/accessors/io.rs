//! What ContainerizationIO's read streams share with Swift. Swift reads no
//! IO value through a getter, so this holds only the test that Rust's
//! constants match Swift's.

#[cfg(test)]
mod tests {
  use crate::bridge::ffi;
  use crate::containerization_io;

  #[test]
  fn copies_swifts_read_stream_buffer_size() {
    assert_eq!(
      ffi::cz_read_stream_buffer_size() as usize,
      containerization_io::ReadStream::BUFFER_SIZE
    );
  }
}
