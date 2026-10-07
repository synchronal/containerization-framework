#![cfg(feature = "integration")]

//! ContainerizationOS: `Terminal`, `CapabilityName`, `CapabilitySet`,
//! `KeychainQuery`, `Sysctl` and `File`.

use cfw::containerization_os::terminal::Size;
use containerization_framework as cfw;
use std::io::IsTerminal;
use std::io::Read;
use std::mem::ManuallyDrop;
use std::os::fd::FromRawFd;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;

const SECURITY_DOMAIN: &str = "dev.reflective.containerization-framework.tests.os";

#[test]
fn parses_capability_names_as_swift_does() {
  let parse = cfw::containerization_os::CapabilityName::parse;

  assert_eq!(
    parse("CAP_CHOWN").expect("a full name"),
    cfw::containerization_os::CapabilityName::Chown
  );
  assert_eq!(
    parse("net_raw").expect("a name without its prefix"),
    cfw::containerization_os::CapabilityName::NetRaw
  );
  assert_eq!(
    parse("cap_sys_admin").expect("a lowercase name"),
    cfw::containerization_os::CapabilityName::SysAdmin
  );

  let error = parse("CAP_NOPE").expect_err("an unknown name");
  assert!(error.to_string().contains("CAP_NOPE"), "{error}");
}

#[test]
fn describes_capability_names() {
  let name = cfw::containerization_os::CapabilityName::CheckpointRestore;

  assert_eq!(name.to_string(), "CAP_CHECKPOINT_RESTORE");
  assert_eq!(name.cap_value(), 40);
  assert_eq!(cfw::containerization_os::CapabilityName::ALL_CASES.len(), 41);
}

#[test]
fn parses_capability_sets_as_swift_does() {
  let parse = cfw::containerization_os::CapabilitySet::parse;

  assert_eq!(
    parse("BOUNDING").expect("an uppercase set"),
    cfw::containerization_os::CapabilitySet::Bounding
  );
  assert_eq!(
    parse("ambient").expect("a lowercase set"),
    cfw::containerization_os::CapabilitySet::Ambient
  );
  assert_eq!(
    cfw::containerization_os::CapabilitySet::Permitted.to_string(),
    "permitted"
  );

  let error = parse("everything").expect_err("an unknown set");
  assert!(error.to_string().contains("everything"), "{error}");
}

#[test]
fn creates_a_pty_of_the_size_asked_for() {
  let (parent, child) =
    cfw::containerization_os::Terminal::create(Some(Size { width: 80, height: 24 })).expect("a pty");

  assert_eq!(child.size().expect("the child's size"), Size { width: 80, height: 24 });

  parent.close().expect("the parent should close");
  child.close().expect("the child should close");
}

#[test]
fn creates_a_pty_of_swifts_default_size() {
  let (parent, child) = cfw::containerization_os::Terminal::create(None).expect("a pty");

  assert_eq!(child.size().expect("the child's size"), Size { width: 120, height: 40 });

  parent.close().expect("the parent should close");
  child.close().expect("the child should close");
}

#[test]
fn resizes_a_pty() {
  let (parent, child) = cfw::containerization_os::Terminal::create(None).expect("a pty");
  let (other_parent, other_child) =
    cfw::containerization_os::Terminal::create(Some(Size { width: 33, height: 11 })).expect("another pty");

  parent
    .resize(Size { width: 100, height: 30 })
    .expect("a resize to a size");
  let resized = child.size().expect("the size after a resize");
  parent
    .resize_width_height(90, 20)
    .expect("a resize to a width and height");
  let resized_to_width_and_height = child.size().expect("the size after a width and height");
  child
    .resize_from(&other_child)
    .expect("a resize from another pty");
  let resized_from = child.size().expect("the size after resizing from another");

  for terminal in [parent, child, other_parent, other_child] {
    terminal.close().expect("the terminal should close");
  }
  assert_eq!(resized, Size { width: 100, height: 30 });
  assert_eq!(resized_to_width_and_height, Size { width: 90, height: 20 });
  assert_eq!(resized_from, Size { width: 33, height: 11 });
}

#[test]
fn writes_to_a_pty() {
  let (parent, child) = cfw::containerization_os::Terminal::create(None).expect("a pty");

  parent.write(b"hello\n").expect("a write to the parent");
  // Borrowed: the terminal closes it.
  let mut reader = ManuallyDrop::new(unsafe { std::fs::File::from_raw_fd(child.handle()) });
  let mut line = [0; 6];
  reader.read_exact(&mut line).expect("the line on the child");

  parent.close().expect("the parent should close");
  child.close().expect("the child should close");
  assert_eq!(&line, b"hello\n");
}

