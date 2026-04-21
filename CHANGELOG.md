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
- Optional caching seam for EIP-1271 signature verification.
  `cow_sdk_signing::Eip1271VerificationCache` is a narrow `Send + Sync`
  trait keyed by `(verifier, digest)` that
  `cow_sdk_contracts::verify_eip1271_signature_async` consults before
  any on-chain `isValidSignature` call. Two default implementations
  ship from the signing crate: the zero-sized
  `NoopEip1271VerificationCache` for callers that do not want caching
  and `InMemoryEip1271VerificationCache`, a TTL-respecting,
  capacity-bounded in-memory store backed by `parking_lot::RwLock`
  (default five-minute TTL, default 1024-entry capacity, oldest-first
  eviction). The cache stores `bool` outcomes — `true` for a
  successful magic-value match and `false` for the typed
  `Eip1271MagicValueMismatch` — and never caches transport, missing
  contract code, serialization, or hex-decode failures so transient
  errors always re-hit the chain. `Eip1271VerificationCache`,
  `NoopEip1271VerificationCache`, and `InMemoryEip1271VerificationCache`
  re-export through the `cow-sdk` facade, and the trait surfaces
  through `cow_sdk::prelude::*` for compositions that hold the cache
  generically.
- New browser-side transport leaf crate `cow-sdk-transport-wasm`.
  The crate ships `FetchTransport`, a `wasm32`-only implementation of
  the shared `cow_sdk_core::HttpTransport` trait backed by
  `web-sys::fetch` and `wasm-bindgen-futures`. Every browser-fetch
  failure is classified through the same `TransportErrorClass`
  taxonomy the native `ReqwestTransport` default uses (`Timeout`,
  `Connect`, `Redirect`, `Decode`, `Body`, `Status`, fallthrough), an
  `AbortController` is wired into the in-flight request when a
  configured timeout elapses so the resulting `AbortError` maps to
  `TransportErrorClass::Timeout`, and the base URL rides the existing
  `Redacted` newtype so it never appears in debug, display, or
  serialized output. A parity integration test drives the native
  `ReqwestTransport` baseline and the `FetchTransport` harness against
  shared fixtures so both adapters deliver byte-identical response
  bodies. The crate compiles to an empty unit on non-`wasm32` targets,
  and is intentionally not re-exported through the `cow-sdk` facade:
  consumers compose `Arc<dyn HttpTransport>` values directly from
  their browser-side code without pulling the crate into native
  transitive graphs.
- Chain-keyed registry of canonical CoW Protocol contract deployments
  under `cow_sdk_contracts::deployments`. The new `Registry` type
  resolves deployed addresses through the typed
  `(ContractId, SupportedChainId, CowEnv)` key triple and ships with an
  embedded manifest at `crates/contracts/registry.toml` seeded for the
  GPv2 settlement, vault-relayer, and EthFlow contracts across every
  supported chain in both the production and staging environments. A
  compile-time `build.rs` validator rejects malformed manifests at build
  time with a precise diagnostic that names the offending row, and the
  runtime `Registry::from_toml_str` loader surfaces the same taxonomy of
  failures through a typed `RegistryError` enum so downstream consumers
  who pipe their own TOML into the loader see the same actionable
  errors. `Registry`, `ContractId`, and `RegistryError` are re-exported
  from the facade, and `Registry` plus `ContractId` surface through
  `cow_sdk::prelude::*` so the typed address lookup is a single import
  away for trading and bridging consumers.
- Typed ERC-20 and EIP-2612 Permit bindings under a new
  `cow_sdk_contracts::erc20` module, generated from the canonical Solidity
  surfaces through the `alloy::sol!` macro. The module exposes the minimal
  `IERC20` interface (`balanceOf`, `approve`, `allowance`, `transfer`,
  `transferFrom` plus the `Transfer` and `Approval` events) and the
  `IERC20Permit` extension covering the EIP-2612 `permit(...)` method, the
  `DOMAIN_SEPARATOR()` and `nonces(...)` view functions, and the `Permit`
  struct used to derive the typed-data hash. A pinned `PERMIT_TYPE_HASH`
  constant and a `permit_typed_data_hash(domain, permit)` helper compose the
  EIP-712 envelope so off-chain signers produce a digest every EIP-2612
  deployment will accept, and the committed Solidity excerpts under
  `crates/contracts/abi/erc20/` preserve upstream provenance for reviewers.
