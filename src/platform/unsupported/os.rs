//! Terminals, file information, the keychain and sysctl.

use super::CzOutcome;
use crate::containerization_os;
use std::convert::Infallible;

taken!(
  terminal -> CzTerminal,
  parent_terminal -> CzTerminal,
  child_terminal -> CzTerminal,
  terminal_size_width -> u16,
  terminal_size_height -> u16,
  keychain_query_result -> containerization_os::keychain::KeychainQueryResult,
  file_info -> CzFileInfo,
  registry_infos -> Vec<containerization_os::keychain::RegistryInfo>,
);

handles!(CzTerminal, CzFileInfo,);

failing!(
  cz_keychain_helper_lookup(&str, Option<String>, &str),
  cz_keychain_helper_list(&str, Option<String>),
  cz_keychain_helper_delete(&str, Option<String>, &str),
  cz_keychain_helper_save(&str, Option<String>, &str, &str, &str),
  cz_terminal_new(i32, bool),
  cz_terminal_current(),
  cz_terminal_create(bool, u16, u16),
  cz_keychain_query_save(&str, Option<String>, &str, &str, &str),
  cz_keychain_query_delete(&str, Option<String>, &str),
  cz_keychain_query_get(&str, Option<String>, &str),
  cz_keychain_query_list(&str, Option<String>),
  cz_keychain_query_exists(&str, Option<String>, &str),
  cz_sysctl_by_name(&str),
  cz_file_info(&str),
);

impl CzTerminal {
  pub(crate) fn duplicate(&self) -> CzTerminal {
    match self.0 {}
  }

  pub(crate) fn handle(&self) -> i32 {
    match self.0 {}
  }

  pub(crate) fn write(&self, _data: Vec<u8>) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn size(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn resize_from(&self, _pty: CzTerminal) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn resize_size(&self, _width: u16, _height: u16) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn resize(&self, _width: u16, _height: u16) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn setraw(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn enable_echo(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn disable_echo(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn close(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn reset(&self) -> CzOutcome {
    match self.0 {}
  }

  pub(crate) fn try_reset(&self) {
    match self.0 {}
  }
}

impl CzFileInfo {
  pub(crate) fn mode(&self) -> u16 {
    match self.0 {}
  }

  pub(crate) fn uid(&self) -> i64 {
    match self.0 {}
  }

  pub(crate) fn gid(&self) -> i64 {
    match self.0 {}
  }

  pub(crate) fn dev(&self) -> i64 {
    match self.0 {}
  }

  pub(crate) fn ino(&self) -> i64 {
    match self.0 {}
  }

  pub(crate) fn size(&self) -> i64 {
    match self.0 {}
  }

  pub(crate) fn path(&self) -> String {
    match self.0 {}
  }

  pub(crate) fn is_directory(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn is_pipe(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn is_socket(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn is_link(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn is_regular_file(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn is_block(&self) -> bool {
    match self.0 {}
  }

  pub(crate) fn is_char(&self) -> bool {
    match self.0 {}
  }
}
