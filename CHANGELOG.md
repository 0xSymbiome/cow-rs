# Changelog

All notable changes to `cow-rs` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Semantic versioning begins with the first functional crate release.

Reserved-placeholder `0.0.1-reserved.0` name-reservation publishes are
excluded from this version history.

Until that first functional publication is live, this file tracks the current
unreleased public contract of the repository.

## [Unreleased]

### Added

- A trading-first Rust SDK workspace covering `cow-sdk`, `cow-sdk-core`,
  `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data`,
  `cow-sdk-orderbook`, `cow-sdk-trading`, `cow-sdk-subgraph`, and
  `cow-sdk-browser-wallet`.
- Deterministic native example scenarios plus browser-hosted WASM verification
  surfaces for the supported SDK and browser-wallet flows.
- Public verification, parity, architecture, ADR, and audit documentation for
  the current Rust SDK surface.

### Changed

- The public documentation graph now routes first-touch users through one
  canonical getting-started path before branching into the maintained example
  families.
- The root landing page and docs hub now expose explicit trust and maintenance
  signals, including the current publication state, security disclosure path,
  and release-readiness references.

### Security

- Repository security reporting now has an explicit private disclosure path and
  a protocol-level escalation note for issues that could affect deployed CoW
  Protocol infrastructure or user funds.

### Notes

- `0.1.0` will be recorded here when the first functional crates.io release is
  live.

## [0.1.0] - TBD

Placeholder for the first functional crates.io release of the `cow-rs` crate
family. This section will be populated when that release is published.
