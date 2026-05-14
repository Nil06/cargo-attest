//! cargo-attest — consumer-side attestation of published Rust artifacts.
//!
//! Attests that a binary you downloaded (from a GitHub release, a crates.io
//! artifact, or anywhere else) was actually built from the source it claims.

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod hash;
mod sources;
mod verdict;

/// cargo-attest: attest that published Rust binaries match their declared source.
#[derive(Parser, Debug)]
#[command(name = "cargo-attest", version, about, long_about = None)]
#[command(bin_name = "cargo")]
enum CargoCli {
    Attest(AttestArgs),
}

#[derive(Parser, Debug)]
#[command(
    name = "attest",
    about = "Attest a published binary matches its source"
)]
struct AttestArgs {
    #[command(subcommand)]
    command: AttestCommand,
}

#[derive(Subcommand, Debug)]
enum AttestCommand {
    /// Attest a GitHub release artifact against the tagged source.
    Release {
        /// owner/repo (e.g. "BurntSushi/ripgrep")
        repo: String,
        /// Release tag (e.g. "14.1.0")
        tag: String,
        /// Optional asset name filter
        #[arg(long)]
        asset: Option<String>,
    },
    /// Attest a crates.io artifact for a given crate version.
    Crate {
        /// Crate name
        name: String,
        /// Version
        version: String,
    },
    /// Hash a local file (utility).
    Hash {
        /// Path to file
        path: std::path::PathBuf,
    },
}

fn main() {
    if let Err(err) = run() {
        eprintln!("ERROR: {err:#}");
        std::process::exit(3);
    }
}

fn run() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let CargoCli::Attest(args) = CargoCli::parse();

    match args.command {
        AttestCommand::Release { repo, tag, asset } => {
            commands::release::run(&repo, &tag, asset.as_deref())
        }
        AttestCommand::Crate { name, version } => commands::krate::run(&name, &version),
        AttestCommand::Hash { path } => commands::hash_cmd::run(&path),
    }
}
