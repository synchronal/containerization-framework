//! Stand-ins for `ContentWriter.swift`.

use super::CzOutcome;
use std::convert::Infallible;

taken!(
  content_writer -> CzContentWriter,
  written_size -> i64,
  written_digest -> String,
);

handles!(CzContentWriter);

failing!(cz_content_writer_new(&str), cz_content_writer_copy(&str, &str));

uninhabited!(CzContentWriter {
  fn write(Vec<u8>) -> CzOutcome;
  fn create(&str) -> CzOutcome;
});
