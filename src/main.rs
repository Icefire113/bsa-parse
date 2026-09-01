use std::{env::args, fs::File, io::Write, time::Instant};

use anyhow::Context;

use crate::bsa::BsaArchive;

mod bsa;
mod util;

fn main() -> anyhow::Result<()> {
    let path = args().nth(1).context("Usage: bsa-parse <bsa_file>")?;
    let start = Instant::now();
    let mut file = BsaArchive::new(&path)?;
    let duration = start.elapsed();
    println!("Took {:?} to parse archive", duration);

    let bytes = file.get_file("interface\\marketplace\\buttons\\pc\\pc_ent.png")?;
    let mut f = File::options()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open("pc_ent.png")?;
    f.write_all(&bytes)?;

    Ok(())
}

// struct BsaArchive {
//     header: BsaHeader,
//     folder_records: Vec<BsaFolderRecord>,
//     file_record_blocks: Vec<BsaFileRecordBlock>,
//     /// If archive flag 0x2 is not set, this block is omitted. A block of lower case file names,
//     /// one after another, each ending in a \0. They are ordered in the same order as those generated
//     /// with the file folder block contents in the BSA archive. These are all the files contained in the
//     /// archive, such as "cuirass.nif" and "cuirass.dds", etc (no paths, just the root names).
//     file_name_blocks: Option<BsaFileNameBlock>,
//     files: Vec<u8>,
// }

// #[derive(Debug)]
// struct BsaCompressedFileBlock {
//     /// Full path and name of the file. Only present if Bit 9 of archiveFlags is set.
//     name: Option<String>,
//     /// Size of uncompressed data.
//     original_size: u32,
//     /// File data that has been compressed with zlib (SE) or LZ4 (SSE).
//     data: Vec<u8>,
// }

// #[derive(Debug)]
// struct BsaUnCompressedFileBlock {
//     /// Full path and name of the file. Only present if Bit 9 of archiveFlags is set.
//     name: Option<String>,
//     data: Vec<u8>,
// }

// #[derive(Debug)]
// struct BsaHeader {
//     /// Constant: "BSA\x00"
//     magic: [u8; 4],
//     /// Currently 104 (0x68) for Skyrim or 105 (0x69) for Skyrim Special Edition.
//     version: BsaVersion,
//     /// Offset of beginning of folder records. All headers are the same size, therefore this value is 36 (0x24).
//     offset: u32,
//     archive_flags: BsaArchiveFlags,
//     folder_count: u32,
//     file_count: u32,
//     /// Total length of all folder names, including \0's but not including the prefixed length byte.
//     total_folder_name_len: u32,
//     /// Total length of all file names, including \0's.
//     total_file_name_len: u32,
//     file_flags: BsaFileFlags,
//     _padding: u16,
// }

// #[derive(Debug)]
// struct BsaFileNameBlock {
//     file_names: Vec<String>,
// }

// #[derive(Debug)]
// struct BsaFolderRecord {
//     name_hash: BsaHash,
//     count: u32,
//     // 4 bytes padding: SSE only
//     offset: u32,
//     // 4 bytes padding: SSE only
// }

// #[derive(Debug)]
// struct BsaFileRecordBlock {
//     /// only if bit 1 of archive_flags is set
//     name: Option<String>,
//     file_record: BsaFileRecord,
// }

// #[derive(Debug)]
// struct BsaFileRecord {
//     name_hash: BsaHash,
//     /// If the 30th bit (0x40000000) is set in the size:
//     /// - If files are default compressed, this file is not compressed.
//     /// - If files are default not compressed, this file is compressed.
//     ///
//     /// If the file is compressed the file data will have the specification of Compressed File block.
//     /// In addition, the size of compressed data is considered to be the ulong "original size" plus the
//     /// compressed data size (4 + compressed size).
//     size: u32,
//     /// Offset to raw file data for this folder. Note that an "offset" is offset from file byte zero (start), NOT from this location.
//     offset: u32,
// }

// #[repr(u32)]
// #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
// enum BsaVersion {
//     Skyrim = 104,
//     SkyrimSE = 105,
// }

// impl TryFrom<u32> for BsaVersion {
//     type Error = anyhow::Error;

//     fn try_from(value: u32) -> Result<Self, Self::Error> {
//         match value {
//             104 => Ok(BsaVersion::Skyrim),
//             105 => Ok(BsaVersion::SkyrimSE),
//             _ => Err(anyhow::anyhow!("Unknown BSA version {value}")),
//         }
//     }
// }
