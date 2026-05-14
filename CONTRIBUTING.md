# Contributing

Thanks for helping improve `cargo-attest`.

## Development Setup

Install Rust 1.86 or newer, then run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

The real-network integration test is intentionally ignored by default:

```bash
cargo test --test release_real -- --ignored
```

## Project Direction

The roadmap lives in [SCHEMA_DIRECTEUR.md](SCHEMA_DIRECTEUR.md). Before making a larger change, check that it fits the current scope:

- consumer-side verification;
- no bit-for-bit rebuilds;
- no vulnerability scanning;
- no producer-side signature issuance.

## Pull Requests

Small, focused PRs are preferred. Include tests when changing behavior, and update the README or roadmap when user-facing behavior changes.

For now, avoid adding heavyweight dependencies unless they unlock a concrete roadmap item.
