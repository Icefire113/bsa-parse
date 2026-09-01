use std::io::Read;

use crate::{
    bsa::{error::BsaArchiveError, hash::BsaHash},
    util::{read_u32_le, read_u64_le},
};

#[derive(Debug)]
pub struct BsaFolderRecord {
    /// Hash of the folder name (eg: menus\chargen)
    pub name_hash: BsaHash,
    /// Amount of files in this folder.
    pub count: u32,
    /// Offset to file records for this folder. (Subtract totalFileNameLength to get the actual offset within the file.)
    pub offset: u32,
}

impl BsaFolderRecord {
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self, BsaArchiveError> {
        let name_hash: BsaHash = read_u64_le(reader)?.into();

        let count: u32 = read_u32_le(reader)?;
        // Only SSE (ver 0x69 has this padding)
        let _padding: u32 = read_u32_le(reader)?;

        let offset: u32 = read_u32_le(reader)?;
        // Only SSE (ver 0x69 has this padding)
        let _padding: u32 = read_u32_le(reader)?;

        Ok(Self {
            name_hash,
            count,
            offset,
        })
    }
}
