#![cfg(feature = "integration")]

//! `ReadStream`, from ContainerizationIO.

use containerization_framework as cfw;

#[test]
fn streams_data_in_chunks_of_its_buffer_size() {
  let stream = cfw::containerization_io::ReadStream::with_data(b"abcdefghij", 4).expect("a stream");

  assert_eq!(
    stream.data_stream().collect::<Vec<_>>(),
    [b"abcd".to_vec(), b"efgh".to_vec(), b"ij".to_vec()]
  );
}

#[test]
fn streams_a_file() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let file = directory.path().join("data.bin");
  let data: Vec<u8> = (0..=255).cycle().take(3000).collect();
  std::fs::write(&file, &data).expect("a file");

  let stream = cfw::containerization_io::ReadStream::with_url(&file, 1024).expect("a stream");

  assert_eq!(stream.data_stream().flatten().collect::<Vec<_>>(), data);
}

#[test]
fn streams_nothing_when_empty() {
  let stream = cfw::containerization_io::ReadStream::new().expect("a stream");

  assert_eq!(stream.data_stream().count(), 0);
}

#[test]
fn streams_again_only_after_a_reset() {
  let stream = cfw::containerization_io::ReadStream::with_data(b"once", 2).expect("a stream");

  assert_eq!(stream.data_stream().count(), 2);
  assert_eq!(stream.data_stream().count(), 0, "the source was read");

  stream.reset().expect("the stream should reset");

  assert_eq!(stream.data_stream().flatten().collect::<Vec<_>>(), b"once");
}

#[test]
fn says_which_file_it_could_not_stream() {
  let error = cfw::containerization_io::ReadStream::with_url(
    "/nowhere/data.bin".as_ref(),
    cfw::containerization_io::ReadStream::BUFFER_SIZE,
  )
  .err()
  .expect("a file that doesn't exist");

  assert!(error.to_string().contains("/nowhere/data.bin"), "{error}");
}
