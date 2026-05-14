# Roadmap

`cargo-attest` is an early-stage Cargo subcommand for consumer-side verification of published Rust artifacts.

This roadmap is intentionally public and high-level. It describes the direction of the project for users and contributors; implementation details may change as the tool matures.

## Current Status

The GitHub Release verification path is usable today:

- resolve a release tag to a commit;
- select a release asset with `--asset`;
- download the selected artifact;
- verify downloaded size against GitHub metadata;
- verify SHA-256 from the release body or a checksum sidecar;
- look up GitHub artifact attestation bundles for the computed SHA-256;
- emit a human verdict or JSON;
- return stable CI-friendly exit codes.

## Verdicts

| Verdict | Meaning |
| --- | --- |
| `TRUSTED` | At least one load-bearing proof matched. |
| `UNVERIFIED` | No proof was broken, but there was not enough evidence to trust the artifact. |
| `MISMATCH` | A declared proof was present and did not match. |
| `ERROR` | The command could not complete. |

## Scope

### In Scope

- GitHub Release artifact verification.
- crates.io artifact verification.
- Published checksums.
- GitHub artifact attestations and SLSA provenance.
- sigstore/cosign signatures.
- Human-readable output and JSON output for CI.

### Out of Scope

- Bit-for-bit rebuilds.
- Vulnerability scanning.
- Code review audits.
- Producer-side signature issuance.

## Milestones

### v0.1 - Walking Skeleton

- [x] Cargo subcommand scaffold.
- [x] `cargo attest hash <file>`.
- [x] Core verdict model.
- [x] GitHub Actions CI.

### v0.2 - GitHub Release Checksums

- [x] `cargo attest release <repo> <tag>`.
- [x] GitHub release metadata lookup.
- [x] Tag-to-commit resolution.
- [x] Asset download.
- [x] SHA-256 verification from release body checksums.
- [x] SHA-256 verification from `.sha256`, `.sha256sum`, and `.sha256.txt` sidecars.
- [x] JSON output through `CARGO_ATTEST_JSON=1`.
- [x] Stable exit codes.
- [x] Real-network integration test against a public release, ignored by default.

### v0.3 - GitHub Artifact Provenance

- [x] Look up GitHub artifact attestation bundles by artifact SHA-256.
- [ ] Cryptographically verify GitHub artifact attestation bundles.
- [ ] Verify the chain `artifact -> workflow run -> commit`.
- [ ] Include provenance checks in human and JSON verdicts.

### v0.4 - crates.io

- [ ] Download `.crate` files from crates.io.
- [ ] Read crates.io metadata.
- [ ] Compare crate metadata with the declared repository.
- [ ] Check source tags where a repository is declared.

### v0.5 - sigstore / cosign

- [ ] Verify detached cosign signatures.
- [ ] Query Rekor when useful.
- [ ] Decide whether to use a Rust-native library, shell out to `cosign`, or support both.

### v1.0 - Stabilization

- [ ] Stable CLI and JSON schema.
- [ ] Local cache for downloaded artifacts and API responses.
- [ ] Broader integration tests.
- [ ] Complete documentation.

## Open Questions

- Should discovered but unverified GitHub artifact attestations ever affect `TRUSTED`, or only fully verified bundles?
- Should the JSON schema be versioned before crates.io support lands?
- What is the right default cache location and eviction policy?
- Should `cargo-attest` stay a CLI-only tool, or eventually expose a library API?
