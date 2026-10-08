#![cfg(feature = "integration")]

//! `ArchiveWriter`, `ArchiveReader` and `WriteEntry`, and `EXT4Unpacker`
//! unpacking an archive they wrote.

use containerization_framework as cfw;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::time::Duration;
use std::time::SystemTime;

/// A directory holding two files, one in a subdirectory, and a symlink to the
/// first.
fn tree(root: &Path) {
  std::fs::write(root.join("a.txt"), b"alpha").expect("a file");
  std::fs::create_dir(root.join("sub")).expect("a subdirectory");
  std::fs::write(root.join("sub").join("b.txt"), b"bravo").expect("a file in the subdirectory");
  std::os::unix::fs::symlink("a.txt", root.join("link")).expect("a symlink");
}

/// Each regular file's path and data, read with the reader's iterator.
fn files(reader: &cfw::containerization_archive::ArchiveReader) -> BTreeMap<String, Vec<u8>> {
  reader
    .make_iterator()
    .filter(|(entry, _)| entry.file_type() == cfw::containerization_archive::URLFileResourceType::Regular)
    .map(|(entry, data)| (entry.path().expect("a path"), data))
    .collect()
}

fn entry(
  path: &str,
  file_type: cfw::containerization_archive::URLFileResourceType,
) -> cfw::containerization_archive::WriteEntry {
  let mut entry = cfw::containerization_archive::WriteEntry::new().expect("an entry");
  entry.set_path(Some(path));
  entry.set_file_type(file_type);
  entry.set_permissions(0o644);
  entry
}

#[test]
fn archives_a_directory_and_reads_it_back() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let root = directory.path().join("root");
  let archive = directory.path().join("root.tar.gz");
  std::fs::create_dir(&root).expect("the archived directory");
  tree(&root);

  let writer = cfw::containerization_archive::ArchiveWriter::with_file(
    cfw::containerization_archive::Format::Pax,
    cfw::containerization_archive::Filter::Gzip,
    Default::default(),
    &archive,
  )
  .expect("a writer");
  writer
    .archive_directory(&root)
    .expect("the directory should archive");
  writer.finish_encoding().expect("the archive should finish");

  let reader = cfw::containerization_archive::ArchiveReader::new(&archive).expect("the archive should open");
  let mut entries = BTreeMap::new();
  for (entry, data) in &reader {
    entries.insert(
      entry.path().expect("a path"),
      (entry.file_type(), entry.symlink_target(), data),
    );
  }
  reader
    .throw_if_stream_failed()
    .expect("the archive should read whole");

  assert_eq!(
    entries.keys().map(String::as_str).collect::<Vec<_>>(),
    ["./", "a.txt", "link", "sub/", "sub/b.txt"]
  );
  assert_eq!(
    entries["a.txt"],
    (
      cfw::containerization_archive::URLFileResourceType::Regular,
      None,
      b"alpha".to_vec()
    )
  );
  assert_eq!(entries["sub/b.txt"].2, b"bravo");
  assert_eq!(
    entries["link"].0,
    cfw::containerization_archive::URLFileResourceType::SymbolicLink
  );
  assert_eq!(entries["link"].1.as_deref(), Some("a.txt"));
  assert_eq!(
    entries["sub/"].0,
    cfw::containerization_archive::URLFileResourceType::Directory
  );
}

#[test]
fn archives_paths_under_a_base() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let root = directory.path().join("root");
  let archive = directory.path().join("some.tar");
  std::fs::create_dir(&root).expect("the base directory");
  tree(&root);

  let writer = cfw::containerization_archive::ArchiveWriter::with_file(
    cfw::containerization_archive::Format::Pax,
    cfw::containerization_archive::Filter::None,
    Default::default(),
    &archive,
  )
  .expect("a writer");
  writer
    .archive(&[&root.join("sub")], &root)
    .expect("the paths should archive");
  writer.finish_encoding().expect("the archive should finish");

  let reader = cfw::containerization_archive::ArchiveReader::new(&archive).expect("the archive should open");

  assert_eq!(
    files(&reader),
    BTreeMap::from([("sub/b.txt".to_string(), b"bravo".to_vec())])
  );
}

