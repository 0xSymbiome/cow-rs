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
- Typed `ValidTo` newtype in `cow-sdk-core` with absolute and relative-window
  constructors plus exported `VALID_TO_MIN_RELATIVE_SECONDS` and
  `VALID_TO_MAX_RELATIVE_SECONDS` constants. `LimitTradeParameters` exposes a
  `valid_to_typed` accessor that resolves absolute or relative inputs through
  the typed boundary so out-of-window deadlines fail closed with a typed
  `ValidationError::ValidToOutOfRange` at the client edge.
- Client-side 8 KB app-data size guard in `cow-sdk-app-data`. `get_app_data_info`
  and `get_app_data_info_legacy` now reject oversized stringified documents
  with a typed `AppDataError::TooLarge { actual_bytes, max_bytes }` before any
  network round trip, matching the orderbook's documented 8192-byte ceiling
  through the exported `APP_DATA_MAX_BYTES` constant.
- `Redacted<T>` newtype in `cow-sdk-core` with `Debug`, `Display`, and
  `Serialize` emitting the literal `[redacted]` placeholder and an
  `into_inner` escape for deliberate access. Secret-bearing configuration
  fields migrated to `Redacted<T>`: `ApiContext::api_key`,
  `ApiContextOverride::api_key`, `IpfsConfig::pinata_api_key`,
  `IpfsConfig::pinata_api_secret`, and the internal `SubgraphApi` API key.
- Shared `reqwest::Client` constructors on the orderbook and subgraph
  clients. `OrderBookApi::from_shared_client` plus its transport-policy
  variant and `SubgraphApi::from_shared_client` plus its static-config and
  transport-policy variants accept a pre-configured client so multi-chain
  consumers can pool one TCP, TLS, and HTTP/2 connection cache across every
  SDK instance they build. The default `new()` constructors stay unchanged
  and keep conservative upstream defaults.
- Opt-in quote-cache seam in `cow-sdk-trading`. The `QuoteCache` trait
  exposes async `lookup`, `insert`, and `invalidate` contract, with
  `NoopQuoteCache` shipped as the pass-through default and
  `InMemoryQuoteCache` shipped as a TTL-driven reference implementation.
  `TradingSdkBuilder::with_quote_cache` wires an `Arc<dyn QuoteCache>`
  onto the builder, so cache policy stays instance-scoped and caller-owned.
  The deterministic `QuoteCacheKey` derivation normalises address inputs so
  Redis-backed user implementations can share entries across processes.
- Byte-oriented constructors on the fixed-length typed newtypes. `Address`,
  `AppDataHash`, `Hash32`, and `OrderUid` expose `from_bytes` built on top
  of exported `const fn` hex encoders (`hex_encode_20`, `hex_encode_32`,
  `hex_encode_56`) and compile-time hex decoders
  (`hex_decode_20`, `hex_decode_32`, `hex_decode_56`). The embedded
  protocol-constant tables in `cow-sdk-core::config` now declare settlement,
  vault-relayer, ethflow, and wrapped-native addresses as `const [u8; 20]`
  byte arrays and construct the typed addresses through `from_bytes`,
  preserving the existing public accessor signatures and behaviour.
- Typestate `TradingSdkBuilder`. The builder carries two marker type
  parameters (`ChainIdUnset`/`ChainIdSet` and `AppCodeUnset`/`AppCodeSet`)
  that track whether the required chain id and app code prerequisites have
  been supplied through the explicit `with_chain_id` and `with_app_code`
  setters. The compile-time-checked `build_ready` terminal is only
  available once both markers reach the `Set` state, and `build_helper_only`
  is only available once the chain-id marker is `Set`. The permissive
  runtime-validated `build` and `build_partial` terminals remain on every
  state for the migration window. A `TradingSdkMode` enum (`Ready` or
  `HelperOnly`) plus the new `TradingError::HelperOnlyMode` variant fail
  quote, post, and off-chain cancellation flows closed when the sdk was
  constructed through the helper-only terminal, while chain-bound helpers
  (pre-sign transaction construction, allowance reads, approval submission,
  and on-chain cancellation) stay fully usable. A runnable
  `typestate_builder_example` demonstrates both terminals without requiring
  external credentials.
- Forward-compatible `#[non_exhaustive]` audit on the public configuration
  and context-override DTO families. `ApiContext`, `ProtocolOptions`, and
  `HttpClientPolicy` in `cow-sdk-core`, `ApiContextOverride` and
  `EnvBaseUrlOverrides` in `cow-sdk-orderbook`, and `DecimalAmount` in
  `cow-sdk-core` all carry `#[non_exhaustive]` at the struct head so the
  SDK can extend these surfaces with additive fields without a major
  semver bump. Ergonomic constructors (`ApiContext::new` plus
  `with_base_urls`/`with_api_key`, `ProtocolOptions::new` plus
  `with_env`/`with_settlement_contract_override`/`with_eth_flow_contract_override`,
  `ApiContextOverride::new` plus `with_chain_id`/`with_env`/`with_base_urls`/`with_api_key`)
  replace struct-literal construction for downstream callers.

### Security

- Defense-in-depth redaction in transport error paths. `From<reqwest::Error>`
  on the orderbook and subgraph error surfaces now calls
  `reqwest::Error::without_url` and classifies failures through the
  documented `is_timeout`, `is_connect`, `is_redirect`, `is_decode`,
  `is_body`, `is_builder`, `is_request`, and `is_status` set, so partner
  routes and their query-string API keys cannot leak through error
  `Display` output. The `Redacted<T>` newtype and the config-layer
  migrations above keep the configuration surface redacted even before a
  request is built.

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
