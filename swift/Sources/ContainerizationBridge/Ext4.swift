//===----------------------------------------------------------------------===//
// ContainerizationEXT4: `EXT4.EXT4Reader`, `EXT4.Formatter` and the values
// they take and return, and `EXT4Unpacker`, from Containerization.
//
// A `SuperBlock` or an `Inode` crosses as its bytes, and an archive's format
// and filter as their `rawValue`s.
//===----------------------------------------------------------------------===//

import Containerization
import ContainerizationArchive
import ContainerizationEXT4
import ContainerizationOCI
import Foundation
import SystemPackage

func openExt4Reader(blockDevice: RustStr) -> CzOutcome {
  let blockDevice = FilePath(blockDevice.toString())

  return CzOutcome { CzExt4Reader(try EXT4.EXT4Reader(blockDevice: blockDevice)) }
}

/// Unchecked: Rust's `Ext4Reader` isn't `Sync`, so one call runs at a time.
final class CzExt4Reader: @unchecked Sendable {
  let reader: EXT4.EXT4Reader

  init(_ reader: EXT4.EXT4Reader) {
    self.reader = reader
  }

  func superBlock() -> RustVec<UInt8> {
    rustBytes(memoryBytes(of: reader.superBlock))
  }

  func exists(path: RustStr, followSymlinks: Bool) -> Bool {
    reader.exists(FilePath(path.toString()), followSymlinks: followSymlinks)
  }

  /// The outcome holds the inode's number and the inode.
  func stat(path: RustStr, followSymlinks: Bool) -> CzOutcome {
    let path = FilePath(path.toString())

    return CzOutcome {
      let (number, inode) = try reader.stat(path, followSymlinks: followSymlinks)
      return (number, inode)
    }
  }

  func listDirectory(path: RustStr) -> CzOutcome {
    let path = FilePath(path.toString())

    return CzOutcome { try reader.listDirectory(path) }
  }

  func readFile(at: RustStr, offset: UInt64, count: UInt?, followSymlinks: Bool) -> CzOutcome {
    let at = FilePath(at.toString())
    let count = count.map { Int($0) }

    return CzOutcome { try reader.readFile(at: at, offset: offset, count: count, followSymlinks: followSymlinks) }
  }

  func export(archive: RustStr) -> CzOutcome {
    let archive = FilePath(archive.toString())

    return CzOutcome { try reader.export(archive: archive) }
  }
}

func readInlineExtendedAttributes(buffer: RustVec<UInt8>) -> CzOutcome {
  let buffer = [UInt8](buffer)

  return CzOutcome { try EXT4.EXT4Reader.readInlineExtendedAttributes(from: buffer) }
}

func readBlockExtendedAttributes(buffer: RustVec<UInt8>) -> CzOutcome {
  let buffer = [UInt8](buffer)

  return CzOutcome { try EXT4.EXT4Reader.readBlockExtendedAttributes(from: buffer) }
}

func rootInode() -> CzOutcome {
  CzOutcome { EXT4.Inode.Root() }
}

func compressExtendedAttributeName(name: RustStr) -> CzOutcome {
  let name = name.toString()

  return CzOutcome {
    let compressed = EXT4.ExtendedAttribute.compressName(name)
    return (compressed.id, compressed.str)
  }
}

func decompressExtendedAttributeName(id: Int, suffix: RustStr) -> CzOutcome {
  let suffix = suffix.toString()

  return CzOutcome { EXT4.ExtendedAttribute.decompressName(id: id, suffix: suffix) }
}

func newExt4Formatter(devicePath: RustStr, options: RustFormatterOptions) -> CzOutcome {
  let devicePath = FilePath(devicePath.toString())
  var journal: EXT4.JournalConfig?
  if options.hasJournal() {
    let mode = options.hasJournalMode() ? EXT4.JournalConfig.JournalMode(options.journalMode()) : nil
    journal = EXT4.JournalConfig(size: options.journalSize(), defaultMode: mode)
  }
  let blockSize = options.blockSize()
  let minDiskSize = options.minDiskSize()

  return CzOutcome {
    CzExt4Formatter(
      try EXT4.Formatter(devicePath, blockSize: blockSize, minDiskSize: minDiskSize, journal: journal)
    )
  }
}

