//! `cargo attest release <repo> <tag>` — attest a GitHub release artifact.
//!
//! v0.2 flow:
//! 1. Fetch release metadata from GitHub API.
//! 2. Resolve the tag → commit SHA (proves the release points at a real commit).
//! 3. Select asset(s) — filter by --asset if given, else attest each one.
//! 4. For each asset:
//!    a. Scan the release body, then `<asset>.sha256`, for a SHA-256 checksum.
//!    b. Download the asset to a temp file.
//!    c. Hash it.
//!    d. Compare. Emit Pass / Fail / Skip check.
//! 5. Aggregate checks into a Verdict.

use anyhow::{Context, Result};
use tempfile::NamedTempFile;

use crate::hash;
use crate::sources::github;
use crate::verdict::{Check, CheckOutcome, Subject, Verdict};

#[derive(Debug)]
struct DeclaredChecksum {
    value: String,
    source: String,
}

pub fn run(repo: &str, tag: &str, asset_filter: Option<&str>) -> Result<()> {
    tracing::info!(repo, tag, ?asset_filter, "release attest");
    let meta = github::fetch_release(repo, tag).context("fetching release metadata")?;

    let subject = Subject::GithubRelease {
        repo: repo.to_string(),
        tag: tag.to_string(),
        asset: asset_filter.map(str::to_string),
    };

    let mut checks: Vec<Check> = Vec::new();

    // Check 1: tag resolves to a commit.
    match &meta.commit_sha {
        Some(sha) => checks.push(Check {
            name: "tag-resolves-to-commit".into(),
            outcome: CheckOutcome::Pass,
            detail: format!("tag {} → {}", meta.tag, &sha[..sha.len().min(12)]),
        }),
        None => checks.push(Check {
            name: "tag-resolves-to-commit".into(),
            outcome: CheckOutcome::Fail,
            detail: "could not resolve tag to commit SHA".into(),
        }),
    }

    // Filter assets.
    let assets: Vec<_> = meta
        .assets
        .iter()
        .filter(|a| match asset_filter {
            Some(f) => a.name.contains(f),
            None => true,
        })
        .filter(|a| !is_checksum_sidecar(&a.name))
        .collect();

    if assets.is_empty() {
        let v = Verdict::Unverified {
            subject,
            reason: match asset_filter {
                Some(f) => format!("no assets matched filter {f:?}"),
                None => "release has no assets".into(),
            },
            checks,
        };
        emit(&v);
        std::process::exit(v.exit_code());
    }

    // Per-asset checksum check.
    let mut any_mismatch = false;
    let mut any_pass = false;
    for asset in &assets {
        let declared = declared_checksum_for(&meta, &asset.name)?;
        match declared {
            None => checks.push(Check {
                name: format!("sha256-body-checksum:{}", asset.name),
                outcome: CheckOutcome::Skip,
                detail: "no SHA-256 found in release body or sidecar asset".into(),
            }),
            Some(decl) => {
                let tmp = NamedTempFile::new().context("creating temp file")?;
                let n = github::download_asset(&asset.download_url, tmp.path())
                    .with_context(|| format!("downloading {}", asset.name))?;
                tracing::debug!(bytes = n, "downloaded asset");
                if asset.size > 0 {
                    if n == asset.size {
                        checks.push(Check {
                            name: format!("asset-size:{}", asset.name),
                            outcome: CheckOutcome::Pass,
                            detail: format!("downloaded {n} bytes, matching GitHub metadata"),
                        });
                    } else {
                        any_mismatch = true;
                        checks.push(Check {
                            name: format!("asset-size:{}", asset.name),
                            outcome: CheckOutcome::Fail,
                            detail: format!(
                                "downloaded {n} bytes, GitHub metadata says {}",
                                asset.size
                            ),
                        });
                    }
                }
                let actual = hash::sha256_file(tmp.path())?;
                if hash::eq_hex(&actual, &decl.value) {
                    any_pass = true;
                    checks.push(Check {
                        name: format!("sha256-body-checksum:{}", asset.name),
                        outcome: CheckOutcome::Pass,
                        detail: format!(
                            "declared via {} and computed SHA-256 match ({})",
                            decl.source,
                            &actual[..16]
                        ),
                    });
                } else {
                    any_mismatch = true;
                    checks.push(Check {
                        name: format!("sha256-body-checksum:{}", asset.name),
                        outcome: CheckOutcome::Fail,
                        detail: format!("declared {} != computed {actual}", decl.value),
                    });
                }
            }
        }
    }

    let v = if any_mismatch {
        Verdict::Mismatch {
            subject,
            reason: "one or more asset checksums did not match the declared value".into(),
            checks,
        }
    } else if any_pass {
        Verdict::Trusted { subject, checks }
    } else {
        Verdict::Unverified {
            subject,
            reason: "no checksum found in release body or sidecar asset — cannot establish trust"
                .into(),
            checks,
        }
    };

    emit(&v);
    std::process::exit(v.exit_code());
}

