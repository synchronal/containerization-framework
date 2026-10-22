//! Stand-ins for `LocalContentStore.swift`: `LocalContentStore` and
//! `Content`.

use super::CzOutcome;
use std::convert::Infallible;

taken!(
  local_content_store -> CzLocalContentStore,
  content -> CzContent,
  ingest_session_id -> String,
  ingest_session_directory -> String,
);

handles!(CzLocalContentStore, CzContent);

failing!(cz_local_content_store_new(&str), cz_local_content_open(&str));

uninhabited!(CzLocalContentStore {
  fn get(&str) -> CzOutcome;
  fn duplicate() -> CzLocalContentStore;
  fn new_ingest_session() -> CzOutcome;
  fn complete_ingest_session(&str) -> CzOutcome;
  fn cancel_ingest_session(&str) -> CzOutcome;
  fn delete_digests(Vec<String>) -> CzOutcome;
  fn delete_keeping(Vec<String>) -> CzOutcome;
  fn total_allocated_size() -> CzOutcome;
  fn ingest(Box<dyn FnOnce(String) -> bool>) -> CzOutcome;
});

uninhabited!(CzContent {
  fn is_some() -> bool;
  fn path() -> String;
  fn digest() -> CzOutcome;
  fn size() -> CzOutcome;
  fn data() -> CzOutcome;
  fn data_range(u64, usize) -> CzOutcome;
});
