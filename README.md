# cargo-attest

> Verify that a published Rust binary is backed by the source it claims, without rebuilding it.

`cargo-attest` is a Cargo subcommand for consumer-side supply-chain checks. Given a GitHub release or a future crates.io artifact, it gathers the available provenance signals, hashes the downloaded artifact, and returns an actionable verdict:

- `TRUSTED`: at least one load-bearing proof matched.
- `UNVERIFIED`: no proof was broken, but there was not enough evidence.
- `MISMATCH`: a declared proof was present and did not match.
- `ERROR`: the run itself failed.

The goal is not to replace reproducible builds. The goal is to give users, package maintainers, and security teams a fast way to answer: "Does this published binary line up with the source it points to?"

## Status

`cargo-attest` is early, but the GitHub Release path already works.

Current support:

- resolve a GitHub release tag to a commit;
- select release assets with `--asset`;
- download the asset locally;
- verify downloaded byte size against GitHub metadata;
- verify SHA-256 from the release body or a checksum sidecar;
- emit human output or JSON;
- return stable exit codes for CI.

Planned next:

- GitHub artifact attestations and workflow provenance;
- crates.io metadata and `.crate` verification;
- sigstore/cosign verification.

See [ROADMAP.md](ROADMAP.md) for the working roadmap.

## Installation

Not published to crates.io yet.

From a local checkout:

```bash
cargo install --path .
```

MSRV: Rust 1.86.

## Usage

Hash a local file:

```bash
cargo attest hash ./my-binary
```

Verify a GitHub release asset:

```bash
cargo attest release BurntSushi/ripgrep 14.1.1 \
  --asset ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz
```

Example output:

```text
TRUSTED
  ✓ tag-resolves-to-commit — tag 14.1.1 -> 4649aa970061
  ✓ asset-size:ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz — downloaded 2566310 bytes, matching GitHub metadata
  ✓ sha256-body-checksum:ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz — declared via sidecar asset ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz.sha256 and computed SHA-256 match
```

Machine-readable output:

```bash
CARGO_ATTEST_JSON=1 cargo attest release BurntSushi/ripgrep 14.1.1 \
  --asset ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz
```

## Checksum Discovery

For v0.2, `cargo-attest` accepts common SHA-256 publication patterns:

- a 64-character hex token on the same line as the asset name in the release body;
- a sidecar asset named `<asset>.sha256`;
- a sidecar asset named `<asset>.sha256sum`;
- a sidecar asset named `<asset>.sha256.txt`.

If a checksum is found and matches the downloaded artifact, the verdict is `TRUSTED`. If no usable checksum is found, the verdict is `UNVERIFIED`. If a checksum is found but does not match, the verdict is `MISMATCH`.

## Exit Codes

| Code | Verdict |
| ---: | --- |
| 0 | `TRUSTED` |
| 1 | `UNVERIFIED` |
| 2 | `MISMATCH` |
| 3 | `ERROR` |

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

The real-network GitHub release test is ignored by default:

```bash
cargo test --test release_real -- --ignored
```

## Security Model

`cargo-attest` only consumes public release metadata and artifacts. It does not execute downloaded artifacts. It currently verifies published hashes and basic GitHub release metadata; it does not yet verify GitHub artifact attestations, SLSA provenance, or sigstore/cosign signatures.

Treat `TRUSTED` as "the implemented checks passed", not as a complete supply-chain guarantee.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
