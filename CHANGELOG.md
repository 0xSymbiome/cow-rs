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
- Broadened `#[non_exhaustive]` coverage across every public DTO family in
  the trading-first surface so future additive fields no longer require a
  major version bump. `cow-sdk-orderbook` now annotates `OrderCreation`,
  `OrderQuoteRequest`, `OrderQuoteResponse`, the wire `Order` and `Trade`
  DTOs, `EthflowData`, `QuoteSide`, `QuoteData`, `GetOrdersRequest`,
  `GetTradesRequest`, `OrderCancellations`, `NativePriceResponse`,
  `TotalSurplus`, `AppDataObject`, `CompetitionOrderStatus`,
  `CompetitionAuction`, `SolverCompetitionResponse`, `SolverSettlement`,
  `SolverExecution`, and `Auction`. `cow-sdk-trading` annotates
  `TradeParameters`, `LimitTradeParameters`, `TraderParameters`,
  `PartialTraderParameters`, `OrderTraderParameters`, `QuoterParameters`,
  `QuoteResults`, `QuoteRequestOverride`, `OrderPostingResult`,
  `SwapAdvancedSettings`, `LimitOrderAdvancedSettings`,
  `PostTradeAdditionalParams`, `TradingAppDataInfo`, `OrderToSignParams`,
  `AllowanceParameters`, `ApprovalParameters`, `OrderbookRuntimeBinding`,
  `SlippageToleranceRequest`, `SlippageToleranceResponse`, and
  `EthFlowTransaction`. `cow-sdk-subgraph` annotates `TotalsResponse`,
  `DailyTotal`, `HourlyTotal`, `LastDaysVolumeResponse`,
  `LastHoursVolumeResponse`, `Total`, and `SubgraphQueryRequest`.
  `cow-sdk-core` also annotates `QuoteAmountsAndCosts`. Every annotated
  struct ships an ergonomic constructor (`::new(required_args)` plus
  chainable `with_*` setters for the optional fields) so downstream
  callers migrate off struct-literal syntax without boilerplate.
- Opt-in `tracing` feature family across the public crate graph. Every
  published leaf crate now exposes a `tracing` feature that pulls
  `tracing = { version = "0.1", default-features = false, features = ["attributes"] }`
  as an optional dependency, and the facade `cow-sdk/tracing` feature
  activates the leaves in one step. Every long-running public
  operation on `OrderBookApi`, `SubgraphApi`, and `TradingSdk`, the
  canonical local signing entry points on `cow-sdk-signing`, and the
  wallet-mediated chain operations on `BrowserWallet` are annotated
  with `#[cfg_attr(feature = "tracing", tracing::instrument(...))]`
  using the documented field registry (`chain`, `env`, `endpoint`,
  `method`, `scheme`, `order_uid`, and related safe identifiers) so
  host applications can route structured spans into their own
  subscriber. With the feature off the SDK emits zero spans and
  incurs no dependency or runtime cost.
- `SdkError::class() -> ErrorClass` classification helper on the facade
  aggregate. Every variant of the facade error family resolves to one of
  `Validation`, `Transport`, `Remote`, `Signing`, `Cancelled`, or
  `Internal` so downstream telemetry layers can partition failures
  without pattern-matching every nested variant by hand. The new
  `docs/observability.md` page ships the complete structured-field
  registry, baseline `tracing-subscriber` setup, OpenTelemetry notes, and
  an explicit reminder that secret-bearing fields are never emitted
  through SDK spans.
- Cooperative cancellation on long-running SDK operations via the
  `cow_sdk_core::Cancellable::cancel_with(&token)` extension-trait
  combinator. `cow-sdk-core` defines the `Cancellable` trait, the
  `WithCancellation<'t, F>` future wrapper, and the `Cancelled`
  marker error; the `cow-sdk` prelude re-exports `Cancellable` and
  `Cancelled` so `use cow_sdk::prelude::*` reaches the combinator.
  `cow-sdk-core` also re-exports
  `tokio_util::sync::CancellationToken` as
  `cow_sdk_core::CancellationToken` so every public crate routes
  cancellation through a single typed import. Every public
  long-running async method on `OrderBookApi`, `SubgraphApi`, and
  `TradingSdk` composes with `.cancel_with(&token)` at the call
  site; the combinator's `Future::poll` performs a biased check
  against `token.is_cancelled()` before polling the inner future,
  so cancellation is observed before the next `.await` and the
  in-flight request future is dropped promptly rather than waiting
  for the request deadline. `CoreError`, `OrderbookError`,
  `TradingError`, `SubgraphError`, `SigningError`, and
  `BrowserWalletError` each carry a typed `Cancelled` variant and
  implement `From<cow_sdk_core::Cancelled>` so the combinator yields
  the crate-level error directly; `SdkError::class()` routes every
  such variant to `ErrorClass::Cancelled` exhaustively.
  `docs/architecture.md` records the cancellation contract under a
  dedicated Cancellation subsection.

