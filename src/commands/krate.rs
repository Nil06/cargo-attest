//! `cargo attest crate <name> <version>` — attest a crates.io artifact.
//!
//! v0.1: stub. v0.4: implements full crates.io flow.

use anyhow::Result;

use crate::sources::crates_io;

pub fn run(name: &str, version: &str) -> Result<()> {
    tracing::info!(name, version, "crate attest requested");
    let meta = crates_io::fetch_crate_meta(name, version)?;
    tracing::info!(
        crate_name = meta.name,
        crate_version = meta.version,
        repository = ?meta.repository,
        checksum = meta.checksum,
        "crate metadata fetched"
    );
    Ok(())
}
