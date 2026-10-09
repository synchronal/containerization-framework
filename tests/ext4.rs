#![cfg(feature = "integration")]

//! `EXT4Unpacker` and `EXT4.Formatter`, and `EXT4.EXT4Reader` reading back
//! what they wrote.

mod support;

use containerization_framework as cfw;
use std::collections::BTreeMap;
use std::path::Path;

const CAPACITY_IN_BYTES: u64 = 512 * 1024 * 1024;

#[test]
fn unpacks_an_image_and_reports_its_progress() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let at = directory.path().join("rootfs.ext4");
  let image = support::store::image(&support::store::image_store());
  let platform = cfw::containerization_oci::image::Platform::current().expect("the current platform");
  let (events, handler) = support::recorded();

  let mount = cfw::containerization::image::EXT4Unpacker::new(CAPACITY_IN_BYTES, None)
    .unpack(&image, &platform, &at, Some(handler))
    .expect("the image should unpack");

  assert_eq!(mount.r#type, "ext4");
  assert!(mount.source.ends_with("/rootfs.ext4"), "{}", mount.source);
  assert!(
    std::fs::metadata(&at).expect("the filesystem's file").len() >= CAPACITY_IN_BYTES,
    "the filesystem holds at least its capacity"
  );

  let events = events.lock().expect("the recording");
  let total: isize = events
    .iter()
    .map(|event| match event {
      cfw::containerization_extras::ProgressEvent::AddTotalItems(items) => *items,
      _ => 0,
    })
    .sum();
  let unpacked: isize = events
    .iter()
    .map(|event| match event {
      cfw::containerization_extras::ProgressEvent::AddItems(items) => *items,
      _ => 0,
    })
    .sum();

  assert!(total > 0, "{events:?}");
  assert_eq!(unpacked, total, "every item counted is unpacked");
}

#[test]
fn unpacks_with_a_journal() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let at = directory.path().join("journaled.ext4");
  let image = support::store::image(&support::store::image_store());
  let platform = cfw::containerization_oci::image::Platform::current().expect("the current platform");
  let journal = cfw::containerization_ext4::ext4::JournalConfig {
    size: None,
    default_mode: Some(cfw::containerization_ext4::ext4::journal_config::JournalMode::Ordered),
  };

  cfw::containerization::image::EXT4Unpacker::new(CAPACITY_IN_BYTES, Some(journal))
    .unpack(&image, &platform, &at, None)
    .expect("the image should unpack");

  let super_block = cfw::containerization_ext4::ext4::EXT4Reader::new(&at)
    .expect("the filesystem should read")
    .super_block();

  assert_ne!(super_block.journal_inum, 0, "the filesystem has a journal");
}

#[test]
fn exports_an_unpacked_filesystem_as_a_tar() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let at = directory.path().join("rootfs.ext4");
  let archive = directory.path().join("rootfs.tar");
  support::store::unpack(&at);

  cfw::containerization_ext4::ext4::EXT4Reader::new(&at)
    .expect("the unpacked filesystem should read")
    .export(&archive)
    .expect("the filesystem should export");

  let listing = std::process::Command::new("tar")
    .arg("-tf")
    .arg(&archive)
    .output()
    .expect("tar should run");

  assert!(listing.status.success(), "{}", String::from_utf8_lossy(&listing.stderr));
  assert!(
    String::from_utf8_lossy(&listing.stdout)
      .lines()
      .any(|path| path.trim_start_matches("./") == "etc/os-release"),
    "the archive should hold the image's files"
  );
}

#[test]
fn says_which_block_device_it_could_not_read() {
  let error = cfw::containerization_ext4::ext4::EXT4Reader::new("/nowhere/rootfs.ext4".as_ref())
    .err()
    .expect("a filesystem that doesn't exist");

  assert!(error.to_string().contains("/nowhere/rootfs.ext4"), "{error}");
}

#[test]
fn reads_the_current_platform() {
  let platform = cfw::containerization_oci::image::Platform::current().expect("the current platform");

  assert_eq!(platform.os, "linux");
  assert_eq!(platform.architecture, "arm64");
}

fn mode(flag: cfw::containerization_ext4::ext4::FileModeFlag, perm: u16) -> u16 {
  cfw::containerization_ext4::ext4::Inode::mode(flag, perm)
}

