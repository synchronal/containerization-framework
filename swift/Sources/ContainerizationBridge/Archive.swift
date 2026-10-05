//===----------------------------------------------------------------------===//
// ContainerizationArchive: `ArchiveWriter`, `ArchiveReader` and `WriteEntry`,
// and `EXT4Unpacker.unpack(archive:compression:at:)`, which takes its
// `Filter`.
//
// Enums cross as their `rawValue`s.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationArchive
import Foundation
import SystemPackage

func unpackExt4Archive(unpacker: RustExt4Unpacker, archive: RustStr, compression: RustStr, at: RustStr) -> CzOutcome {
  // Not `Sendable`, but a value only the task below uses.
  nonisolated(unsafe) let unpacker = EXT4Unpacker(unpacker)
  let archive = URL(filePath: archive.toString())
  let compression = compression.toString()
  let at = URL(filePath: at.toString())

  return CzOutcome {
    let filter = try archiveFilter(compression)
    return try blocking { try await unpacker.unpack(archive: archive, compression: filter, at: at) }
  }
}

func xattrFormatDescription(format: RustStr) -> CzOutcome {
  let format = format.toString()

  return CzOutcome {
    guard let format = Options.XattrFormat(rawValue: format) else {
      throw BridgeError.malformed("extended attribute format", format)
    }
    return format.description
  }
}

/// For a unit test: the raw values of the enum `name`, in the order of Rust's
/// `ALL`.
func archiveRawValues(name: RustStr) -> RustVec<RustString> {
  let name = name.toString()
  let rawValues: [String] =
    switch name {
    case "Format":
      [
        Format.ustar, .gnutar, .pax, .paxRestricted, .cpio, .cpioNewc, .zip, .shar, .sharDump, .iso9660,
        .sevenZip, .arBSD, .arGNU, .mtree, .xar,
      ].map(\.rawValue)
    case "Filter":
      [
        Filter.none, .gzip, .bzip2, .compress, .lzma, .xz, .uu, .rpm, .lzip, .lrzip, .lzop, .grzip, .lz4, .zstd,
      ].map(\.rawValue)
    case "XattrFormat":
      [Options.XattrFormat.schily, .libarchive, .all].map(\.rawValue)
    case "URLFileResourceType":
      [
        URLFileResourceType.namedPipe, .characterSpecial, .directory, .blockSpecial, .regular, .symbolicLink,
        .socket, .unknown,
      ].map(\.rawValue)
    default:
      preconditionFailure("no enum named \(name)")
    }

  return rustStrings(rawValues)
}

func archiveDefaultLocales() -> RustVec<RustString> {
  rustStrings(ArchiveWriterConfiguration.defaultLocales)
}

private func archiveFormat(_ rawValue: String) throws -> Format {
  guard let format = Format(rawValue: rawValue) else {
    throw BridgeError.malformed("archive format", rawValue)
  }
  return format
}

private func archiveFilter(_ rawValue: String) throws -> Filter {
  guard let filter = Filter(rawValue: rawValue) else {
    throw BridgeError.malformed("archive filter", rawValue)
  }
  return filter
}

extension ArchiveWriterConfiguration {
  init(_ configuration: RustArchiveWriterConfiguration) throws {
    let options = try list(configuration.optionsLen()) { index -> Options in
      switch configuration.optionKindAt(index) {
      case .CompressionLevel:
        return .compressionLevel(configuration.optionCompressionLevelAt(index))
      case .CompressionStore:
        return .compression(.store)
      case .CompressionDeflate:
        return .compression(.deflate)
      case .Xattrformat:
        let rawValue = configuration.optionXattrFormatAt(index).toString()
        guard let format = Options.XattrFormat(rawValue: rawValue) else {
          throw BridgeError.malformed("extended attribute format", rawValue)
        }
        return .xattrformat(format)
      }
    }

    self.init(
      format: try archiveFormat(configuration.format().toString()),
      filter: try archiveFilter(configuration.filter().toString()),
      options: options,
      locales: strings(configuration.localesLen()) { configuration.localesAt($0) }
    )
  }
}