#[test]
fn writes_entries_whole_and_in_chunks() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let archive = directory.path().join("entries.tar");
  let modified = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);

  let configuration = cfw::containerization_archive::ArchiveWriterConfiguration::new(
    cfw::containerization_archive::Format::Ustar,
    cfw::containerization_archive::Filter::None,
  );
  let writer = cfw::containerization_archive::ArchiveWriter::new(&configuration).expect("a writer");
  writer
    .open(&archive)
    .expect("the archive's file should open");

  let mut whole = entry("whole.txt", cfw::containerization_archive::URLFileResourceType::Regular);
  whole.set_size(Some(5));
  whole.set_modification_date(Some(modified));
  writer
    .write_entry(&whole, Some(b"whole"))
    .expect("an entry with data");
  writer
    .write_entry(
      &entry("empty", cfw::containerization_archive::URLFileResourceType::Directory),
      None,
    )
    .expect("an entry without data");

  let mut chunked = cfw::containerization_archive::WriteEntry::with_archive(&writer);
  chunked.set_path(Some("chunked.txt"));
  chunked.set_file_type(cfw::containerization_archive::URLFileResourceType::Regular);
  chunked.set_permissions(0o600);
  chunked.set_size(Some(10));
  let transaction = writer.make_transaction_writer();
  transaction.write_header(&chunked).expect("a header");
  transaction.write_chunk(b"chunk").expect("a first chunk");
  transaction.write_chunk(b"ed!!!").expect("a second chunk");
  transaction.finish().expect("the entry should finish");
  writer.finish_encoding().expect("the archive should finish");

  let reader = cfw::containerization_archive::ArchiveReader::with_format(
    cfw::containerization_archive::Format::Ustar,
    cfw::containerization_archive::Filter::None,
    &archive,
  )
  .expect("the archive should open");
  let (read, data) = reader.extract_file("whole.txt").expect("the whole entry");

  assert_eq!(data, b"whole");
  assert_eq!(read.permissions() & 0o777, 0o644);
  assert_eq!(read.size(), Some(5));
  assert_eq!(read.modification_date(), Some(modified));

  let (read, data) = reader
    .extract_file("chunked.txt")
    .expect("the chunked entry");

  assert_eq!(data, b"chunked!!!");
  assert_eq!(read.permissions() & 0o777, 0o600);
}

#[test]
fn reads_and_writes_an_entrys_properties() {
  let mut entry = cfw::containerization_archive::WriteEntry::new().expect("an entry");

  assert_eq!(entry.path(), None);
  assert_eq!(entry.size(), None);
  assert_eq!(
    entry.file_type(),
    cfw::containerization_archive::URLFileResourceType::Unknown
  );
  assert_eq!(entry.modification_date(), None);

  let before_1970 = SystemTime::UNIX_EPOCH - Duration::from_secs(86_400);
  let accessed = SystemTime::UNIX_EPOCH + Duration::from_millis(1_600_000_000_250);
  let xattrs = BTreeMap::from([
    ("user.one".to_string(), b"1".to_vec()),
    ("user.two".to_string(), vec![0, 2, 255]),
  ]);
  entry.set_path(Some("dir/file"));
  entry.set_path_utf8(Some("dir/fïle"));
  entry.set_size(Some(42));
  entry.set_permissions(0o755);
  entry.set_owner(Some(501));
  entry.set_group(Some(20));
  entry.set_file_type(cfw::containerization_archive::URLFileResourceType::Regular);
  entry.set_hardlink(Some("other"));
  entry.set_symlink_target(Some("target"));
  entry.set_creation_date(Some(before_1970));
  entry.set_content_access_date(Some(accessed));
  entry.set_xattrs(&xattrs);

  assert_eq!(entry.path_utf8().as_deref(), Some("dir/fïle"));
  assert_eq!(entry.size(), Some(42));
  assert_eq!(entry.permissions() & 0o777, 0o755);
  assert_eq!(entry.owner(), Some(501));
  assert_eq!(entry.group(), Some(20));
  assert_eq!(
    entry.file_type(),
    cfw::containerization_archive::URLFileResourceType::Regular
  );
  assert_eq!(entry.hardlink().as_deref(), Some("other"));
  assert_eq!(entry.hardlink_utf8().as_deref(), Some("other"));
  assert_eq!(entry.symlink_target().as_deref(), Some("target"));
  // Swift's setter drops a fraction of a second, and moves a date before 1970
  // one second later.
  assert_eq!(entry.creation_date(), Some(before_1970 + Duration::from_secs(1)));
  assert_eq!(
    entry.content_access_date(),
    Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1_600_000_000))
  );
  assert_eq!(entry.xattrs(), xattrs);
  assert!(
    entry
      .strmode()
      .expect("a mode string")
      .starts_with("-rwxr-xr-x"),
    "{:?}",
    entry.strmode()
  );

  entry.set_size(None);
  entry.set_path(None);
  entry.set_modification_date(None);

  assert_eq!(entry.size(), None);
  assert_eq!(entry.path(), None);
  assert_eq!(entry.modification_date(), None);
}