/// A filesystem holding a directory, a file with an extended attribute, a
/// hard link to it, and a symlink to it.
fn format(at: &Path) {
  let formatter = cfw::containerization_ext4::ext4::Formatter::new(at, Default::default()).expect("a formatter");
  formatter
    .create(
      "/etc".as_ref(),
      mode(cfw::containerization_ext4::ext4::FileModeFlag::S_IFDIR, 0o755),
      Default::default(),
    )
    .expect("a directory");
  formatter
    .create(
      "/etc/hello.txt".as_ref(),
      mode(cfw::containerization_ext4::ext4::FileModeFlag::S_IFREG, 0o644),
      cfw::containerization_ext4::ext4::formatter::CreateOptions {
        buf: Some(b"hello, ext4"),
        uid: Some(1000),
        gid: Some(1000),
        xattrs: Some(BTreeMap::from([("user.note".to_string(), b"hi".to_vec())])),
        ..Default::default()
      },
    )
    .expect("a file");
  formatter
    .link("/etc/again.txt".as_ref(), "/etc/hello.txt".as_ref())
    .expect("a hard link");
  formatter
    .create(
      "/hello".as_ref(),
      mode(cfw::containerization_ext4::ext4::FileModeFlag::S_IFLNK, 0o777),
      cfw::containerization_ext4::ext4::formatter::CreateOptions {
        link: Some("etc/hello.txt".into()),
        ..Default::default()
      },
    )
    .expect("a symlink");
  formatter.close().expect("the filesystem should close");
}

/// A gzipped tar of a directory holding two files, one in a subdirectory, and
/// a symlink to the first.
fn archive(directory: &Path) -> std::path::PathBuf {
  let root = directory.join("root");
  let archive = directory.join("root.tar.gz");
  std::fs::create_dir_all(root.join("sub")).expect("the archived directory");
  std::fs::write(root.join("a.txt"), b"alpha").expect("a file");
  std::fs::write(root.join("sub").join("b.txt"), b"bravo").expect("a file in the subdirectory");
  std::os::unix::fs::symlink("a.txt", root.join("link")).expect("a symlink");

  let writer = cfw::containerization_archive::ArchiveWriter::with_file(
    cfw::containerization_archive::Format::PaxRestricted,
    cfw::containerization_archive::Filter::Gzip,
    Default::default(),
    &archive,
  )
  .expect("a writer");
  writer
    .archive_directory(&root)
    .expect("the directory should archive");
  writer.finish_encoding().expect("the archive should finish");

  archive
}

/// What `archive` holds, read back from a filesystem it was unpacked into.
fn assert_unpacked(at: &Path) {
  let reader = cfw::containerization_ext4::ext4::EXT4Reader::new(at).expect("the filesystem should read");

  assert_eq!(
    reader
      .read_file("/a.txt".as_ref(), Default::default())
      .expect("a file"),
    b"alpha"
  );
  assert_eq!(
    reader
      .read_file("/sub/b.txt".as_ref(), Default::default())
      .expect("a file in the subdirectory"),
    b"bravo"
  );
  assert_eq!(
    reader
      .read_file("/link".as_ref(), Default::default())
      .expect("the symlink's target"),
    b"alpha"
  );
}

#[test]
fn formats_a_filesystem_and_reads_it_back() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let at = directory.path().join("formatted.ext4");
  format(&at);

  let reader = cfw::containerization_ext4::ext4::EXT4Reader::new(&at).expect("the filesystem should read");
  let super_block = reader.super_block();

  assert_eq!(super_block.magic, cfw::containerization_ext4::ext4::SUPER_BLOCK_MAGIC);
  assert_eq!(super_block.block_size(), 4096);
  assert_eq!(
    reader
      .list_directory("/etc".as_ref())
      .expect("the directory's entries"),
    ["again.txt", "hello.txt"]
  );
  assert_eq!(
    reader
      .read_file("/etc/hello.txt".as_ref(), Default::default())
      .expect("the file"),
    b"hello, ext4"
  );
  assert_eq!(
    reader
      .read_file(
        "/etc/hello.txt".as_ref(),
        cfw::containerization_ext4::ext4::ext4_reader::ReadFileOptions {
          offset: 7,
          count: Some(3),
          ..Default::default()
        },
      )
      .expect("part of the file"),
    b"ext"
  );
  assert_eq!(
    reader
      .read_file("/hello".as_ref(), Default::default())
      .expect("the symlink's target"),
    b"hello, ext4"
  );
}

