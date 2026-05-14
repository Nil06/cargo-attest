# Security Policy

`cargo-attest` is a supply-chain verification tool, so security reports matter even while the project is early.

## Supported Versions

The project is pre-1.0. Security fixes will target the `main` branch until versioned releases begin.

## Reporting a Vulnerability

Please do not open a public issue for a vulnerability.

Report security issues through GitHub private vulnerability reporting if it is enabled on the repository. If it is not enabled yet, contact the maintainer directly through the GitHub profile listed on the repository.

Useful details include:

- affected command and version or commit;
- exact input release/crate if public;
- expected verdict vs actual verdict;
- whether the issue can cause a false `TRUSTED`, hide a `MISMATCH`, or execute untrusted content.

## Current Guarantees

`cargo-attest` does not execute downloaded release assets. It downloads artifacts, hashes them, and compares metadata/proofs.

Current checks are limited to GitHub release metadata, asset size, and SHA-256 checksums from release bodies or sidecar files. GitHub artifact attestations, SLSA provenance, and sigstore/cosign signatures are planned but not implemented yet.
