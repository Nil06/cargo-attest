//! `cargo attest hash <file>` — utility to print the SHA-256 of a local file.
//!
//! Useful for debugging and for comparing manually against published checksums.

use anyhow::Result;
use std::path::Path;

use crate::hash;

pub fn run(path: &Path) -> Result<()> {
    let digest = hash::sha256_file(path)?;
    println!("{}  {}", digest, path.display());
    Ok(())
}
