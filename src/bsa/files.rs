use std::{collections::HashMap, io::Read};

use common_util::bin_read::{read_bstring, read_bzstring, read_n_bytes, read_u32_le, read_u64_le};

use crate::bsa::{
    error::BsaArchiveError, folder::BsaFolderRecord, hash::BsaHash, header::ArchiveFlags,
};

#[derive(Debug)]
pub struct BsaFileRecordBlock {
    /// Name of the folder. Only present if Bit 1 of archiveFlags is set, we only parse bsa files in which this flag is set
    pub folder_name: String,
    pub file_records: HashMap<BsaHash, BsaFileRecord>,
}

impl BsaFileRecordBlock {
    pub fn parse(
        reader: &mut impl Read,
        folder_meta: &HashMap<BsaHash, BsaFolderRecord>,
    ) -> Result<Self, BsaArchiveError> {
        let folder_name = read_bzstring(reader)?;
        let folder_hash = BsaHash::from_path(&folder_name, true);

        let num_files = folder_meta
            .get(&folder_hash)
            .ok_or(BsaArchiveError::Malformed(format!(
                "Folder {:#?} could not be found in our folder list, hash was {:?}",
                folder_name, folder_hash
            )))?
            .count;
        let mut file_records: HashMap<BsaHash, BsaFileRecord> = HashMap::new();
        for _ in 0..num_files {
            let record = BsaFileRecord::parse(reader)?;
            file_records.insert(record.name_hash, record);
        }

        Ok(Self {
            folder_name,
            file_records,
        })
    }
}

#[derive(Debug)]
pub struct BsaFileRecord {
    name_hash: BsaHash,
    /// - If files are default compressed, this file is not compressed.
    /// - If files are default not compressed, this file is compressed.
    ///
    /// If the file is compressed the file data will have the specification of Compressed File block.
    /// In addition, the size of compressed data is considered to be the ulong "original size" plus the
    /// compressed data size (4 + compressed size).
    ///
    /// `size` is the total length of the file data block, which when `EMBED_FILE_NAMES` is set also
    /// includes the bstring file name (length byte + string).
    pub size_info: BsaFileRecordSizeInfo,

    /// Offset to raw file data for this folder. Note that an "offset" is offset from file byte zero (start), NOT from this location.
    pub offset: u32,
}

#[derive(Debug)]
pub struct BsaFileRecordSizeInfo {
    size: u32,
    pub invert_default_compression: bool,
}

impl From<u32> for BsaFileRecordSizeInfo {
    fn from(value: u32) -> Self {
        if value & 0x4000_0000 != 0 {
            Self {
                invert_default_compression: true,
                size: value & !0x4000_0000,
            }
        } else {
            Self {
                invert_default_compression: false,
                size: value,
            }
        }
    }
}

impl BsaFileRecord {
    pub fn parse(reader: &mut impl Read) -> Result<Self, BsaArchiveError> {
        let name_hash: BsaHash = read_u64_le(reader)?.into();
        let size: u32 = read_u32_le(reader)?;
        let offset: u32 = read_u32_le(reader)?;
        Ok(Self {
            name_hash,
            size_info: size.into(),
            offset,
        })
    }
}

#[derive(Debug)]
pub struct BsaCompressedFileBlock {
    /// Full path and name of the file. Only present if Bit 9 of archiveFlags is set.
    #[allow(unused)]
    pub(crate) name: Option<String>,
    pub(crate) original_size: u32,
    /// lz4 compressed data (zlib on SE)
    pub(crate) data: Vec<u8>,
}

impl BsaCompressedFileBlock {
    pub fn parse(
        reader: &mut impl Read,
        size_info: &BsaFileRecordSizeInfo,
        flags: &ArchiveFlags,
    ) -> Result<Self, BsaArchiveError> {
        let (name, name_len) = if flags.contains(ArchiveFlags::EMBED_FILE_NAMES) {
            let name = read_bstring(reader)?;
            let name_len = 1 + name.len();
            (Some(name), name_len)
        } else {
            (None, 0)
        };
        let original_size: u32 = read_u32_le(reader)?;
        let compressed_len = size_info.size as usize - 4 - name_len;
        let data: Vec<u8> = read_n_bytes(reader, compressed_len)?;
        Ok(Self {
            name,
            original_size,
            data,
        })
    }
}

#[derive(Debug)]
pub struct BsaUncompressedFileBlock {
    /// Full path and name of the file. Only present if Bit 9 of archiveFlags is set.
    #[allow(unused)]
    pub(crate) name: Option<String>,
    pub(crate) data: Vec<u8>,
}

impl BsaUncompressedFileBlock {
    pub fn parse(
        reader: &mut impl Read,
        size_info: &BsaFileRecordSizeInfo,
        flags: &ArchiveFlags,
    ) -> Result<Self, BsaArchiveError> {
        let (name, name_len) = if flags.contains(ArchiveFlags::EMBED_FILE_NAMES) {
            let name = read_bstring(reader)?;
            let name_len = 1 + name.len();
            (Some(name), name_len)
        } else {
            (None, 0)
        };
        let data: Vec<u8> = read_n_bytes(reader, size_info.size as usize - name_len)?;

        Ok(Self { name, data })
    }
}
