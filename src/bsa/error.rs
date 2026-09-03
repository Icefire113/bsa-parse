use common_util::errors::UtilReadError;

use crate::bsa::hash::BsaHash;

#[derive(Debug, thiserror::Error)]
pub enum BsaArchiveError {
    #[error("IO Error: {:?}", _0)]
    Io(#[from] std::io::Error),

    #[error("Read Error: {:?}", _0)]
    ReadError(#[from] UtilReadError),

    #[error("Invalid Magic: {:#?}", _0)]
    InvalidMagic([u8; 4]),

    #[error("Unsupported Version: {}", _0)]
    UnsupportedVersion(u32),

    #[error("Malformed archive: {}", _0)]
    Malformed(String),

    #[error("Decompress Error")]
    DecompressError(#[from] Lz4DecompressError),

    #[error("Cannot get top level file")]
    CannotGetTopLevelFile,

    #[error("Cannot read file")]
    CannotReadFile(#[source] std::io::Error),

    #[error("Folder not found, hash was {:?}", _0)]
    FolderNotFound(BsaHash),

    #[error("File not found, hash was {:?}", _0)]
    FileNotFound(BsaHash),
}

#[derive(Debug, thiserror::Error)]
pub enum Lz4DecompressError {
    #[error("Frame Decompress Error {:?}", _0)]
    FrameDecompress(#[from] std::io::Error),
    #[error("Block Decompress Error: {:?}", _0)]
    BlockDecompress(#[from] lz4_flex::block::DecompressError),
}