- Deterministic native example scenarios plus browser-hosted WASM verification
  surfaces for the supported SDK and browser-wallet flows.
- Public verification, parity, architecture, ADR, and audit documentation for
  the current Rust SDK surface.
- Typed decimal-aware amount boundary in `cow-sdk-core`. `Amount` wraps an
  unsigned 256-bit atomic quantity as a typed `BigUint` and keeps the
  canonical base-10 string on the wire, while `DecimalAmount` pairs an
  atomic value with a decimals scale for display and user-input flows. The
  typed amount surface is the single canonical shape on the
  `cow-sdk-trading` request boundary, with `From<BigUint>` and
  `TryFrom<&str>` conversions for atomic interop.
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
- Shared `reqwest::Client` pooling for multi-chain consumers.
  `OrderBookApi::builder()` and `SubgraphApi::builder()` both expose a
  convenience `.client(shared)` setter on native targets that installs a
  `cow_sdk_core::ReqwestTransport` wrapping the supplied client, so one
  warm TCP, TLS, and HTTP/2 connection cache backs every chain and
  service the consumer routes through. Custom transport implementations
  install through the analogous `.transport(Arc::new(...))` setter.
  Default construction on native targets installs a conservative
  `ReqwestTransport` automatically.
- Stability invariant across the published `cow-sdk` crate family.
  `cow-sdk`, `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`,
  `cow-sdk-app-data`, `cow-sdk-orderbook`, `cow-sdk-trading`,
  `cow-sdk-subgraph`, and `cow-sdk-browser-wallet` exclude
  `alloy-provider` from their transitive dependency graph by
  construction. Consumers select their own chain-RPC runtime through
  the `cow_sdk_core::AsyncProvider` seam and its documented alloy
  adapter guide, and a CI workflow step runs
  `cargo tree --invert alloy-provider -p cow-sdk-core -p cow-sdk-contracts -p cow-sdk-signing -p cow-sdk-orderbook -p cow-sdk-subgraph -p cow-sdk-app-data -p cow-sdk-trading -p cow-sdk`
  on every pull request, asserting empty output before the change can
  land.
- Canonical `alloy::sol!`-generated typed bindings for every contract
  surface the SDK emits call-data against: `GPv2Settlement` settlement
  plus pre-signature and invalidation, `GPv2VaultRelayer` authorization
  checks, `CoWSwapEthFlow` order creation and invalidation, the EIP-1967
  storage-slot and proxy ownership surface, and the `IERC20` plus
  `IERC20Permit` (EIP-2612) ERC-20 surface. Every binding is generated
  from Solidity excerpts committed under `crates/contracts/abi/**/*.sol`
  and gated by a byte-identity parity regression against fixtures
  derived from the upstream TypeScript SDK, so the encoded call-data
  output is always sourced from the upstream CoW Protocol Solidity
  surface rather than a parallel Rust reimplementation.
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
- Playwright deterministic-lane suites for both example consoles now
  track the current SDK contract end-to-end. Quote, order, and
  order-trades assertions compare addresses through a case-insensitive
  helper; the order-trades fixture routes the current `/api/v2/trades`
  endpoint; the solver-competition assertions describe the reviewed
  `SolverSettlement` contract (ranking, solver address, score, and
  clearing-prices map) rather than fields that are not part of the
  typed boundary; the orderbook network-failure assertion matches the
  classified `reqwest` error text; and the chain-mismatch fail-closed
  contract is verified by asserting the disabled `#sign-order` button
  and its chain-mismatch title rather than attempting to click a button
  the console deliberately disables. The `browser-wallet-console`
  diagnostic labeller also classifies EIP-1193 provider codes that
  arrive through the `Display`-formatted Rust error shape
  (`… rejected by the user (4001): …`) in addition to the JSON
  `"code": 4001` shape, so rejected typed-data signing now renders the
  `EIP-1193 4001` label consistently across Chromium and Firefox.

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
- Repository security reporting now has an explicit private disclosure path and
  a protocol-level escalation note for issues that could affect deployed CoW
  Protocol infrastructure or user funds.

### Changed

