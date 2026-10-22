//! Stand-ins for `OS.swift`: ContainerizationOS's terminals, capabilities,
//! keychain queries, sysctl, network protocol counters and file information.

use super::CzOutcome;
use crate::containerization_os;
use std::convert::Infallible;

taken!(
  terminal -> CzTerminal,
  parent_terminal -> CzTerminal,
  child_terminal -> CzTerminal,
  terminal_size_width -> u16,
  terminal_size_height -> u16,
  keychain_query_result -> Result<containerization_os::keychain::KeychainQueryResult, crate::error::Error>,
  file_info -> CzFileInfo,
  stat -> containerization_os::Stat,
  proc_net_snmp -> containerization_os::linux::ProcNetSnmp,
);

handles!(CzTerminal, CzFileInfo,);

failing!(
  cz_capability_name_parse(&str),
  cz_capability_set_parse(&str),
  cz_terminal_new(i32, bool),
  cz_terminal_current(),
  cz_terminal_create(bool, u16, u16),
  cz_keychain_query_save(&str, Option<String>, &str, &str, &str),
  cz_keychain_query_delete(&str, Option<String>, &str),
  cz_keychain_query_get(&str, Option<String>, &str),
  cz_keychain_query_list(&str, Option<String>),
  cz_keychain_query_exists(&str, Option<String>, &str),
  cz_sysctl_by_name(&str),
  cz_proc_net_snmp_new(&str, Option<String>, Option<String>),
  cz_proc_net_snmp_read(Option<i32>),
  cz_file_info(&str),
);

uninhabited!(CzTerminal {
  fn duplicate() -> CzTerminal;
  fn handle() -> i32;
  fn write(Vec<u8>) -> CzOutcome;
  fn size() -> CzOutcome;
  fn resize_from(CzTerminal) -> CzOutcome;
  fn resize_size(u16, u16) -> CzOutcome;
  fn resize(u16, u16) -> CzOutcome;
  fn setraw() -> CzOutcome;
  fn enable_echo() -> CzOutcome;
  fn disable_echo() -> CzOutcome;
  fn close() -> CzOutcome;
  fn reset() -> CzOutcome;
  fn try_reset();
});

uninhabited!(CzFileInfo {
  fn mode() -> u16;
  fn uid() -> i64;
  fn gid() -> i64;
  fn dev() -> i64;
  fn ino() -> i64;
  fn size() -> i64;
  fn path() -> String;
  fn is_directory() -> bool;
  fn is_pipe() -> bool;
  fn is_socket() -> bool;
  fn is_link() -> bool;
  fn is_regular_file() -> bool;
  fn is_block() -> bool;
  fn is_char() -> bool;
});