#[test]
fn stats_files_links_and_symlinks() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let at = directory.path().join("formatted.ext4");
  format(&at);
  let reader = cfw::containerization_ext4::ext4::EXT4Reader::new(&at).expect("the filesystem should read");

  let (number, inode) = reader
    .stat("/etc/hello.txt".as_ref(), true)
    .expect("the file's inode");
  let (again, _) = reader
    .stat("/etc/again.txt".as_ref(), true)
    .expect("the hard link's inode");
  let (_, link) = reader
    .stat("/hello".as_ref(), false)
    .expect("the symlink's inode");

  assert_eq!(
    inode.mode,
    mode(cfw::containerization_ext4::ext4::FileModeFlag::S_IFREG, 0o644)
  );
  assert_eq!((inode.uid, inode.gid), (1000, 1000));
  assert_eq!(inode.size_low, 11);
  assert_eq!(inode.links_count, 2);
  assert_eq!(again, number, "a hard link shares its target's inode");
  assert_eq!(
    link.mode & mode(cfw::containerization_ext4::ext4::FileModeFlag::TYPE_MASK, 0),
    mode(cfw::containerization_ext4::ext4::FileModeFlag::S_IFLNK, 0)
  );
  assert!(
    reader
      .exists("/hello".as_ref(), false)
      .expect("the path is UTF-8")
  );
  assert!(
    !reader
      .exists("/nowhere".as_ref(), true)
      .expect("the path is UTF-8")
  );
  assert_eq!(
    cfw::containerization_ext4::ext4::EXT4Reader::read_inline_extended_attributes(&inode.inline_xattrs)
      .expect("the file's inline extended attributes")
      .len(),
    2,
    "the file's attribute, and the empty `system.data` Swift adds beside it"
  );
  assert_eq!(
    cfw::containerization_ext4::ext4::FileXattrsState::read(&inode.inline_xattrs, 4, 4)
      .expect("the entries after the inline header")
      .len(),
    2,
    "the same attributes, read past the header as readInlineExtendedAttributes does"
  );
}

#[test]
fn says_which_path_it_could_not_read() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let at = directory.path().join("formatted.ext4");
  format(&at);
  let reader = cfw::containerization_ext4::ext4::EXT4Reader::new(&at).expect("the filesystem should read");

  let error = reader
    .read_file("/nowhere.txt".as_ref(), Default::default())
    .err()
    .expect("a file that doesn't exist");

  assert!(error.to_string().contains("/nowhere.txt"), "{error}");
}

#[test]
fn unlinks_a_file() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let at = directory.path().join("unlinked.ext4");
  let formatter = cfw::containerization_ext4::ext4::Formatter::new(&at, Default::default()).expect("a formatter");
  formatter
    .create(
      "/gone.txt".as_ref(),
      mode(cfw::containerization_ext4::ext4::FileModeFlag::S_IFREG, 0o644),
      cfw::containerization_ext4::ext4::formatter::CreateOptions {
        buf: Some(b"soon gone"),
        ..Default::default()
      },
    )
    .expect("a file");
  formatter
    .unlink("/gone.txt".as_ref(), false)
    .expect("the file should unlink");
  formatter.close().expect("the filesystem should close");

  let reader = cfw::containerization_ext4::ext4::EXT4Reader::new(&at).expect("the filesystem should read");

  assert!(
    !reader
      .exists("/gone.txt".as_ref(), true)
      .expect("the path is UTF-8")
  );
}

#[test]
fn rejects_a_block_size_it_cannot_format() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let options = cfw::containerization_ext4::ext4::formatter::FormatterOptions {
    block_size: 3000,
    ..Default::default()
  };

  let error = cfw::containerization_ext4::ext4::Formatter::new(&directory.path().join("bad.ext4"), options)
    .err()
    .expect("a block size that isn't a power of two");

  assert!(error.to_string().contains("3000"), "{error}");
}