/// The outcome holds the size and the number of items.
func scanArchiveHeaders(format: RustStr, filter: RustStr, file: RustStr) -> CzOutcome {
  let format = format.toString()
  let filter = filter.toString()
  let file = URL(filePath: file.toString())

  return CzOutcome {
    let totals = try EXT4.Formatter.scanArchiveHeaders(
      format: try archiveFormat(format),
      filter: try archiveFilter(filter),
      file: file
    )
    return (totals.size, totals.items)
  }
}

/// Unchecked: Rust's `Formatter` isn't `Sync`, so one call runs at a time.
final class CzExt4Formatter: @unchecked Sendable {
  let formatter: EXT4.Formatter

  init(_ formatter: EXT4.Formatter) {
    self.formatter = formatter
  }

  func link(link: RustStr, target: RustStr) -> CzOutcome {
    let link = FilePath(link.toString())
    let target = FilePath(target.toString())

    return CzOutcome { try formatter.link(link: link, target: target) }
  }

  func unlink(path: RustStr, directoryWhiteout: Bool) -> CzOutcome {
    let path = FilePath(path.toString())

    return CzOutcome { try formatter.unlink(path: path, directoryWhiteout: directoryWhiteout) }
  }

  /// The dates are seconds since 1970. `buf` stands for `nil` when `hasBuf` is
  /// false, and the xattrs when `hasXattrs` is.
  func create(
    path: RustStr,
    link: RustString?,
    mode: UInt16,
    access: Double,
    modification: Double,
    creation: Double,
    now: Double,
    hasBuf: Bool,
    buf: RustVec<UInt8>,
    uid: UInt32?,
    gid: UInt32?,
    hasXattrs: Bool,
    xattrNames: RustVec<RustString>,
    xattrLengths: RustVec<UInt64>,
    xattrValues: RustVec<UInt8>,
    recursion: Bool
  ) -> CzOutcome {
    let path = FilePath(path.toString())
    let link = link.map { FilePath($0.toString()) }
    var ts = FileTimestamps(
      access: Date(timeIntervalSince1970: access),
      modification: Date(timeIntervalSince1970: modification),
      creation: Date(timeIntervalSince1970: creation)
    )
    ts.now = Date(timeIntervalSince1970: now)
    let stream = hasBuf ? InputStream(data: Data(buf)) : nil
    let xattrs = hasXattrs ? dataMap(names: xattrNames, lengths: xattrLengths, values: xattrValues) : nil

    return CzOutcome {
      stream?.open()
      defer { stream?.close() }

      return try formatter.create(
        path: path,
        link: link,
        mode: mode,
        ts: ts,
        buf: stream,
        uid: uid,
        gid: gid,
        xattrs: xattrs,
        recursion: recursion
      )
    }
  }

  func close() -> CzOutcome {
    CzOutcome { try formatter.close() }
  }

  func unpack(source: RustStr, format: RustStr, compression: RustStr, progress: RustProgressHandler) -> CzOutcome {
    // Not `Sendable`, but a class only the task below uses.
    nonisolated(unsafe) let formatter = formatter
    let source = URL(filePath: source.toString())
    let format = format.toString()
    let compression = compression.toString()
    let progress = progressHandler(progress)

    return CzOutcome {
      let format = try archiveFormat(format)
      let filter = try archiveFilter(compression)
      return try blocking {
        try await formatter.unpack(source: source, format: format, compression: filter, progress: progress)
      }
    }
  }

  func unpackReader(reader: CzArchiveReader, progress: RustProgressHandler) -> CzOutcome {
    // Not `Sendable`, but classes only the task below uses.
    nonisolated(unsafe) let formatter = formatter
    nonisolated(unsafe) let reader = reader.reader
    let progress = progressHandler(progress)

    return CzOutcome { try blocking { try await formatter.unpack(reader: reader, progress: progress) } }
  }
}

extension CzOutcome {
  func ext4Formatter() -> CzExt4Formatter { taken() }

  func inodeNumber() -> UInt32 {
    (taken() as (EXT4.InodeNumber, EXT4.Inode)).0
  }

