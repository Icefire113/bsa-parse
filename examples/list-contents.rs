use std::env::args;

use anyhow::Context;

use bsa_parse::BsaArchive;

fn main() -> anyhow::Result<()> {
    let path = args().nth(1).context("Usage: bsa-parse <bsa_file>")?;
    let file = BsaArchive::new(&path)?;

    for p in file.iter_full_filenames() {
        println!("{}", p);
        file.get_file(&p)?;
    }

    Ok(())
}