- `cow_sdk_core::OrderBalance` is replaced with two distinct contract
  types — `SellTokenSource { Erc20, External, Internal }` and
  `BuyTokenDestination { Erc20, Internal }` — modeling the sell-side
  allowance path and the buy-side payout path as separate enums that
  mirror the services `model::order::SellTokenSource` and
  `model::order::BuyTokenDestination` byte-identically on the wire.
  Every `OrderCreation`, `UnsignedOrder`, `QuoteData`, `Order`,
  `OrderFlags`, `TradeFlags`, `TradeSimulation`, `TradeParameters`,
  `LimitTradeParameters`, `QuoteRequestOverride`, `QuoteCacheKey`, and
  related SDK surface now carries the side-specific type on its
  `sell_token_balance` and `buy_token_balance` fields, so quote-derived
  and direct trading-order construction cannot silently rewrite the
  buy-side destination: the previously-shipped
  `OrderBalance::normalize_for_buy` and
  `cow_sdk_contracts::normalize_buy_token_balance` helpers, which
  collapsed `External` into `Erc20` on the buy side, are retired and the
  type system now rejects any cross-side assignment at compile time. A
  fixture round-trip test pins both enums to the services kebab-case
  wire strings (`"erc20"`, `"external"`, `"internal"`) and the closed
  `BuyTokenDestination` domain rejects the sell-only `"external"` value
  on deserialization; a pinned compile-fail witness under
  `crates/core/tests/ui/` guards the cross-side rejection against
  regression. Both new enums are `#[non_exhaustive]` with
  `Default = Erc20`, derive the full
  `Debug + Clone + Copy + PartialEq + Eq + Hash + Serialize + Deserialize`
  set, re-export through the root `cow-sdk` facade and `cow_sdk::prelude`,
  and surface from the `cow-sdk-orderbook` crate for downstream consumers
  that construct orderbook DTOs directly.
- `cow_sdk_subgraph::SubgraphApi` is constructed exclusively through the
  typestate `SubgraphApi::builder()` so the compiler enforces that the
  chain id, partner Graph API key, and HTTP transport are all supplied
  before `.build()` becomes callable. The builder accepts an
  `Arc<dyn HttpTransport + Send + Sync>` via `.transport(...)` and
  exposes optional fluent setters for the `SubgraphTransportPolicy`,
  per-chain base-URL map, and a shared `reqwest::Client` for multi-chain
  connection-pool reuse. On native targets the builder also exposes a
  `.build()` overload that defaults the transport to `ReqwestTransport`,
  so the common single-target consumer never has to wire a transport
  explicitly; on `wasm32` targets the caller must supply a
  `FetchTransport` from `cow-sdk-transport-wasm` before `.build()`
  becomes reachable. The legacy `SubgraphApi::new`, `with_config`,
  `with_config_and_transport_policy`, `from_shared_client`,
  `from_shared_client_with_config`, and
  `from_shared_client_with_transport_policy` free constructors are
  retired; the post-construction `with_transport_policy` modifier
  remains available for adjusting an existing instance. `SubgraphApi`
  surface continues to live in the dedicated `cow-sdk-subgraph` crate
  and is not re-exported through the root facade.
- `cow_sdk_orderbook::OrderBookApi` is constructed exclusively through the
  typestate `OrderBookApi::builder()` (or the convenience
  `OrderBookApi::builder_from_context(ApiContext)` seed) so the compiler
  enforces that the chain id, environment, and HTTP transport are all
  supplied before `.build()` becomes callable. The builder accepts an
  `Arc<dyn HttpTransport + Send + Sync>` via `.transport(...)` and exposes
  optional fluent setters for the `OrderBookTransportPolicy`, partner
  API key, base-URL map, per-environment base-URL overrides, and a shared
  `reqwest::Client` for multi-chain connection-pool reuse. On native
  targets the builder also exposes a `.build()` overload that defaults the
  transport to `ReqwestTransport`, so the common single-target consumer
  never has to wire a transport explicitly; on `wasm32` targets the
  caller must supply a `FetchTransport` from `cow-sdk-transport-wasm`
  before `.build()` becomes reachable. The legacy `OrderBookApi::new`,
  `new_with_transport_policy`, `from_shared_client`,
  `from_shared_client_with_transport_policy`, and `new_with_base_url`
  free constructors are retired; the post-construction
  `with_transport_policy`, `with_env_base_url`, and
  `with_context_override` modifiers remain available for adjusting an
  existing instance. `OrderBookApiBuilder` is re-exported through the
  facade and prelude.
