//! Stand-ins for `Reference.swift`: `Reference` and `ParsedDigest`.

use super::CzOutcome;
use std::convert::Infallible;

taken!(
  reference -> CzReference,
  parsed_digest_encoded -> String,
);

handles!(CzReference);

failing!(
  cz_reference_new(&str, Option<String>, Option<String>, Option<String>),
  cz_reference_parse(&str),
  cz_reference_with_name(&str),
  cz_reference_resolve_domain(&str),
  cz_parsed_digest_parse(&str),
  cz_parsed_digest_parse_path_component(&str),
  cz_parsed_digest_is_valid(&str),
  cz_parsed_digest_description(&str),
  cz_parsed_digest_path(&str, &str),
);

uninhabited!(CzReference {
  fn domain() -> Option<String>;
  fn resolved_domain() -> Option<String>;
  fn path() -> String;
  fn tag() -> Option<String>;
  fn digest() -> Option<String>;
  fn name() -> String;
  fn description() -> String;
  fn with_tag(&str) -> CzOutcome;
  fn with_digest(&str) -> CzOutcome;
  fn normalize();
});