  /// An inode, alone or with its number.
  func inodeBytes() -> RustVec<UInt8> {
    let inode = held((EXT4.InodeNumber, EXT4.Inode).self)?.1 ?? taken()
    return rustBytes(memoryBytes(of: inode))
  }

  func compressedNameId() -> UInt8 {
    (taken() as (UInt8, String)).0
  }

  func compressedNameStr() -> String {
    (taken() as (UInt8, String)).1
  }

  func scannedSize() -> Int64 {
    (taken() as (Int64, Int)).0
  }

  func scannedItems() -> Int {
    (taken() as (Int64, Int)).1
  }
}

/// A value's bytes, as it is laid out in memory.
private func memoryBytes<T>(of value: T) -> Data {
  withUnsafeBytes(of: value) { Data($0) }
}

func unpackExt4(
  unpacker: RustExt4Unpacker,
  image: CzImage,
  platform: RustPlatform,
  at: RustStr,
  progress: RustProgressHandler
) -> CzOutcome {
  // Not `Sendable`, but a value only the task below uses.
  nonisolated(unsafe) let unpacker = EXT4Unpacker(unpacker)
  let image = image.image
  let platform = Platform(platform)
  let at = URL(filePath: at.toString())
  let progress = progressHandler(progress)

  return CzOutcome {
    try blocking { try await unpacker.unpack(image, for: platform, at: at, progress: progress) }
  }
}

// For unit tests that compare Rust's copies with Swift's values.

func ext4FileModeFlags() -> RustVec<UInt16> {
  let flags: [EXT4.FileModeFlag] = [
    .S_IXOTH, .S_IWOTH, .S_IROTH, .S_IXGRP, .S_IWGRP, .S_IRGRP, .S_IXUSR, .S_IWUSR, .S_IRUSR, .S_ISVTX,
    .S_ISGID, .S_ISUID, .S_IFIFO, .S_IFCHR, .S_IFDIR, .S_IFBLK, .S_IFREG, .S_IFLNK, .S_IFSOCK, .TypeMask,
  ]
  let vec = RustVec<UInt16>()
  for flag in flags {
    vec.push(value: EXT4.Inode.Mode(flag, 0))
  }
  return vec
}

func ext4PrefixMapKeys() -> RustVec<Int> {
  let vec = RustVec<Int>()
  for key in EXT4.ExtendedAttribute.prefixMap.keys.sorted() {
    vec.push(value: key)
  }
  return vec
}

func ext4PrefixMapValues() -> RustVec<RustString> {
  rustStrings(EXT4.ExtendedAttribute.prefixMap.sorted { $0.key < $1.key }.map(\.value))
}

func ext4SuperBlockMagic() -> UInt16 {
  EXT4.SuperBlockMagic
}

/// The sizes of `SuperBlock` and `Inode`, each followed by the offsets of the
/// fields Rust's test names.
func ext4Layout() -> RustVec<UInt64> {
  let superBlock: [PartialKeyPath<EXT4.SuperBlock>] = [
    \.magic, \.mmpBlock, \.lastErrorBlock, \.reserved, \.checksum,
  ]
  let inode: [PartialKeyPath<EXT4.Inode>] = [\.block, \.projid, \.inlineXattrs]
  let layout =
    [MemoryLayout<EXT4.SuperBlock>.size]
    + superBlock.map { MemoryLayout<EXT4.SuperBlock>.offset(of: $0) ?? -1 }
    + [MemoryLayout<EXT4.Inode>.size]
    + inode.map { MemoryLayout<EXT4.Inode>.offset(of: $0) ?? -1 }

  let vec = RustVec<UInt64>()
  for value in layout {
    vec.push(value: UInt64(bitPattern: Int64(value)))
  }
  return vec
}

/// A `FileTimestamps`' `accessLo` and `accessHi`, for a date as seconds since
/// 1970.
func fileTimestampsAccess(seconds: Double) -> RustVec<UInt32> {
  let timestamps = FileTimestamps(access: Date(timeIntervalSince1970: seconds), modification: nil, creation: nil)
  let vec = RustVec<UInt32>()
  vec.push(value: timestamps.accessLo)
  vec.push(value: timestamps.accessHi)
  return vec
}