- `cow_sdk_core::HttpTransport` is dyn-compatible through `async-trait`,
  so downstream clients compose transports as `Arc<dyn HttpTransport>`
  without reaching for a bespoke adapter trait. Both the native
  `ReqwestTransport` default and the browser `FetchTransport` default
  carry the matching `#[async_trait(?Send)]` impl annotation so the
  trait-object dispatch compiles on every supported runtime.
- `ContractsError`, `AppDataError`, and `RegistryError` carry typed
  underlying sources through `#[source]` chains.
  `ContractsError::DecodeHex { field, source: hex::FromHexError }`,
  `ContractsError::InvalidHexPrefix { field }`, and
  `ContractsError::InvalidDecodedLength { field, expected, actual }`
  partition the contracts hex-decode diagnostic surface so each arm
  exposes its underlying cause typed; `AppDataError::Schema` pairs a
  path-prefixed display message with a typed
  `jsonschema::ValidationError<'static>` source;
  `AppDataError::Calculation` carries the typed `cid` / `multihash`
  failure through a boxed trait-object source; and
  `RegistryError::Parse { source: toml::de::Error }` exposes the typed
  TOML-deserialization error.
- The browser `FetchTransport` default uses the browser-native
  `redirect: "follow"` fetch mode, so redirect-chain failures surface as
  `TypeError`-shaped DOMExceptions classified through
  `TransportErrorClass::Connect`. The module-level rustdoc documents the
  redirect-mode choice.
- Lifted `cow_sdk_core::HttpTransport` to the production injection point for
  HTTPS REST traffic and shipped `cow_sdk_core::transport::reqwest::ReqwestTransport`
  as the native default implementation. The trait now exposes `async fn`
  methods returning a typed `TransportError` so browser and native adapters
  share one async-first signature. The native default classifies every
  underlying `reqwest::Error` through the documented `is_timeout`,
  `is_connect`, `is_redirect`, `is_decode`, `is_body`, `is_builder`,
  `is_request`, `is_status`, fallthrough partition and calls
  `reqwest::Error::without_url` before wrapping so endpoint URLs never leak
  through the error surface. URL-bearing configuration rides the existing
  `Redacted` newtype so the base URL stays redacted in debug, display, and
  serialized output. The facade and prelude re-export `HttpTransport`,
  `TransportError`, `ReqwestTransport`, and `ReqwestTransportConfig` so
  downstream consumers reach the new transport seam through a single import.
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
- The canonical atomic amount type is now `cow_sdk_core::Amount(BigUint)`.
  A single typed newtype carries every atomic quantity across
  `cow-sdk-core`, `cow-sdk-orderbook`, `cow-sdk-trading`, `cow-sdk-signing`,
  `cow-sdk-app-data`, and `cow-sdk-contracts`; the custom
  `Serialize`/`Deserialize` impls emit and parse the canonical base-10
  decimal string so every DTO, ABI encoder, and pinned parity fixture
  remains byte-identical against upstream. `Amount::new` accepts decimal
  and `0x`-prefixed hex literals as before, and `Amount::from_atoms`,
  `Amount::as_biguint`, and `Amount::into_biguint` expose the inner
  `BigUint` directly for typed arithmetic.
- Workspace dependency hygiene. The `http` crate now lives in the root
  `[workspace.dependencies]` table pinned at `1.4.0` alongside the other
  shared transport crates, so `cow-sdk-core` consumes it through a single
  `http.workspace = true` declaration instead of an inline `1.3.1` pin.
  `crates/app-data/Cargo.toml` now carries exactly one `serde_json`
  declaration (with the `preserve_order` feature preserved) routed through
  `[workspace.dependencies]`, rather than a duplicated entry across the
  `[dependencies]` and `[dev-dependencies]` tables. The internal
  backward-compatibility `TypedOrder = UnsignedOrder` alias in
  `cow-sdk-signing` is retired; the canonical `UnsignedOrder` type is the
  single name for the pre-signature order state exported through the public
  signing surface and the `cow-sdk` prelude.