func newWriteEntry() -> CzOutcome {
  CzOutcome { CzWriteEntry(WriteEntry()) }
}

/// Unchecked: Rust's `WriteEntry` isn't `Sync`, so one call runs at a time.
final class CzWriteEntry: @unchecked Sendable {
  let entry: WriteEntry

  init(_ entry: WriteEntry) {
    self.entry = entry
  }

  /// Another handle on the same entry, for a writer to take.
  func duplicate() -> CzWriteEntry {
    CzWriteEntry(entry)
  }

  func hasSize() -> Bool {
    entry.size != nil
  }

  func size() -> Int64 {
    entry.size ?? 0
  }

  func setSize(isSet: Bool, size: Int64) {
    entry.size = isSet ? size : nil
  }

  func permissions() -> UInt16 {
    entry.permissions
  }

  func setPermissions(permissions: UInt16) {
    entry.permissions = permissions
  }

  func hasOwner() -> Bool {
    entry.owner != nil
  }

  func owner() -> UInt32 {
    entry.owner ?? 0
  }

  func setOwner(isSet: Bool, owner: UInt32) {
    entry.owner = isSet ? owner : nil
  }

  func hasGroup() -> Bool {
    entry.group != nil
  }

  func group() -> UInt32 {
    entry.group ?? 0
  }

  func setGroup(isSet: Bool, group: UInt32) {
    entry.group = isSet ? group : nil
  }

  func hardlink() -> String? {
    entry.hardlink
  }

  func setHardlink(hardlink: RustString?) {
    entry.hardlink = hardlink?.toString()
  }

  func hardlinkUtf8() -> String? {
    entry.hardlinkUtf8
  }

  func setHardlinkUtf8(hardlink: RustString?) {
    entry.hardlinkUtf8 = hardlink?.toString()
  }

  func strmode() -> String? {
    entry.strmode
  }

  func fileType() -> String {
    entry.fileType.rawValue
  }

  func setFileType(fileType: RustStr) {
    entry.fileType = URLFileResourceType(rawValue: fileType.toString())
  }

  // Dates, as seconds since 1970.

  func hasContentAccessDate() -> Bool {
    entry.contentAccessDate != nil
  }

  func contentAccessDate() -> Double {
    entry.contentAccessDate?.timeIntervalSince1970 ?? 0
  }

  func setContentAccessDate(isSet: Bool, seconds: Double) {
    entry.contentAccessDate = isSet ? Date(timeIntervalSince1970: seconds) : nil
  }

  func hasCreationDate() -> Bool {
    entry.creationDate != nil
  }

  func creationDate() -> Double {
    entry.creationDate?.timeIntervalSince1970 ?? 0
  }

  func setCreationDate(isSet: Bool, seconds: Double) {
    entry.creationDate = isSet ? Date(timeIntervalSince1970: seconds) : nil
  }

  func hasModificationDate() -> Bool {
    entry.modificationDate != nil
  }

  func modificationDate() -> Double {
    entry.modificationDate?.timeIntervalSince1970 ?? 0
  }

  func setModificationDate(isSet: Bool, seconds: Double) {
    entry.modificationDate = isSet ? Date(timeIntervalSince1970: seconds) : nil
  }

  func path() -> String? {
    entry.path
  }

  func setPath(path: RustString?) {
    entry.path = path?.toString()
  }

  func pathUtf8() -> String? {
    entry.pathUtf8
  }

  func setPathUtf8(path: RustString?) {
    entry.pathUtf8 = path?.toString()
  }

  func symlinkTarget() -> String? {
    entry.symlinkTarget
  }

  func setSymlinkTarget(target: RustString?) {
    entry.symlinkTarget = target?.toString()
  }

  func xattrs() -> CzOutcome {
    CzOutcome { entry.xattrs }
  }