fn declared_checksum_for(
    meta: &github::ReleaseMeta,
    asset_name: &str,
) -> Result<Option<DeclaredChecksum>> {
    if let Some(value) = extract_checksum_for(&meta.body, asset_name) {
        return Ok(Some(DeclaredChecksum {
            value,
            source: "release body".into(),
        }));
    }

    let Some(sidecar) = find_checksum_sidecar(&meta.assets, asset_name) else {
        return Ok(None);
    };

    let text = github::download_asset_text(&sidecar.download_url)
        .with_context(|| format!("downloading checksum sidecar {}", sidecar.name))?;
    let value = extract_checksum_for(&text, asset_name).or_else(|| extract_first_checksum(&text));
    Ok(value.map(|value| DeclaredChecksum {
        value,
        source: format!("sidecar asset {}", sidecar.name),
    }))
}

fn emit(v: &Verdict) {
    if std::env::var("CARGO_ATTEST_JSON").is_ok() {
        println!("{}", serde_json::to_string(v).unwrap());
    } else {
        println!("{}", v.summary());
        if let Verdict::Trusted { checks, .. }
        | Verdict::Unverified { checks, .. }
        | Verdict::Mismatch { checks, .. } = v
        {
            for c in checks {
                let mark = match c.outcome {
                    CheckOutcome::Pass => "✓",
                    CheckOutcome::Fail => "✗",
                    CheckOutcome::Skip => "·",
                };
                println!("  {mark} {} — {}", c.name, c.detail);
            }
        }
    }
}

/// Scan free-form release body text for a SHA-256 hex string near the given filename.
///
/// Heuristic, generous: we accept any 64-hex-char token on the same line as the
/// filename, in either order. Many maintainers paste `sha256sum` output verbatim.
fn extract_checksum_for(body: &str, asset_name: &str) -> Option<String> {
    for line in body.lines() {
        if !line.contains(asset_name) {
            continue;
        }
        // Walk tokens; first 64-hex token wins.
        for tok in line.split(|c: char| c.is_whitespace() || c == '`' || c == '*' || c == '|') {
            let t = tok.trim();
            if is_sha256_hex(t) {
                return Some(t.to_ascii_lowercase());
            }
        }
    }
    None
}

fn extract_first_checksum(text: &str) -> Option<String> {
    for tok in text.split(|c: char| c.is_whitespace() || c == '`' || c == '*' || c == '|') {
        let t = tok.trim();
        if is_sha256_hex(t) {
            return Some(t.to_ascii_lowercase());
        }
    }
    None
}

fn find_checksum_sidecar<'a>(
    assets: &'a [github::Asset],
    asset_name: &str,
) -> Option<&'a github::Asset> {
    let candidates = [
        format!("{asset_name}.sha256"),
        format!("{asset_name}.sha256sum"),
        format!("{asset_name}.sha256.txt"),
    ];
    assets.iter().find(|asset| {
        candidates
            .iter()
            .any(|candidate| asset.name.eq_ignore_ascii_case(candidate))
    })
}

fn is_checksum_sidecar(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".sha256") || lower.ends_with(".sha256sum") || lower.ends_with(".sha256.txt")
}

fn is_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_sha256_when_present() {
        let body = "
            ## Checksums

            `abc123def4567890abc123def4567890abc123def4567890abc123def4567890`  ripgrep-14.1.0-x86_64-linux.tar.gz
        ";
        let s = extract_checksum_for(body, "ripgrep-14.1.0-x86_64-linux.tar.gz").unwrap();
        assert_eq!(s.len(), 64);
    }

    #[test]
    fn returns_none_when_absent() {
        let body = "no hashes here, just vibes";
        assert!(extract_checksum_for(body, "foo.tar.gz").is_none());
    }

    #[test]
    fn ignores_non_hex_64_tokens() {
        let body =
            "thisis64charsofnonhexnonsenseGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG foo.tar.gz";
        assert!(extract_checksum_for(body, "foo.tar.gz").is_none());
    }

    #[test]
    fn extracts_first_checksum_from_sidecar_text() {
        let text = "abc123def4567890abc123def4567890abc123def4567890abc123def4567890";
        let s = extract_first_checksum(text).unwrap();
        assert_eq!(s.len(), 64);
    }

    #[test]
    fn detects_checksum_sidecars() {
        assert!(is_checksum_sidecar("tool.tar.gz.sha256"));
        assert!(is_checksum_sidecar("tool.tar.gz.sha256sum"));
        assert!(is_checksum_sidecar("tool.tar.gz.sha256.txt"));
        assert!(!is_checksum_sidecar("tool.tar.gz"));
    }
}