### Fixed

- Example crates now construct every `#[non_exhaustive]` public DTO
  through the published ergonomic constructors (`::new(required_args)`
  plus chained `with_*` setters) rather than struct-literal syntax, so
  the `examples/native`, `examples/wasm/sdk-verification-console`, and
  `examples/wasm/browser-wallet-console` build surfaces stay green
  under the broadened `#[non_exhaustive]` coverage shipped in the same
  `[Unreleased]` cycle. The `cow-sdk-core::cancellation` rustdoc also
  corrects a spelling drift in the `Cancelled` marker's documentation.
- Example-crate browser-hosted tests now align with the current public
  contract. The `sdk-verification-console` deterministic-export suite
  compares wrapped-native and sample-order addresses through a
  case-insensitive helper so the byte-array-sourced lowercase hex output
  no longer breaks the assertion, and the EIP-1271 payload preview
  assertion now matches the `0x`-prefixed hex shape that
  `eip1271_signature_payload` actually returns. The
  `browser-wallet-console` test-only helper surface is also reachable
  under `wasm32-unknown-unknown`, so headless wasm-pack runs exercise
  the injected-wallet, session, and cached-detection paths alongside
  their native counterparts. The helpers remain marked `#[doc(hidden)]`
  and stay excluded from the public API surface.

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
- Generator-backed property tests on the four deterministic-codec crates.
  `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`, and `cow-sdk-app-data`
  now run their `property_contract.rs` suites through `proptest = "1.11"` with
  shrinking and committed `proptest-regressions/` seed files so shrink outcomes
  stay reproducible across contributors. Every invariant family the previous
  deterministic enumerator exercised is preserved as a named `proptest!` case
  with a plain-English doc comment.
- Fixture-driven `parity_contract.rs` regressions on `cow-sdk-signing`,
  `cow-sdk-orderbook`, `cow-sdk-app-data`, `cow-sdk-subgraph`, and
  `cow-sdk-contracts`. Each regression loads `parity/fixtures/<surface>.json`
  at compile time, validates the schema version and surface label, iterates
  every documented case with an id-keyed dispatcher that fails closed on
  unknown ids, invokes the covered Rust helper, and carries the fixture
  case id into every assertion message so a broken CI run names the upstream
  vector that diverged.
- A field-level trading parity round-trip on
  `crates/trading/tests/parity_contract.rs`. The regression reconstructs typed
  Rust inputs from every case in `parity/fixtures/trading.json` and drives
  the shipped helpers — `post_swap_order`, `post_limit_order`,
  `post_sell_native_currency_order`, `get_quote_results`, `get_quote_only`,
  `get_order_to_sign`, `get_pre_sign_transaction`, `get_eth_flow_transaction`,
  `onchain_cancellation_transaction`, `build_app_data`, `merge_app_data_doc`,
  `suggest_slippage_bps`, `TradingSdk`, and `protocol_options_for_order` —
  with per-field `assert_eq!` messages that name the fixture case id and
  the diverging field at once.
