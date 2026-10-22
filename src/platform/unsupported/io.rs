//! Stand-ins for `IO.swift`: ContainerizationIO's `ReadStream`.

use super::CzOutcome;
use std::convert::Infallible;

taken!(read_stream -> CzReadStream);

handles!(CzReadStream, CzDataStream);

failing!(
  cz_read_stream_new(),
  cz_read_stream_with_url(&str, usize),
  cz_read_stream_with_data(Vec<u8>, usize),
);

uninhabited!(CzReadStream {
  fn reset() -> CzOutcome;
  fn data_stream() -> CzDataStream;
});

uninhabited!(CzDataStream {
  fn next() -> CzOutcome;
});
