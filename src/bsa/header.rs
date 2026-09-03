use std::io::Read;

use bitflags::bitflags;
use common_util::bin_read::{read_n_bytes_const, read_u16_le, read_u32_le};

use crate::bsa::error::BsaArchiveError;

const HEADER_MAGIC: [u8; 4] = *b"BSA\0";

#[derive(Debug)]
pub struct BsaHeader {
    pub archive_flags: ArchiveFlags,
    pub folder_count: u32,
    pub file_count: u32,
    #[allow(unused)]
    pub total_folder_name_len: u32,
    #[allow(unused)]
    pub total_file_name_len: u32,
    pub file_flags: FileFlags,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FileFlags: u16 {
        const MESHES = 0x1;
        const TEXTURES = 0x2;
        const MENUS = 0x4;
        const SOUNDS = 0x8;
        const VOICES = 0x10;
        const SHADERS = 0x20;
        const TREES = 0x40;
        const FONTS = 0x80;
        const MISC = 0x100;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ArchiveFlags: u32 {
        /// Include Directory Names. The game may not load a BSA without this bit set.
        const INCLUDE_DIRECTORY_NAMES = 0x1;

        /// Include File Names. The game may not load a BSA without this bit set.
        const INCLUDE_FILE_NAMES = 0x2;

        /// Compressed Archive. Files are compressed by default (not necessarily all files).
        const COMPRESSED_ARCHIVE = 0x4;

        /// Retain Directory Names. No effect on file structure.
        const RETAIN_DIRECTORY_NAMES = 0x8;

        /// Retain File Names. No effect on file structure.
        const RETAIN_FILE_NAMES = 0x10;

        /// Retain File Name Offsets. No effect on file structure.
        const RETAIN_FILE_NAME_OFFSETS = 0x20;

        /// Xbox360 archive. Hash values and numbers after the header are big-endian.
        const XBOX360_ARCHIVE = 0x40;

        /// Retain Strings During Startup. No effect on file structure.
        const RETAIN_STRINGS_DURING_STARTUP = 0x80;

        /// Embed File Names. File data blocks begin with a bstring containing the full path.
        /// For example: `$2B textures\effects\fxfluidstreamdripatlus.dds`
        /// where `$2B` indicates the name is 43 bytes. Data follows immediately after.
        const EMBED_FILE_NAMES = 0x100;

        /// XMem Codec. Xbox 360 only compression algorithm. Requires `COMPRESSED_ARCHIVE`.
        const XMEM_CODEC = 0x200;
    }
}

impl BsaHeader {
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self, BsaArchiveError> {
        let magic: [u8; 4] = read_n_bytes_const(reader)?;
        if magic != HEADER_MAGIC {
            return Err(BsaArchiveError::InvalidMagic(magic));
        }

        let version: u32 = read_u32_le(reader)?;
        // 0x69 = SSE, 0x68 = SE
        if version != 0x69 {
            return Err(BsaArchiveError::UnsupportedVersion(version));
        }

        let offset: u32 = read_u32_le(reader)?;
        if offset != 36 {
            return Err(BsaArchiveError::Malformed(format!(
                "Header offset {} != 36 (0x00000024)",
                offset
            )));
        }
        let archive_flags: u32 = read_u32_le(reader)?;
        let folder_count: u32 = read_u32_le(reader)?;
        let file_count: u32 = read_u32_le(reader)?;
        let total_folder_name_len: u32 = read_u32_le(reader)?;
        let total_file_name_len: u32 = read_u32_le(reader)?;
        let file_flags: u16 = read_u16_le(reader)?;
        let _padding: u16 = read_u16_le(reader)?;

        Ok(Self {
            archive_flags: ArchiveFlags::from_bits(archive_flags).ok_or(
                BsaArchiveError::Malformed(format!(
                    "Unknown bits set in archive_flags {:?}",
                    archive_flags
                )),
            )?,
            folder_count,
            file_count,
            total_folder_name_len,
            total_file_name_len,
            file_flags: FileFlags::from_bits(file_flags).ok_or(BsaArchiveError::Malformed(
                format!("Unknown bits set in file_flags {:?}", file_flags),
            ))?,
        })
    }
}