- A cargo-fuzz harness under a standalone `fuzz/` crate that pins
  `libfuzzer-sys` to an exact version and carries five fuzz targets
  covering the deterministic codec boundaries: `fuzz_order_uid_pack_unpack`
  asserts the pack-and-extract round-trip for `OrderUid` components;
  `fuzz_typed_data_digest` asserts `hash_order` stays deterministic under
  Arbitrary-derived `UnsignedOrder` shapes and exercises
  `hash_order_cancellations`; `fuzz_app_data_cid_roundtrip` asserts
  `cid_to_app_data_hex(app_data_hex_to_cid(x)?)? == x` on both the
  keccak-256 and sha2-256 multihash paths and typed-error-not-panic on
  malformed input; `fuzz_order_signature_classify` confirms
  `SigningScheme::try_from(u8)` remains total across all 256
  discriminants and feeds arbitrary bytes through
  `decode_eip1271_signature_data` and the `Signature` decoder;
  `fuzz_subgraph_graphql_error_decode` feeds arbitrary JSON candidates to
  the `SubgraphGraphQlError` decoder and asserts successful decodes
  round-trip through `serde_json::to_vec` without panicking. The
  fuzz crate sits outside the root workspace members list so the stable
  toolchain is never forced onto nightly; `fuzz/README.md` documents the
  shared-harness conventions, supported-platform boundary, and the
  reproduce-from-corpus workflow.
- A scheduled weekly `.github/workflows/fuzz.yml` report-only lane
  (Friday 05:00 UTC plus `workflow_dispatch`) that matrix-runs each of
  the five fuzz targets on `ubuntu-latest` for five minutes under a
  sixty-minute job timeout. The workflow uses SHA-pinned third-party
  actions with `# Source ref:` comments, `permissions: contents: read`,
  a `concurrency` group scoped to workflow and ref with
  `cancel-in-progress: false`, the pinned nightly toolchain exposed
  through the `RUST_FUZZ_TOOLCHAIN` env variable, and uploads
  `fuzz/corpus/<target>/` and `fuzz/artifacts/<target>/` as a
  `fuzz-<target>-corpus-and-artifacts` workflow artifact on
  `if: failure()`. `CONTRIBUTING.md` gained a "Running Fuzz Targets
  Locally" section covering the nightly prerequisite,
  `cargo install cargo-fuzz --locked`, `cargo fuzz list`, the per-target
  one-minute local-run command, and the reproduce-from-corpus
  invocation.
- A Firefox Playwright project on `e2e/browser-wallet/` that runs
  alongside the existing Chromium project, so the browser-wallet
  deterministic-lane DOM contract is validated under both widely
  deployed browser engines. `e2e/browser-wallet/playwright.config.ts`
  declares the additional project with `{ ...devices["Desktop Firefox"] }`
  and inherits every root-level setting (`baseURL`, `viewport`, `trace`,
  `webServer`, `fullyParallel`, `forbidOnly`, `retries`, `timeout`,
  `expect.timeout`, `reporter`). `.github/workflows/browser-wallet-e2e.yml`
  installs both browsers via
  `bunx playwright install --with-deps chromium firefox` while keeping
  the existing SHA-pinned actions, `permissions: contents: read`,
  `concurrency`, `persist-credentials: false`, and `timeout-minutes: 45`
  hygiene intact. The EIP-6963 `announceProvider` fixture at
  `e2e/browser-wallet/fixtures/injected-wallet.ts` is unchanged; the
  fixture uses standard DOM surfaces that resolve identically under
  Chromium and Firefox. `docs/browser-runtime-proof-posture.md`
  acknowledges the two-browser deterministic matrix under the existing
  Deterministic Lane. ADR 0007 is unchanged.

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
- ADR 0010 records the runtime-neutral async and transport posture covering
  cooperative cancellation, the shared `reqwest::Client` pattern, the
  url-stripped reqwest error classification, and the opt-in `tracing` feature.
- ADR 0011 records the typed amount boundary and the typestate ready-state
  construction rule for `TradingSdkBuilder`.
- The Cooperative Cancellation Contract Audit is a standing audit covering
  the shared `CancellationToken` re-export, the
  `cow_sdk_core::Cancellable::cancel_with(&token)` extension-trait combinator
  as the canonical public composition path for every long-running public
  operation on `OrderBookApi`, `SubgraphApi`, and `TradingSdk`, the typed
  `Cancelled` variants on each crate-level error enum, the `From<Cancelled>`
  bridges, and the biased `tokio::select!` semantics that the combinator
  delivers inside its `Future::poll`.
- The Credential Surface Contract Hygiene Audit is refreshed to cover the
  `Redacted<T>` wrapper and the transport-level error redaction path.
- `docs/release-checklist.md` now describes the functional `0.1.0` crates.io
  release publish sequence in finished-product language, naming the nine
  `first-release` crates the sequence publishes in dependency order.

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