#[test]
fn streams_each_entrys_data() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let root = directory.path().join("root");
  let archive = directory.path().join("root.tar");
  std::fs::create_dir(&root).expect("the archived directory");
  tree(&root);
  let writer = cfw::containerization_archive::ArchiveWriter::with_file(
    cfw::containerization_archive::Format::Pax,
    cfw::containerization_archive::Filter::None,
    Default::default(),
    &archive,
  )
  .expect("a writer");
  writer
    .archive_directory(&root)
    .expect("the directory should archive");
  writer.finish_encoding().expect("the archive should finish");

  let reader = cfw::containerization_archive::ArchiveReader::new(&archive).expect("the archive should open");
  let mut streamed = BTreeMap::new();
  for (entry, mut data) in reader.make_streaming_iterator() {
    let mut bytes = Vec::new();
    data
      .read_to_end(&mut bytes)
      .expect("the entry's data should read");
    streamed.insert(entry.path().expect("a path"), bytes);
  }

  assert_eq!(streamed["a.txt"], b"alpha");
  assert_eq!(streamed["sub/b.txt"], b"bravo");
  assert_eq!(streamed["link"], b"");
}

#[test]
fn extracts_an_archive_and_rejects_paths_that_escape_it() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let archive = directory.path().join("escape.tar");
  let into = directory.path().join("into");
  let writer = cfw::containerization_archive::ArchiveWriter::with_file(
    cfw::containerization_archive::Format::Pax,
    cfw::containerization_archive::Filter::None,
    Default::default(),
    &archive,
  )
  .expect("a writer");
  for (path, data) in [("kept.txt", b"kept"), ("../escaped.txt", b"gone")] {
    let mut file = entry(path, cfw::containerization_archive::URLFileResourceType::Regular);
    file.set_size(Some(4));
    writer.write_entry(&file, Some(data)).expect("an entry");
  }
  writer.finish_encoding().expect("the archive should finish");

  let rejected = cfw::containerization_archive::ArchiveReader::new(&archive)
    .expect("the archive should open")
    .extract_contents(&into)
    .expect("the archive should extract");

  assert_eq!(rejected, ["../escaped.txt"]);
  assert_eq!(std::fs::read(into.join("kept.txt")).expect("the kept file"), b"kept");
  assert!(!directory.path().join("escaped.txt").exists());
}

#[test]
fn reads_from_a_file_handle_it_owns() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let root = directory.path().join("root");
  let archive = directory.path().join("root.tar");
  std::fs::create_dir(&root).expect("the archived directory");
  tree(&root);
  let file = std::fs::File::create(&archive).expect("the archive's file");
  let writer =
    cfw::containerization_archive::ArchiveWriter::new(&cfw::containerization_archive::ArchiveWriterConfiguration::new(
      cfw::containerization_archive::Format::Pax,
      cfw::containerization_archive::Filter::None,
    ))
    .expect("a writer");
  writer
    .open_with_file_descriptor(std::os::fd::AsRawFd::as_raw_fd(&file))
    .expect("the descriptor should open");
  writer
    .archive_directory(&root)
    .expect("the directory should archive");
  writer.finish_encoding().expect("the archive should finish");
  drop(file);

  let reader = cfw::containerization_archive::ArchiveReader::with_file_handle(
    cfw::containerization_archive::Format::Pax,
    cfw::containerization_archive::Filter::None,
    std::fs::File::open(&archive).expect("the archive").into(),
  )
  .expect("the archive should open");

  assert_eq!(files(&reader)["a.txt"], b"alpha");
}

#[test]
fn reads_a_bundled_zip() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let root = directory.path().join("root");
  let archive = directory.path().join("root.zip");
  std::fs::create_dir(&root).expect("the archived directory");
  tree(&root);
  let writer = cfw::containerization_archive::ArchiveWriter::with_file(
    cfw::containerization_archive::Format::Zip,
    cfw::containerization_archive::Filter::None,
    cfw::containerization_archive::archive_writer::ArchiveWriterOptions {
      options: vec![
        cfw::containerization_archive::Options::Compression(
          cfw::containerization_archive::options::Compression::Deflate,
        ),
        cfw::containerization_archive::Options::CompressionLevel(9),
      ],
      ..Default::default()
    },
    &archive,
  )
  .expect("a writer");
  writer
    .archive_directory(&root)
    .expect("the directory should archive");
  writer.finish_encoding().expect("the archive should finish");

  let bundle = std::fs::read(&archive).expect("the zip's bytes");
  let reader = cfw::containerization_archive::ArchiveReader::with_bundle("root.zip", &bundle, Some("archive-test"))
    .expect("the bundle should open");

  assert_eq!(files(&reader)["sub/b.txt"], b"bravo");
}