#[test]
fn formats_with_a_journal() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let at = directory.path().join("journaled.ext4");
  let options = cfw::containerization_ext4::ext4::formatter::FormatterOptions {
    min_disk_size: 64 * 1024 * 1024,
    journal: Some(cfw::containerization_ext4::ext4::JournalConfig {
      size: None,
      default_mode: Some(cfw::containerization_ext4::ext4::journal_config::JournalMode::Ordered),
    }),
    ..Default::default()
  };

  cfw::containerization_ext4::ext4::Formatter::new(&at, options)
    .expect("a formatter")
    .close()
    .expect("the filesystem should close");

  let super_block = cfw::containerization_ext4::ext4::EXT4Reader::new(&at)
    .expect("the filesystem should read")
    .super_block();

  assert_ne!(super_block.journal_inum, 0, "the filesystem has a journal");
}

#[test]
fn unpacks_an_archive_and_reports_its_progress() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let archive = archive(directory.path());
  let at = directory.path().join("unpacked.ext4");
  let (events, handler) = support::recorded();
  let (size, items) = cfw::containerization_ext4::ext4::Formatter::scan_archive_headers(
    cfw::containerization_archive::Format::PaxRestricted,
    cfw::containerization_archive::Filter::Gzip,
    &archive,
  )
  .expect("the archive's headers");

  let formatter = cfw::containerization_ext4::ext4::Formatter::new(&at, Default::default()).expect("a formatter");
  formatter
    .unpack(
      &archive,
      cfw::containerization_ext4::ext4::formatter::UnpackOptions {
        progress: Some(handler),
        ..Default::default()
      },
    )
    .expect("the archive should unpack");
  formatter.close().expect("the filesystem should close");

  assert_eq!(size, 10, "the regular files' bytes");
  assert_unpacked(&at);

  let events = events.lock().expect("the recording");
  assert!(
    events.contains(&cfw::containerization_extras::ProgressEvent::AddTotalItems(items)),
    "{events:?}"
  );
  assert!(
    events.contains(&cfw::containerization_extras::ProgressEvent::AddTotalSize(size)),
    "{events:?}"
  );
}

#[test]
fn unpacks_an_archive_from_a_reader() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let archive = archive(directory.path());
  let at = directory.path().join("unpacked.ext4");
  let reader = cfw::containerization_archive::ArchiveReader::new(&archive).expect("the archive should open");

  let formatter = cfw::containerization_ext4::ext4::Formatter::new(&at, Default::default()).expect("a formatter");
  formatter
    .unpack_reader(&reader, None)
    .expect("the archive should unpack");
  formatter.close().expect("the filesystem should close");

  assert_unpacked(&at);
}

#[test]
fn makes_a_root_inode() {
  let root = cfw::containerization_ext4::ext4::Inode::root().expect("a root inode");

  assert_eq!(
    root.mode,
    mode(cfw::containerization_ext4::ext4::FileModeFlag::S_IFDIR, 0o755)
  );
  assert_eq!(root.links_count, 2);
  assert_ne!(root.ctime, 0, "its times are set");
}

#[test]
fn compresses_extended_attribute_names() {
  assert_eq!(
    cfw::containerization_ext4::ext4::ExtendedAttribute::compress_name("user.mime_type").expect("a name"),
    (1, "mime_type".to_string())
  );
  assert_eq!(
    cfw::containerization_ext4::ext4::ExtendedAttribute::compress_name("system.posix_acl_access").expect("a name"),
    (2, String::new()),
    "the longest prefix wins"
  );
  assert_eq!(
    cfw::containerization_ext4::ext4::ExtendedAttribute::decompress_name(6, "selinux").expect("a name"),
    "security.selinux"
  );
  assert_eq!(
    cfw::containerization_ext4::ext4::ExtendedAttribute::decompress_name(5, "plain").expect("a name"),
    "plain"
  );
}

#[test]
fn rejects_extended_attributes_without_a_header() {
  let error = cfw::containerization_ext4::ext4::EXT4Reader::read_block_extended_attributes(&[0; 32])
    .err()
    .expect("a buffer with no header");

  assert!(error.to_string().contains("header"), "{error}");
}
