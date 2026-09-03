//! Parser for Bethesda BSA archive files (Skyrim SE only).
//!
//! ```
//! use bsa_parse::BsaArchive;
//!
//! # fn example() {
//! let archive = BsaArchive::new("Skyrim - Textures0.bsa").unwrap();
//! # }
//! ```

pub mod bsa;
pub mod util;

pub use bsa::BsaArchive;