- `docs/parity-scope.md` now carries an explicit `Intentionally Out-of-Scope`
  section enumerating the upstream TypeScript-SDK surfaces that `cow-rs`
  intentionally declines to mirror, each entry paired with its rationale
  and the negative parity test that enforces the exclusion at the code
  level. ADR 0011 is refreshed in place to describe the canonical typed
  `Amount(BigUint)` newtype as the single atomic-amount shape and to
  cross-link the parity-scope document as the authoritative exclusion
  list; reviewers can now navigate between the ADR and the scope doc
  without private-context chasing.
- Strengthen error types on `ContractsError`, `AppDataError`,
  `OrderbookError`, and `TradingError`: wrapper variants that previously
  held arbitrary external error strings now carry typed `#[from]`
  converters (`ContractsError::Abi(alloy_sol_types::Error)`,
  `ContractsError::Serialization(serde_json::Error)`,
  `AppDataError::Json(serde_json::Error)`,
  `OrderbookError::Serialization(serde_json::Error)`) or structured
  validation fields (`{ field, reason }` for every validation-class
  variant; `{ operation, message }` for `ContractsError::Provider`;
  `{ message }` for `ContractsError::Decode`, `AppDataError::Schema`,
  and `AppDataError::Calculation`; `{ class, detail }` for the REST
  transport variants on `AppDataError` and `OrderbookError`;
  `{ status, message }` for `AppDataError::Pinning`). A new
  `cow_sdk::ValidationReason` enum describes the canonical validation
  failure modes (`Missing`, `OutOfRange`, `BadShape`, `Precondition`)
  and surfaces through `cow_sdk::prelude::*`; a new
  `cow_sdk::TransportErrorClass` enum classifies REST-transport failure
  categories (`Timeout`, `Connect`, `Redirect`, `Decode`, `Body`,
  `Builder`, `Request`, `Status`, `Other`) and is re-exported from the
  facade. `ContractsError::Decode` has also been split so hex-decode
  failures carry a structured `{ field, message }` payload and
  trade-index violations surface through the dedicated
  `ContractsError::InvalidTokenIndex { index, registered }` variant.
- Tighten the typed shape of three public error variants so downstream
  callers can pattern-match on the typed payload without re-parsing
  error messages: `cow_sdk_contracts::ContractsError::MissingClearingPrice`
  now carries `{ token: cow_sdk_core::Address }` in place of the prior
  stringly-typed token payload; `cow_sdk_contracts::ContractsError::Eip1271MagicValueMismatch`
  now carries `{ expected: [u8; 4], actual: [u8; 4] }` in place of the
  prior hex-string payload, matching the four-byte EIP-1271 function
  selector the protocol uses on the wire; and
  `cow_sdk_trading::TradingError::RecoverableSignatureOwnerMismatch` now
  carries `{ owner: cow_sdk_core::Address, signer: cow_sdk_core::Address }`
  in place of the prior stringly-typed address payload. The `Display`
  output for each variant is unchanged: addresses render as their
  canonical `0x`-prefixed hex form and the 4-byte magic values render as
  `0x`-prefixed lowercase hex.
- The `alloy-sol-macro` and `alloy-sol-types` crates now live in the root
  `[workspace.dependencies]` table pinned at `1.5.7` alongside the existing
  `alloy-primitives`, `alloy-dyn-abi`, and `alloy-json-abi` declarations.
  No crate consumes the new dependencies in this release yet; the workspace
  table pin ensures every future consumer of the `alloy::sol!` macro idiom
  resolves against a single authoritative version, keeping the Ethereum
  primitives stack consistent across the published surface.
- `cow-sdk-contracts` now derives its `GPv2Settlement` call-data bindings from
  an `alloy::sol!` interface block sourced from the upstream
  `cowprotocol/contracts` Solidity surface. The `SettlementEncoder` order-refund
  interactions (`freeFilledAmountStorage` and `freePreSignatureStorage`) now
  produce their ABI call-data through the generated typed selector encoders, and
  a new `SettlementEncoder::encoded_settlement_calldata` helper exposes the
  fully ABI-encoded `settle(...)` payload for downstream transaction builders.
  The `GPv2Order` EIP-712 type-hash, the 56-byte `OrderUid` layout, and the
  trade-flags bit encoding remain byte-identical to the pre-migration
  baseline, and the committed Solidity excerpt at
  `crates/contracts/abi/settlement/GPv2Settlement.sol` preserves upstream
  provenance for reviewers.
