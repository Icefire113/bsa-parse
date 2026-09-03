# bsa-parse

A Rust library for parsing Bethesda BSA archive files, targeting Skyrim Special Edition.

## Usage

```rust
use bsa_parse::BsaArchive;

let archive = BsaArchive::new("Skyrim - Textures0.bsa")?;

for path in archive.iter_full_filenames() {
    // ...
}
```

## CLI example

A small binary example is included that lists and extracts every file:

```shell
cargo run -p list-contents -- "path/to/Skyrim - Textures0.bsa"
```

## Intentional limitations

This crate is a focused, read-only parser for Skyrim SE archives. It does **not** support the full range of BSA variants, by design:

- **Skyrim SE only.** Only BSA version `0x69` (105) is accepted. Skyrim LE (`0x68`), Oblivion, and Fallout archives are rejected.
- **Directory/file name flags required.** Archives must set `INCLUDE_DIRECTORY_NAMES` and `INCLUDE_FILE_NAMES`; archives without them are rejected.
- **LZ4 compression only.** Compressed files use LZ4, as in SSE. zlib compression (Skyrim LE, Oblivion) and the Xbox 360 `XMEM` codec are not implemented.
- **No Xbox 360 / big-endian support.** Archives with the `XBOX360_ARCHIVE` flag are not handled.
- **No top-level files.** `get_file` requires a `folder\file` path; files at the archive root cannot be retrieved.
- **In-memory only.** File data is returned as `Vec<u8>`; there is no streaming or on-disk extraction.
- **Read-only.** Extracted data is decompressed on the fly; nothing is written back.