  /// `values` is every value joined, each `lengths` long in turn.
  func setXattrs(names: RustVec<RustString>, lengths: RustVec<UInt64>, values: RustVec<UInt8>) {
    let values = Data(values)
    var xattrs: [String: Data] = [:]
    var offset = values.startIndex
    for (name, length) in zip(strings(names), lengths) {
      let end = offset + Int(length)
      xattrs[name] = values.subdata(in: offset..<end)
      offset = end
    }
    entry.xattrs = xattrs
  }
}

func newArchiveWriter(configuration: RustArchiveWriterConfiguration) -> CzOutcome {
  CzOutcome { CzArchiveWriter(try ArchiveWriter(configuration: try ArchiveWriterConfiguration(configuration))) }
}

func archiveWriterWithFile(configuration: RustArchiveWriterConfiguration, file: RustStr) -> CzOutcome {
  let file = URL(filePath: file.toString())

  return CzOutcome {
    let configuration = try ArchiveWriterConfiguration(configuration)
    return CzArchiveWriter(
      try ArchiveWriter(
        format: configuration.format,
        filter: configuration.filter,
        options: configuration.options,
        locales: configuration.locales,
        file: file
      )
    )
  }
}

/// Unchecked: Rust's `ArchiveWriter` isn't `Sync`, so one call runs at a time.
final class CzArchiveWriter: @unchecked Sendable {
  let writer: ArchiveWriter

  init(_ writer: ArchiveWriter) {
    self.writer = writer
  }

  func newEntry() -> CzWriteEntry {
    CzWriteEntry(WriteEntry(writer))
  }

  func open(file: RustStr) -> CzOutcome {
    let file = URL(filePath: file.toString())

    return CzOutcome { try writer.open(file: file) }
  }

  func openWithFileDescriptor(fileDescriptor: Int32) -> CzOutcome {
    CzOutcome { try writer.open(fileDescriptor: fileDescriptor) }
  }

  func finishEncoding() -> CzOutcome {
    CzOutcome { try writer.finishEncoding() }
  }

  func makeTransactionWriter() -> CzArchiveWriterTransaction {
    CzArchiveWriterTransaction(writer.makeTransactionWriter())
  }

  func writeEntry(entry: CzWriteEntry, hasData: Bool, data: RustVec<UInt8>) -> CzOutcome {
    let data = hasData ? Data(data) : nil

    return CzOutcome {
      if let data {
        try data.withUnsafeBytes { try writer.writeEntry(entry: entry.entry, data: $0) }
      } else {
        try writer.writeEntry(entry: entry.entry, data: nil)
      }
      return ()
    }
  }

  func archiveDirectory(dir: RustStr) -> CzOutcome {
    let dir = URL(filePath: dir.toString())

    return CzOutcome { try writer.archiveDirectory(dir) }
  }

  func archive(paths: RustVec<RustString>, base: RustStr) -> CzOutcome {
    let paths = strings(paths).map { FilePath($0) }
    let base = FilePath(base.toString())

    return CzOutcome { try writer.archive(paths, base: base) }
  }
}

/// Unchecked: Rust's `ArchiveWriterTransaction` isn't `Sync`, so one call
/// runs at a time.
final class CzArchiveWriterTransaction: @unchecked Sendable {
  let transaction: ArchiveWriterTransaction

  init(_ transaction: ArchiveWriterTransaction) {
    self.transaction = transaction
  }

  func writeHeader(entry: CzWriteEntry) -> CzOutcome {
    CzOutcome { try transaction.writeHeader(entry: entry.entry) }
  }

  func writeChunk(data: RustVec<UInt8>) -> CzOutcome {
    let data = Data(data)

    return CzOutcome { try data.withUnsafeBytes { try transaction.writeChunk(data: $0) } }
  }

  func finish() -> CzOutcome {
    CzOutcome { try transaction.finish() }
  }
}

func openArchiveReader(file: RustStr) -> CzOutcome {
  let file = URL(filePath: file.toString())

  return CzOutcome { CzArchiveReader(try ArchiveReader(file: file)) }
}

