use std::string::FromUtf8Error;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum UtilReadError {
    #[error("IO Error")]
    IoError(#[from] std::io::Error),

    #[error("UTF-8 Parse Error")]
    Utf8ParseError(#[from] FromUtf8Error),
}
