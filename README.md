<div align="center">

# cargo-attest

Verify published Rust binaries against the source they claim, without rebuilding them.

<p>
  <a href="https://github.com/Nil06/cargo-attest/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/Nil06/cargo-attest/actions/workflows/ci.yml/badge.svg"></a>
  <img alt="MSRV" src="https://img.shields.io/badge/MSRV-1.86-blue">
  <img alt="License" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green">
  <img alt="Status" src="https://img.shields.io/badge/status-early%20alpha-orange">
</p>

</div>

`cargo-attest` is a Cargo subcommand for consumer-side supply-chain checks. It downloads a published artifact, gathers the evidence that links it to source, and returns a verdict you can use in a terminal or CI job.

It is built for a simple question:

> Does this binary line up with the source and release metadata it points to?

## Why

Rust has strong source-level tooling, but many users consume compiled artifacts from GitHub Releases, package managers, Docker images, and CI pipelines. Rebuilding every binary bit-for-bit is valuable, but it is expensive and rare in everyday workflows.

`cargo-attest` takes a pragmatic path: verify the evidence publishers already provide, then make the result explicit.

| Verdict | Meaning |
| --- | --- |
| `TRUSTED` | At least one load-bearing proof matched. |
| `UNVERIFIED` | Nothing was proven wrong, but evidence was missing. |
| `MISMATCH` | Published evidence exists and does not match the artifact. |
| `ERROR` | The command could not complete. |

## What Works Today

GitHub Release verification is implemented:

- resolves the release tag to a commit;
- selects assets with `--asset`;
- downloads the artifact locally;
- checks downloaded size against GitHub metadata;
- verifies SHA-256 from the release body or checksum sidecars;
- looks up GitHub artifact attestation bundles for the computed SHA-256;
- prints human output or JSON;
- exits with stable CI-friendly codes.

Planned next:

- cryptographic verification of GitHub artifact attestation bundles;
- workflow provenance checks;
- crates.io `.crate` verification;
- sigstore/cosign verification.

## Quick Start

Not published to crates.io yet. From a checkout:

```bash
cargo install --path .
```

Hash a local file:

```bash
cargo attest hash ./my-binary
```

Verify a real GitHub Release asset:

```bash
cargo attest release BurntSushi/ripgrep 14.1.1 \
  --asset ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz
```

Example output:

```text
TRUSTED
  ✓ tag-resolves-to-commit — tag 14.1.1 → 4649aa970061
  ✓ asset-size:ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz — downloaded 2566310 bytes, matching GitHub metadata
  ✓ sha256-body-checksum:ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz — declared via sidecar asset ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz.sha256 and computed SHA-256 match (4cf9f2741e6c465f)
  · github-artifact-attestation:ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz — lookup unavailable: GitHub attestation API denied the request (401 Unauthorized); set GH_TOKEN or GITHUB_TOKEN to retry
```

## JSON Output

Set `CARGO_ATTEST_JSON=1`:

```bash
CARGO_ATTEST_JSON=1 cargo attest release BurntSushi/ripgrep 14.1.1 \
  --asset ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz
```

Exit codes:

| Code | Verdict |
| ---: | --- |
| 0 | `TRUSTED` |
| 1 | `UNVERIFIED` |
| 2 | `MISMATCH` |
| 3 | `ERROR` |

## Checksum Discovery

For v0.2, `cargo-attest` accepts common SHA-256 publication patterns:

- a 64-character hex token on the same line as the asset name in the release body;
- a sidecar asset named `<asset>.sha256`;
- a sidecar asset named `<asset>.sha256sum`;
- a sidecar asset named `<asset>.sha256.txt`.

If no usable checksum is found, the artifact is `UNVERIFIED`, not `TRUSTED`.

## Security Model

`cargo-attest` does not execute downloaded artifacts. It downloads bytes, hashes them, and compares them to public release evidence.

Current checks are intentionally limited:

- GitHub release metadata;
- release tag resolution;
- GitHub asset size;
- SHA-256 checksums.
- GitHub artifact attestation bundle lookup.

It does not yet cryptographically verify GitHub artifact attestation bundles, SLSA provenance, or sigstore/cosign signatures. Treat `TRUSTED` as "the implemented checks passed", not as a complete supply-chain guarantee.

GitHub attestation lookup uses public API access when available. If GitHub denies an unauthenticated request, set `GH_TOKEN` or `GITHUB_TOKEN` and rerun the command.

## Development

MSRV: Rust 1.86.

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Run the real-network GitHub release test:

```bash
cargo test --test release_real -- --ignored
```

## Roadmap

See [ROADMAP.md](ROADMAP.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