func archiveReaderWithFormat(format: RustStr, filter: RustStr, file: RustStr) -> CzOutcome {
  let format = format.toString()
  let filter = filter.toString()
  let file = URL(filePath: file.toString())

  return CzOutcome {
    CzArchiveReader(try ArchiveReader(format: try archiveFormat(format), filter: try archiveFilter(filter), file: file))
  }
}

/// The reader owns the descriptor, and closes it.
func archiveReaderWithFileHandle(format: RustStr, filter: RustStr, fileHandle: Int32) -> CzOutcome {
  let format = format.toString()
  let filter = filter.toString()
  let fileHandle = FileHandle(fileDescriptor: fileHandle, closeOnDealloc: true)

  return CzOutcome {
    CzArchiveReader(
      try ArchiveReader(
        format: try archiveFormat(format),
        filter: try archiveFilter(filter),
        fileHandle: fileHandle
      )
    )
  }
}

func archiveReaderWithBundle(name: RustStr, bundle: RustVec<UInt8>, tempDirectoryBaseName: RustString?) -> CzOutcome {
  let name = name.toString()
  let bundle = Data(bundle)
  let tempDirectoryBaseName = tempDirectoryBaseName?.toString()

  return CzOutcome {
    CzArchiveReader(try ArchiveReader(name: name, bundle: bundle, tempDirectoryBaseName: tempDirectoryBaseName))
  }
}

/// Unchecked: Rust's `ArchiveReader` isn't `Sync`, so one call runs at a time.
final class CzArchiveReader: @unchecked Sendable {
  let reader: ArchiveReader

  init(_ reader: ArchiveReader) {
    self.reader = reader
  }

  func makeIterator() -> CzArchiveIterator {
    CzArchiveIterator(reader.makeIterator())
  }

  func makeStreamingIterator() -> CzStreamingIterator {
    CzStreamingIterator(reader.makeStreamingIterator())
  }

  func throwIfStreamFailed() -> CzOutcome {
    CzOutcome { try reader.throwIfStreamFailed() }
  }

  func extractContents(to: RustStr) -> CzOutcome {
    let to = URL(filePath: to.toString())

    return CzOutcome { try reader.extractContents(to: to) }
  }

  func extractFile(path: RustStr) -> CzOutcome {
    let path = path.toString()

    return CzOutcome { try reader.extractFile(path: path) }
  }
}

/// Unchecked: Rust's iterator borrows its reader, and isn't `Sync`.
final class CzArchiveIterator: @unchecked Sendable {
  var iterator: ArchiveReader.Iterator

  init(_ iterator: ArchiveReader.Iterator) {
    self.iterator = iterator
  }

  func next() -> CzOutcome {
    CzOutcome.holding(iterator.next())
  }
}

/// Unchecked: Rust's iterator borrows its reader, and isn't `Sync`.
final class CzStreamingIterator: @unchecked Sendable {
  var iterator: ArchiveReader.StreamingIterator

  init(_ iterator: ArchiveReader.StreamingIterator) {
    self.iterator = iterator
  }

  func next() -> CzOutcome {
    CzOutcome.holding(iterator.next())
  }
}

/// Unchecked: Rust's reader borrows the archive's reader, and isn't `Sync`.
final class CzArchiveEntryReader: @unchecked Sendable {
  let reader: ArchiveEntryReader

  init(_ reader: ArchiveEntryReader) {
    self.reader = reader
  }

  /// The bytes read, or a failure for Swift's `-1`.
  func read(maxLength: UInt) -> CzOutcome {
    CzOutcome {
      var buffer = [UInt8](repeating: 0, count: Int(maxLength))
      let count = buffer.withUnsafeMutableBufferPointer { buffer in
        guard let baseAddress = buffer.baseAddress else { return 0 }
        return reader.read(baseAddress, maxLength: buffer.count)
      }
      guard count >= 0 else {
        throw BridgeError.readFailed
      }
      return Array(buffer[..<count])
    }
  }
}