- `cow-sdk-contracts` now derives its `GPv2VaultRelayer` authorization-role
  bindings from an `alloy::sol!` interface block that declares the canonical
  GPv2 Vault Relayer surface alongside the partial Balancer V2 Vault ABI the
  relayer proxies (`manageUserBalance` and `batchSwap`). Vault role hashes
  returned by `required_vault_roles` now source their 4-byte method selectors
  from the generated typed interface and derive the role digest through the
  `alloy-sol-types` ABI-encoded `(address, bytes4)` tuple, keeping the
  role-hash byte output identical to the pre-migration baseline. The
  committed Solidity excerpt at
  `crates/contracts/abi/vault-relayer/GPv2VaultRelayer.sol` preserves
  upstream provenance for reviewers.
- `cow-sdk-contracts` now hosts the typed `CoWSwapEthFlow` call-data
  bindings under a new `cow_sdk_contracts::eth_flow` module generated from
  an `alloy::sol!` interface block sourced from the upstream Solidity
  surface. The module exposes a typed `EthFlowOrderData` payload plus
  `encode_create_order_calldata` and `encode_invalidate_order_calldata`
  helpers, the latter covering the `invalidateOrder(EthFlowOrderData)` entry
  point that is distinct from the `GPv2Settlement::invalidateOrder(bytes)`
  call used for regular orders. `cow-sdk-trading` now produces its EthFlow
  order-creation and on-chain-cancellation transaction call-data through the
  new typed encoder; signed `int64` quote ids round-trip through the ABI
  boundary with canonical two's-complement sign-extension, and the call-data
  layout now matches the upstream on-chain struct field order byte-for-byte.
  The typed EthFlow surface is re-exported as `cow_sdk_trading::eth_flow`
  for downstream consumers, and the committed Solidity excerpt at
  `crates/contracts/abi/eth-flow/CoWSwapEthFlow.sol` preserves upstream
  provenance for reviewers.
- The `cow-sdk-contracts` EIP-1967 proxy-inspection surface now derives
  from an `alloy::sol!` interface block that declares the canonical
  EIP-173 ownership proxy ABI alongside the EIP-1967 storage-slot
  derivations. The paired `IMPLEMENTATION_STORAGE_SLOT` / `OWNER_STORAGE_SLOT`
  hex-string constants and the `proxy_interface` / `EIP173_PROXY_ABI`
  JSON-fragment helpers are replaced by a typed `Eip1967Slot` enum with
  `Admin` and `Implementation` variants carrying the canonical 32-byte slot
  hashes, a public `SlotBytes` type alias for the underlying `B256`
  representation, the generated `IEip173Proxy` interface type, and an
  `admin_address` reader that decodes storage responses through
  `alloy_primitives::Address::from_word` rather than ad-hoc byte slicing.
  The existing `implementation_address` and `owner_address` readers keep
  their signatures and now route through the typed surface; `owner_address`
  stays available as a legacy alias for `admin_address` so downstream
  ownership-proxy consumers migrate without behavioral changes. The
  committed Solidity excerpt at
  `crates/contracts/abi/eip1967/Eip1967.sol` preserves upstream provenance
  for reviewers.

- The typed `cow_sdk_contracts::deployments::Registry` is now the single
  authority for resolving canonical contract addresses from the
  `(ContractId, SupportedChainId, CowEnv)` key triple. The historical
  `cow_sdk_core::settlement_contract_address`,
  `cow_sdk_core::vault_relayer_address`, and
  `cow_sdk_core::eth_flow_contract_address` free-function accessors are
  retired; every caller in the workspace now goes through
  `Registry::default().address(...)`, with configuration-override use
  cases routed through the new `Registry::with_override` extension point
  so callers that previously supplied a local-dev deployment address
  retain that capability without reaching for the retired accessors.
  `AllowanceParameters` and `ApprovalParameters` rename their
  `vault_relayer_address` field and the matching
  `with_vault_relayer_address` builder to `vault_relayer_override` and
  `with_vault_relayer_override` respectively so the name tells the
  reader the field only applies when the canonical registry entry needs
  to be bypassed.
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
  release publish sequence in finished-product language, naming the
  published `cow-sdk` crate family the sequence publishes in dependency
  order.

