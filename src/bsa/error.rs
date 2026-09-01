use crate::util;

#[derive(Debug, thiserror::Error)]
pub enum BsaArchiveError {
    #[error("IO Error: {:?}", _0)]
    Io(#[from] std::io::Error),

    #[error("Read Error: {:?}", _0)]
    ReadError(#[from] util::errors::UtilReadError),

    #[error("Invalid Magic: {:?}", _0)]
    InvalidMagic([u8; 4]),

    #[error("Unsupported Version: {}", _0)]
    UnsupportedVersion(u32),

    #[error("Malformed archive: {}", _0)]
    Malformed(String),

    #[error("Decompress Error: {:?}", _0)]
    DecompressError(#[from] lz4_flex::block::DecompressError),
}
