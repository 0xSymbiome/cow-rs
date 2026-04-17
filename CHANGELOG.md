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
- Typed decimal-aware amount boundary in `cow-sdk-core`. `AtomAmount` wraps an
  unsigned 256-bit atomic quantity and keeps the canonical base-10 string on
  the wire, while `DecimalAmount` pairs an atomic value with a decimals scale
  for display and user-input flows. Typed accessor helpers on
  `cow-sdk-trading::TradeParameters` and
  `cow-sdk-trading::LimitTradeParameters` surface the new types at the
  trading boundary without changing the existing wire-compatible
  `Amount`-based signatures. Existing `Amount`-backed surfaces remain
  supported; new typed code should prefer `AtomAmount` and `DecimalAmount`.
- Zero-copy settlement call-data representation. Settlement, interaction, and
  swap encoder outputs now hold their payload as `bytes::Bytes` so fanning
  the same encoded call data across multiple settlement candidates shares a
  single backing allocation through reference-counted clones. The public
  JSON wire form remains the canonical `0x`-prefixed hex string.
- Criterion benchmark suites for order hashing, UID pack and extract, signing
  typed-data envelope construction, deterministic app-data stringification,
  orderbook quote-fee aggregation, and trading limit-order construction, plus
  a non-blocking weekly benchmarks workflow that archives the Criterion HTML
  and JSON reports.
- Public performance posture note under `docs/performance.md` mapping the
  benchmarked hot paths and their reported ranges.

### Changed

- `cow_sdk_core::Address` compares and hashes case-insensitively through the
  lowercase normalized key while `Address::as_str` preserves the original
  input casing. `addresses_equal` now dispatches to the same case-insensitive
  comparison as the `PartialEq` and `Hash` implementations.
- New `as_bytes` and `byte_length` accessors on `Address`, `AppDataHash`,
  `Hash32`, `OrderUid`, and `HexData` expose the stored hex representation as
  a byte slice and report the decoded byte length without allocation.
- Hot cross-crate helpers (`addresses_equal`, `token_id`,
  `pack_order_uid_params`, `extract_order_uid_params`, `hash_order_for_contract`,
  `compute_order_uid`, and the small `SigningScheme` and trade-flag codecs)
  carry `#[inline]` annotations so equality checks and contract encoding on
  the fast path incur no avoidable work.

### Changed

- The public documentation graph now routes first-touch users through one
  canonical getting-started path before branching into the maintained example
  families.
- The root landing page and docs hub now expose explicit trust and maintenance
  signals, including the current publication state, security disclosure path,
  and release-readiness references.
- Public error enums and the documented growth-state enums now use
  `#[non_exhaustive]` so additive variants remain compatible with the shipped
  `0.1.0` surface.
- `CoreError` is now the single canonical shared core error name across the
  public facade and guides; the unused `CowRsError` alias has been removed as a
  naming finalization before the first functional release.

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
