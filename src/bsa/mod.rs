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
        error::{BsaArchiveError, Lz4DecompressError},
        files::{BsaCompressedFileBlock, BsaFileRecordBlock, BsaUncompressedFileBlock},
        folder::BsaFolderRecord,
        hash::BsaHash,
        header::{ArchiveFlags, BsaHeader, FileFlags},
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
    /// A mapping of file name hashes to file names, useful for knowing what files are in this archive
    filename_map: HashMap<BsaHash, String>,
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

        // only present when ArchiveFlags::INCLUDE_FILE_NAMES is set
        let mut filename_map: HashMap<BsaHash, String> = HashMap::new();
        let mut br: BufReader<&File> = BufReader::new(&file);
        for _ in 0..header.file_count {
            let file_name = read_string(&mut br)?;
            filename_map.insert(BsaHash::from_path(&file_name, false), file_name);
        }

        Ok(Self {
            file,
            header,
            folders,
            file_blocks,
            filename_map,
        })
    }

    pub fn iter_full_filenames(&self) -> impl Iterator<Item = String> {
        self.file_blocks
            .values()
            .map(|b| {
                b.file_records.keys().map(|file_name_hash| {
                    b.folder_name.clone() + "\\" + &self.filename_map[file_name_hash].to_string()
                })
            })
            .flatten()
    }

    /// Iterates over the file names in this archive
    pub fn iter_filenames(&self) -> impl Iterator<Item = &String> {
        self.filename_map.values()
    }

    /// Iterates over the file blocks in this archive, note that despite the name,
    /// this actally iterates over the FOLDERS in this archive
    /// (as well as some of the metadata about the files in those folders)
    pub fn iter_file_blocks(&self) -> impl Iterator<Item = &BsaFileRecordBlock> {
        self.file_blocks.values()
    }

    /// Iterates over the folder records in this archive
    ///
    /// Basically just the folders in this archive
    pub fn iter_folder_records(&self) -> impl Iterator<Item = &BsaFolderRecord> {
        self.folders.values()
    }

    /// Number of files in this archive
    pub fn file_count(&self) -> u32 {
        self.header.file_count
    }

    /// Number of folders in this archive
    pub fn folder_count(&self) -> u32 {
        self.header.folder_count
    }

    /// Archive file flags
    pub fn file_flags(&self) -> FileFlags {
        self.header.file_flags
    }

    /// Archive flags
    pub fn archive_flags(&self) -> ArchiveFlags {
        self.header.archive_flags
    }

    /// Gets a file from the archive, performs path normalization
    pub fn get_file(&self, file_path: &str) -> Result<Vec<u8>, BsaArchiveError> {
        // normalize path
        let file_path = file_path.replace("/", "\\").to_ascii_lowercase();
        let (folder, file_name) = file_path
            .rsplit_once("\\")
            .ok_or(BsaArchiveError::CannotGetTopLevelFile)?;

        match self.read_file_block(folder, file_name)? {
            Either::Left(compressed_block) => Ok(Self::lz4_decompress(
                &compressed_block.data,
                compressed_block.original_size as usize,
            )?),
            Either::Right(uncompressed_block) => Ok(uncompressed_block.data),
        }
    }

    /// Helper to decompress lz4 blocks, handles both frame and block decompression
    fn lz4_decompress(src: &[u8], decompressed_size: usize) -> Result<Vec<u8>, Lz4DecompressError> {
        if src.starts_with(&[0x04, 0x22, 0x4D, 0x18]) {
            let mut buf = Vec::with_capacity(decompressed_size);
            FrameDecoder::new(src).read_to_end(&mut buf)?;
            Ok(buf)
        } else {
            Ok(lz4_flex::decompress(src, decompressed_size)?)
        }
    }

    fn read_file_block(
        &self,
        folder: &str,
        file: &str,
    ) -> Result<Either<BsaCompressedFileBlock, BsaUncompressedFileBlock>, BsaArchiveError> {
        let folder_hash = BsaHash::from_path(folder, true);
        let file_hash = BsaHash::from_path(file, false);

        let folder_rec = self
            .file_blocks
            .get(&folder_hash)
            .ok_or(BsaArchiveError::FolderNotFound(folder_hash))?;
        let file_rec = folder_rec
            .file_records
            .get(&file_hash)
            .ok_or(BsaArchiveError::FolderNotFound(file_hash))?;

        let mut br: BufReader<&File> = BufReader::new(&self.file);
        br.seek(std::io::SeekFrom::Start(file_rec.offset as u64))
            .map_err(BsaArchiveError::CannotReadFile)?;

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
