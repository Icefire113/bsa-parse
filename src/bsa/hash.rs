use std::fmt::{Debug, Display};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct BsaHash {
    pub low: u32,
    pub high: u32,
}

impl BsaHash {
    /// Constructs a hash from a path and a flag indicating whether it's a folder.
    ///
    /// A note to past bethesda devs: this makes no sense, was there really no better hash function available?
    /// thank you claude for turning the c code into rust so I didnt have to
    pub fn from_path(path: impl AsRef<str>, is_folder: bool) -> Self {
        // adapted from: https://en.uesp.net/wiki/Oblivion_Mod:Hash_Calculation#Pure_C

        // Normalize: '/' -> '\\', everything else ASCII-lowercased.
        let mut buf: Vec<u8> = path
            .as_ref()
            .bytes()
            .map(|b| {
                if b == b'/' {
                    b'\\'
                } else {
                    b.to_ascii_lowercase()
                }
            })
            .collect();
        if buf.is_empty() {
            panic!("NO HASH EMPTY PATH")
        }

        let len = buf.len();

        // Extension start index, if this is a file with one.
        let ext_pos: Option<usize> = if is_folder {
            None
        } else {
            buf.iter().rposition(|&b| b == b'.')
        };

        // ---- Hash 1: extension bytes ----
        let mut hash: u64 = 0;
        let mut cur_len = len; // shrinks to exclude the extension, if present
        let mut end = len;
        let mut ext_str = String::new();

        if let Some(ext) = ext_pos {
            for &b in &buf[ext..len] {
                hash = hash.wrapping_mul(0x1003f).wrapping_add(b as u64);
            }
            // Save the (already-normalized) extension as a string for the match below.
            ext_str = String::from_utf8_lossy(&buf[ext..len]).into_owned();

            cur_len = ext;
            end = ext;
            buf.truncate(end);
        }

        // ---- Body hash ----
        let mut hash2_32: u32 = 0;
        let inner_end = end as isize - 2;
        if inner_end > 1 {
            for &b in &buf[1..inner_end as usize] {
                hash2_32 = hash2_32.wrapping_mul(0x1003f).wrapping_add(b as u32);
            }
        }

        hash = hash.wrapping_add(hash2_32 as u64);
        hash <<= 32;

        // ---- Hash 2: length/edges/extension bits ----
        let mut hash2: u32 = buf[cur_len - 1] as u32;
        hash2 |= if cur_len > 2 {
            (buf[cur_len - 2] as u32) << 8
        } else {
            0
        };
        hash2 |= (cur_len as u32) << 16;
        hash2 |= (buf[0] as u32) << 24;

        match ext_str.as_str() {
            s if s.starts_with(".kf") => hash2 |= 0x80,
            s if s.starts_with(".nif") => hash2 |= 0x8000,
            s if s.starts_with(".dds") => hash2 |= 0x8080,
            s if s.starts_with(".wav") => hash2 |= 0x80000000,
            _ => {}
        }

        let full = hash.wrapping_add(hash2 as u64);
        BsaHash {
            low: full as u32,
            high: (full >> 32) as u32,
        }
    }
}

impl From<u64> for BsaHash {
    fn from(value: u64) -> Self {
        Self {
            low: value as u32,
            high: (value >> 32) as u32,
        }
    }
}

impl Into<u64> for BsaHash {
    fn into(self) -> u64 {
        (self.high as u64) << 32 | self.low as u64
    }
}

impl Debug for BsaHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08x}:{:08x}", self.high, self.low)
    }
}

impl Display for BsaHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08x}{:08x}", self.high, self.low)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        let hash: u64 = BsaHash::from_path("interface\\marketplace\\buttons\\xbox", true).into();
        assert_eq!(hash, 0x17ea7da269226f78)
    }

    #[test]
    fn test_path_normalization() {
        assert_eq!(
            BsaHash::from_path("textures/tree", true),
            BsaHash::from_path("Textures\\Tree", true)
        );
        assert_eq!(
            BsaHash::from_path("tree_1.dds", false),
            BsaHash::from_path("Tree_1.DDS", false)
        );
    }
}
