pub mod errors;

use std::io::{BufRead, Read};

use crate::util::errors::UtilReadError;

/// Reads `n` bytes from the reader
pub(crate) fn read_n_bytes<R: Read>(reader: &mut R, n: usize) -> Result<Vec<u8>, UtilReadError> {
    let mut buff = vec![0u8; n];
    reader.read_exact(&mut buff)?;
    Ok(buff)
}

/// Reads `n` bytes from the reader, useful when you want to avoid allocating a `Vec`
pub(crate) fn read_n_bytes_const<R: Read, const N: usize>(
    reader: &mut R,
) -> Result<[u8; N], UtilReadError> {
    let mut buff = [0u8; N];
    reader.read_exact(&mut buff)?;
    Ok(buff)
}

/// Reads a u8 from the reader
pub(crate) fn read_u8_le<R: Read>(reader: &mut R) -> Result<u8, UtilReadError> {
    let mut buff = [0u8; size_of::<u8>()];
    reader.read_exact(&mut buff)?;
    Ok(u8::from_le_bytes(buff))
}

/// Reads a u16 from the reader
pub(crate) fn read_u16_le<R: Read>(reader: &mut R) -> Result<u16, UtilReadError> {
    let mut buff = [0u8; size_of::<u16>()];
    reader.read_exact(&mut buff)?;
    Ok(u16::from_le_bytes(buff))
}

/// Reads a u32 from the reader
pub(crate) fn read_u32_le<R: Read>(reader: &mut R) -> Result<u32, UtilReadError> {
    let mut buff = [0u8; size_of::<u32>()];
    reader.read_exact(&mut buff)?;
    Ok(u32::from_le_bytes(buff))
}

/// Reads a u64 from the reader
pub(crate) fn read_u64_le<R: Read>(reader: &mut R) -> Result<u64, UtilReadError> {
    let mut buff = [0u8; size_of::<u64>()];
    reader.read_exact(&mut buff)?;
    Ok(u64::from_le_bytes(buff))
}

/// Reads a C style string from the reader
pub(crate) fn read_string<R: BufRead>(reader: &mut R) -> Result<String, UtilReadError> {
    let mut buff = vec![];
    reader.read_until(b'\0', &mut buff)?;
    // dont put the null terminator in the string
    buff.pop();
    Ok(String::from_utf8(buff)?)
}

/// Reads a byte-prefixed string, formatted as:
/// `<len><str>`
/// where `len` is a u8, and the next `len` bytes are a null terminated string.
///
/// Note that this function will check for utf-8 validity but not the presence of a null terminator (as rust does not need them)
pub(crate) fn read_bzstring(reader: &mut impl Read) -> Result<String, UtilReadError> {
    let len: u8 = read_u8_le(reader)?;
    let mut buff = vec![0u8; len as usize];
    reader.read_exact(&mut buff)?;

    if buff.pop() != Some(b'\0') {
        return Err(UtilReadError::ExpectedNullTerminator);
    }
    Ok(String::from_utf8(buff)?)
}

/// Reads a byte-prefixed string, formatted as:
/// `<len><str>`
/// where `len` is a u8, and the next `len` bytes are a string.
///
/// Note that this function will check for utf-8 validity
pub(crate) fn read_bstring(reader: &mut impl Read) -> Result<String, UtilReadError> {
    let len: u8 = read_u8_le(reader)?;
    let mut buff = vec![0u8; len as usize];
    reader.read_exact(&mut buff)?;

    Ok(String::from_utf8(buff)?)
}