#[test]
fn changes_and_resets_a_ptys_modes() {
  let (parent, child) = cfw::containerization_os::Terminal::create(None).expect("a pty");
  let terminal = cfw::containerization_os::Terminal::new(child.handle(), true).expect("a terminal on the child");

  terminal.setraw().expect("raw mode");
  terminal.disable_echo().expect("echo off");
  terminal.enable_echo().expect("echo on");
  terminal.reset().expect("a reset to the initial state");
  terminal.try_reset();
  assert_eq!(terminal.handle(), child.handle());
  assert_eq!(terminal.clone().handle(), child.handle());

  parent.close().expect("the parent should close");
  child.close().expect("the child should close");
  assert!(terminal.size().is_err(), "the closed descriptor has no size");
}

#[test]
fn needs_a_pty_to_save_its_state() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let file = std::fs::File::create(directory.path().join("plain")).expect("a file");
  let descriptor = std::os::fd::AsRawFd::as_raw_fd(&file);

  assert!(cfw::containerization_os::Terminal::new(descriptor, true).is_err());
  assert!(cfw::containerization_os::Terminal::new(descriptor, false).is_ok());
}

#[test]
fn finds_the_current_terminal_on_stdio() {
  let on_a_terminal =
    std::io::stderr().is_terminal() || std::io::stdout().is_terminal() || std::io::stdin().is_terminal();

  assert_eq!(cfw::containerization_os::Terminal::current().is_ok(), on_a_terminal);
}

#[test]
fn reads_sysctls_by_name() {
  let memory = cfw::containerization_os::sysctl::by_name("hw.memsize").expect("the host's memory");

  assert!(memory > 0, "{memory}");
  assert!(cfw::containerization_os::sysctl::by_name("no.such.sysctl").is_err());
}

#[test]
fn reads_file_info() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let path = directory.path().join("file");
  std::fs::write(&path, b"hello").expect("a file");
  std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).expect("its permissions");
  let metadata = std::fs::metadata(&path).expect("its metadata");

  let info = cfw::containerization_os::file::info(&path).expect("the file's info");

  assert_eq!(info.path(), path.display().to_string());
  assert_eq!(info.size(), 5);
  assert_eq!(info.mode() & 0o777, 0o640);
  assert_eq!(info.uid(), i64::from(metadata.uid()));
  assert_eq!(info.gid(), i64::from(metadata.gid()));
  assert_eq!(info.ino(), metadata.ino() as i64);
  assert_eq!(info.dev(), metadata.dev() as i64);
  assert!(info.is_regular_file());
  assert!(!info.is_directory());
  assert!(!info.is_link());
  assert!(!info.is_pipe());
  assert!(!info.is_socket());
  assert!(!info.is_block());
  assert!(!info.is_char());
}

#[test]
fn reads_file_info_without_following_symlinks() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  std::fs::write(directory.path().join("file"), b"hello").expect("a file");
  std::os::unix::fs::symlink("file", directory.path().join("link")).expect("a symlink");

  let link = cfw::containerization_os::file::info(&directory.path().join("link")).expect("the link's info");
  let dir = cfw::containerization_os::file::info(directory.path()).expect("the directory's info");
  let null = cfw::containerization_os::file::info(std::path::Path::new("/dev/null")).expect("/dev/null's info");

  assert!(link.is_link());
  assert!(!link.is_regular_file());
  assert!(dir.is_directory());
  assert!(null.is_char());
  assert!(cfw::containerization_os::file::info(&directory.path().join("missing")).is_err());
}

/// Writes to the login keychain, under a domain of its own, and removes what
/// it wrote.
#[test]
fn saves_gets_lists_and_deletes_keychain_entries() {
  let query = cfw::containerization_os::keychain::KeychainQuery::new();
  let hostname = "os.example.test";

  query
    .save(SECURITY_DOMAIN, None, hostname, "user", "password")
    .expect("the entry should save");
  let exists = query.exists(SECURITY_DOMAIN, None, hostname);
  let got = query.get(SECURITY_DOMAIN, None, hostname);
  let listed = query.list(SECURITY_DOMAIN, None);
  query
    .delete(SECURITY_DOMAIN, None, hostname)
    .expect("the entry should delete");

  assert!(exists.expect("whether the entry exists"));
  let got = got.expect("the entry").expect("a saved entry");
  assert_eq!(got.username, "user");
  assert_eq!(got.password, "password");
  assert!(got.created_date <= got.modified_date, "{got:?}");
  assert!(!format!("{got:?}").contains("password\""), "{got:?}");
  let listed = listed.expect("the saved entries");
  assert_eq!(
    listed
      .iter()
      .map(|info| info.hostname.as_str())
      .collect::<Vec<_>>(),
    [hostname]
  );
  assert!(
    !query
      .exists(SECURITY_DOMAIN, None, hostname)
      .expect("whether the deleted entry exists")
  );
  assert!(
    query
      .get(SECURITY_DOMAIN, None, hostname)
      .expect("the deleted entry")
      .is_none()
  );
  query
    .delete(SECURITY_DOMAIN, None, hostname)
    .expect("deleting a missing entry succeeds");
}
