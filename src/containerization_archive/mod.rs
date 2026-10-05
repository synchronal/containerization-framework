//! Containerization's `ContainerizationArchive` module: archives read and
//! written through libarchive.
//!
//! Each type wraps the Swift type of the same name, and its methods are the
//! Swift methods written in snake case. `Options`' nested types are in
//! [`options`], and `ArchiveReader`'s iterators are in [`archive_reader`].

mod archive_entry_reader;
pub mod archive_reader;
pub mod archive_writer;
mod archive_writer_configuration;
pub mod options;
mod write_entry;

pub use self::archive_entry_reader::ArchiveEntryReader;
pub use self::archive_reader::ArchiveReader;
pub use self::archive_writer::ArchiveWriter;
pub use self::archive_writer::ArchiveWriterTransaction;
pub use self::archive_writer_configuration::ArchiveWriterConfiguration;
pub use self::archive_writer_configuration::Filter;
pub use self::archive_writer_configuration::Format;
pub use self::archive_writer_configuration::Options;
pub use self::write_entry::URLFileResourceType;
pub use self::write_entry::WriteEntry;
