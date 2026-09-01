use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read, Seek},
    path::Path,
};

use either::Either;
use lz4_flex::frame::FrameDecoder;

use crate::{
    bsa::{
        error::BsaArchiveError,
        files::{BsaCompressedFileBlock, BsaFileRecordBlock, BsaUncompressedFileBlock},
        folder::BsaFolderRecord,
        hash::BsaHash,
        header::{ArchiveFlags, BsaHeader},
    },
    util::read_string,
};

pub mod error;

mod files;
mod folder;
mod hash;
mod header;

#[derive(Debug)]
pub struct BsaArchive {
    file: File,
    header: BsaHeader,
    /// Map of folder name hashes to folder metadata
    folders: HashMap<BsaHash, BsaFolderRecord>,
    /// Map of folder name hashes to the things in that folder
    file_blocks: HashMap<BsaHash, BsaFileRecordBlock>,
    filename_list: Vec<String>,
}

impl BsaArchive {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, BsaArchiveError> {
        let mut file: File = File::options().read(true).write(false).open(path)?;
        let header: BsaHeader = BsaHeader::parse(&mut file)?;
        if !header
            .archive_flags
            .contains(ArchiveFlags::INCLUDE_DIRECTORY_NAMES | ArchiveFlags::INCLUDE_FILE_NAMES)
        {
            return Err(BsaArchiveError::Malformed(format!(
                "Missing archive flag bits 1 or 2: {:?}",
                header.archive_flags
            )));
        }

        let mut folders: HashMap<BsaHash, BsaFolderRecord> = HashMap::new();

        for _ in 0..header.folder_count {
            let record = BsaFolderRecord::parse(&mut file)?;
            folders.insert(record.name_hash, record);
        }

        let mut file_blocks: HashMap<BsaHash, BsaFileRecordBlock> = HashMap::new();
        // read folder name first
        for _ in 0..header.folder_count {
            let block = BsaFileRecordBlock::parse(&mut file, &folders)?;
            file_blocks.insert(BsaHash::from_path(&block.folder_name, true), block);
        }
        let mut filename_list: Vec<String> = Vec::new();
        let mut br = BufReader::new(&file);
        for _ in 0..header.file_count {
            filename_list.push(read_string(&mut br)?);
        }

        Ok(Self {
            file,
            header,
            folders,
            file_blocks,
            filename_list,
        })
    }

    pub fn get_file(&mut self, file: &str) -> Result<Vec<u8>, BsaArchiveError> {
        match self.read_file_block(file)? {
            Either::Left(compressed_block) => {
                if compressed_block.data.starts_with(&[0x04, 0x22, 0x4D, 0x18]) {
                    // frame format
                    let mut decompressed: Vec<u8> =
                        Vec::with_capacity(compressed_block.original_size as usize);
                    let x: &[u8] = &compressed_block.data;
                    FrameDecoder::new(x).read_to_end(&mut decompressed)?;
                    Ok(decompressed)
                } else {
                    Ok(lz4_flex::decompress(
                        &compressed_block.data,
                        compressed_block.original_size as usize,
                    )?)
                }
            }
            Either::Right(uncompressed_block) => Ok(uncompressed_block.data),
        }
    }

    fn read_file_block(
        &mut self,
        file: &str,
    ) -> Result<Either<BsaCompressedFileBlock, BsaUncompressedFileBlock>, BsaArchiveError> {
        let (path, file_name) = file
            .rsplit_once("\\")
            .expect("Ill handle errors later, i want to go to bed");

        let folder_hash = BsaHash::from_path(path, true);
        let file_hash = BsaHash::from_path(file_name, false);

        let rec = self.file_blocks.get(&folder_hash).expect("NOT THERE");
        let file_rec = rec.file_records.get(&file_hash).expect("no file :(");

        let mut br = BufReader::new(&self.file);
        br.seek(std::io::SeekFrom::Start(file_rec.offset as u64))
            .expect("file too small");

        let is_compressed = self
            .header
            .archive_flags
            .contains(ArchiveFlags::COMPRESSED_ARCHIVE)
            ^ file_rec.size_info.invert_default_compression;

        Ok(if is_compressed {
            Either::Left(BsaCompressedFileBlock::parse(
                &mut br,
                &file_rec.size_info,
                &self.header.archive_flags,
            )?)
        } else {
            Either::Right(BsaUncompressedFileBlock::parse(
                &mut br,
                &file_rec.size_info,
                &self.header.archive_flags,
            )?)
        })
    }
}