### Removed

- Retired the hand-rolled ABI encoder helpers previously maintained inside
  `cow-sdk-contracts`. Every encoded call-data payload the SDK emits now
  flows through the `alloy::sol!`-generated typed bindings for
  `GPv2Settlement`, `GPv2VaultRelayer`, `CoWSwapEthFlow`, the EIP-1967
  proxy, and `IERC20` / `IERC20Permit`, so the byte output is sourced
  directly from the upstream CoW Protocol Solidity ABI rather than from
  a parallel Rust reimplementation. Byte-identity parity with the pre-
  migration encoder output is gated by the regression contract at
  `crates/contracts/tests/parity_contract.rs`.
- Retired the legacy free-function constructor family on `OrderBookApi`
  (`new`, `new_with_transport_policy`, `from_shared_client`,
  `from_shared_client_with_transport_policy`, `new_with_base_url`) and
  on `SubgraphApi` (`new`, `with_config`,
  `with_config_and_transport_policy`, `from_shared_client`,
  `from_shared_client_with_config`,
  `from_shared_client_with_transport_policy`). `OrderBookApi::builder()`
  and `SubgraphApi::builder()` are now the sole production construction
  paths; the typestate markers encode the required inputs (chain,
  environment or API key, transport) at compile time so a misconstructed
  client is a build error rather than a first-quote runtime surprise.
  Shared `reqwest::Client` pooling remains available through the
  builder's `.client(shared)` convenience setter on native targets; a
  `trybuild` UI witness asserts `.build()` without `.transport(...)`
  does not compile on `wasm32` targets.
- Retired every CIDv0 (dag-pb + sha2-256, `Qm...`-prefixed) encoding and
  decoding path from `cow-sdk-app-data`. CIDv1 with the raw multicodec
  (`0x55`) over a keccak-256 multihash (`0x1b`) is the only supported
  CID shape, matching the cow-protocol services backend. The retired
  helpers `app_data_hex_to_cid_legacy`, `app_data_hex_to_cid_with_mode`,
  `get_app_data_info_legacy`, `fetch_doc_from_app_data_hex_legacy`,
  `fetch_doc_from_app_data_hex_legacy_with_policy`,
  `upload_metadata_doc_to_ipfs_legacy`, and the `CidMode` enum are no
  longer part of the public surface. The decoder now rejects CIDv0
  inputs at the boundary with a typed `AppDataError::InvalidCid`;
  consumers that need to parse historical Qm-prefixed values use a
  general-purpose `cid` crate directly. The `sha2` dependency has been
  dropped from `cow-sdk-app-data`.
- The order-level `fee_amount` descriptor is no longer a public field or a
  public builder setter on `cow_sdk_orderbook::QuoteData`,
  `cow_sdk_orderbook::OrderCreation`, `cow_sdk_orderbook::Order`, or
  `cow_sdk_orderbook::AuctionOrder`. Order submissions always wire
  `"feeAmount": "0"` to satisfy the services `NonZeroFee` constraint and
  preserve the EIP-712 struct-hash contract, so callers no longer risk
  constructing an order that the orderbook would reject at submission. The
  network-cost amount returned by `/api/v1/quote` is now accessed through
  the typed `QuoteData::network_cost_amount` getter and the
  `with_network_cost_amount` / `set_network_cost_amount` setters.
- The retired `executedFeeAmount` and `fullFeeAmount` descriptors have been
  removed from the orderbook order-response DTO. Fee exposure on the
  response flows through the canonical `executedFee` component, and
  quote-response fee descriptors flow through `protocolFeeBps` only, in
  line with the current services schema. `cow_sdk_orderbook::calculate_total_fee`
  now takes a single `executed_fee` argument and normalizes it into the
  `total_fee` value surfaced on the transformed order.

### Notes

- `0.1.0` will be recorded here when the first functional crates.io release is
  live.

## [0.1.0] - TBD

Placeholder for the first functional crates.io release of the `cow-rs` crate
family. This section will be populated when that release is published.