#[test]
fn writes_with_an_extended_attribute_format() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let archive = directory.path().join("xattrs.tar");
  let configuration = cfw::containerization_archive::ArchiveWriterConfiguration {
    options: vec![cfw::containerization_archive::Options::Xattrformat(
      cfw::containerization_archive::options::XattrFormat::Schily,
    )],
    ..cfw::containerization_archive::ArchiveWriterConfiguration::new(
      cfw::containerization_archive::Format::Pax,
      cfw::containerization_archive::Filter::None,
    )
  };
  let writer = cfw::containerization_archive::ArchiveWriter::new(&configuration).expect("a writer");
  writer
    .open(&archive)
    .expect("the archive's file should open");
  let mut file = entry("xattr.txt", cfw::containerization_archive::URLFileResourceType::Regular);
  let xattrs = BTreeMap::from([("user.note".to_string(), b"hi".to_vec())]);
  file.set_size(Some(1));
  file.set_xattrs(&xattrs);
  writer.write_entry(&file, Some(b"x")).expect("an entry");
  writer.finish_encoding().expect("the archive should finish");

  let (read, _) = cfw::containerization_archive::ArchiveReader::new(&archive)
    .expect("the archive should open")
    .extract_file("xattr.txt")
    .expect("the entry");

  assert_eq!(read.xattrs(), xattrs);
}

#[test]
fn describes_an_extended_attribute_format() {
  assert_eq!(
    cfw::containerization_archive::options::XattrFormat::Schily
      .description()
      .expect("a description"),
    "SCHILY"
  );
}

#[test]
fn says_which_archive_it_could_not_open() {
  let error = cfw::containerization_archive::ArchiveReader::new("/nowhere/archive.tar".as_ref())
    .err()
    .expect("an archive that doesn't exist");

  assert!(error.to_string().contains("/nowhere/archive.tar"), "{error}");
}

#[test]
fn refuses_a_path_that_isnt_utf8_without_asking_swift() {
  use std::os::unix::ffi::OsStrExt;

  let directory = tempfile::tempdir().expect("a temporary directory");
  let archive = directory
    .path()
    .join(std::ffi::OsStr::from_bytes(b"\xff.tar"));

  let error = cfw::containerization_archive::ArchiveWriter::with_file(
    cfw::containerization_archive::Format::Pax,
    cfw::containerization_archive::Filter::None,
    Default::default(),
    &archive,
  )
  .err()
  .expect("a path that isn't UTF-8");

  assert!(matches!(error, cfw::Error::Failed { .. }), "{error:?}");
  assert!(error.to_string().contains("UTF-8"), "{error}");
  assert_eq!(
    std::fs::read_dir(directory.path())
      .expect("the temporary directory")
      .count(),
    0,
    "Swift was asked to write the archive"
  );
}

#[test]
fn says_which_file_is_not_in_an_archive() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let archive = directory.path().join("one.tar");
  let writer = cfw::containerization_archive::ArchiveWriter::with_file(
    cfw::containerization_archive::Format::Pax,
    cfw::containerization_archive::Filter::None,
    Default::default(),
    &archive,
  )
  .expect("a writer");
  let mut file = entry("one.txt", cfw::containerization_archive::URLFileResourceType::Regular);
  file.set_size(Some(3));
  writer.write_entry(&file, Some(b"one")).expect("an entry");
  writer.finish_encoding().expect("the archive should finish");

  let error = cfw::containerization_archive::ArchiveReader::new(&archive)
    .expect("the archive should open")
    .extract_file("two.txt")
    .err()
    .expect("a file the archive lacks");

  assert!(error.to_string().contains("two.txt"), "{error}");
}

#[test]
fn unpacks_an_archive_into_a_filesystem() {
  let directory = tempfile::tempdir().expect("a temporary directory");
  let root = directory.path().join("root");
  let archive = directory.path().join("root.tar.gz");
  let at = directory.path().join("root.ext4");
  let exported = directory.path().join("exported.tar");
  std::fs::create_dir(&root).expect("the archived directory");
  tree(&root);
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

  cfw::containerization::image::EXT4Unpacker::new(64 * 1024 * 1024, None)
    .unpack_archive(&archive, cfw::containerization_archive::Filter::Gzip, &at)
    .expect("the archive should unpack");
  cfw::containerization_ext4::ext4::EXT4Reader::new(&at)
    .expect("the filesystem should read")
    .export(&exported)
    .expect("the filesystem should export");

  let files = files(&cfw::containerization_archive::ArchiveReader::new(&exported).expect("the export should open"));
  let files: BTreeMap<_, _> = files
    .into_iter()
    .map(|(path, data)| (path.trim_start_matches("./").to_string(), data))
    .collect();

  assert_eq!(files["a.txt"], b"alpha");
  assert_eq!(files["sub/b.txt"], b"bravo");
}
