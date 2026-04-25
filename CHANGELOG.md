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

- App-data metadata now exposes a typed `HookList` slot on `AppDataParams`
  for hook-bearing documents (cow-shed, flash-loans, bridging). The
  `OrderbookClient` trait is now reachable from `cow-sdk-orderbook` so
  capability consumers can compose against the trait without the trading-crate
  dependency.
- Order quote requests now pre-validate the `(signingScheme,
  onchainOrder)` pair locally so incompatible ECDSA/on-chain
  combinations fail with a typed error before the HTTP call.
  `OrderCreation` also carries an opt-in
  `with_full_balance_check(bool)` builder method matching the upstream
  services policy while preserving the existing wire shape when unset.
- The lowest-level transport seam on both the native and browser adapters now
  emits one tracing span per request with method, endpoint (path-only, never
  the full URL), and byte counts when the `tracing` feature is enabled.
  Default-constructed transports now apply a `cow-sdk/<version>` user-agent
  and a 60-second TCP keepalive aligned with the upstream services defaults.
- The order-book retry orchestrator now emits per-attempt tracing events with
  attempt index, response status or transport error class, and backoff
  duration when the `tracing` feature is enabled, and supports jitter
  strategies for production deployments. The advertised `quote_id`,
  `attempts`, and `status` tracing fields are now populated on the request
  spans they document.
- Typed-amount arithmetic now lives on `cow_sdk_core::Amount` directly,
  with Add, Sub, AddAssign, SubAssign, checked_add, checked_sub, and
  checked_mul operators that delegate to the underlying integer storage.
- Testing depth across `cow-sdk-contracts` now spans every
  `alloy::sol!` binding family through three reinforcing lanes:
  ten new byte-identity parity fixtures in
  `parity/fixtures/contracts.json` pin the call-data output of
  `GPv2Settlement` (`invalidateOrder`, `setPreSignature`,
  `freeFilledAmountStorage`, `freePreSignatureStorage`),
  `GPv2VaultRelayer` (`transferFromAccounts`), `CoWSwapEthFlow`
  (`createOrder` and `invalidateOrder(EthFlowOrderData)`),
  `IERC20` (`approve`, `transferFrom`), and the EIP-2612 Permit
  typed-data digest against the deployed USD Coin domain; five
  new `cargo-fuzz` targets under `fuzz/fuzz_targets/`
  (`fuzz_settlement_settle_encode`,
  `fuzz_settlement_invalidate_order_encode`,
  `fuzz_ethflow_create_order_encode`,
  `fuzz_erc20_permit_typed_data_hash`, and
  `fuzz_vault_relayer_transfer_from_accounts_encode`) drive
  arbitrary input through the same encoders and assert selector
  identity, call-data length-consistency, round-trip identity,
  and the EIP-712 envelope composition invariant, with the
  scheduled fuzz lane matrix now enumerating ten targets; and
  the `PROPERTIES.md` registry gains five new invariant rows
  (`PROP-CORE-008` for the split `SellTokenSource` and
  `BuyTokenDestination` enums governed by ADR 0016,
  `PROP-TRD-004` for the client-side submission validator
  governed by ADR 0015, `PROP-TRD-005` for the `PartnerFee`
  policy round-trip, `PROP-APP-004` for the `AppDataValidation`
  size-limit warning threshold, and `PROP-ORD-004` for the
  typed `OrderbookRejection` parser governed by ADR 0017).
- ADR 0018 (`Typed App-Data Merge As The Single Canonical
  Quote-To-Post Edit Path`), ADR 0019 (`HTTP Transport Is The
  Sole Live-Dispatch Surface On The Orderbook And Subgraph
  Clients`), and ADR 0020 (`EthFlow Transaction Bundle Carries
  The Signer-Derived Owner For Pre-HTTP Validation`) ship
  under `docs/adr/` as the governing decision records for the
  current-state architecture contracts on the trading
  quote-to-post merge path, the orderbook and subgraph
  dispatch surface, and the native-currency submission seam.
  ADR 0018 ships alongside a standing audit at
  `docs/audit/trading-app-data-merge-audit.md`, and ADR 0020
  ships alongside
  `docs/audit/trading-ethflow-owner-identity-audit.md`; both
  audits' `Related docs` blocks reciprocate the ADR `Proven
  by` cross-link contract. The `docs/adr/README.md` index
  lists the three new entries in numerical order after
  `0017`.
- `scripts/fetch-upstream-pins.sh` materializes the pinned
  upstream CoW Protocol repositories
  (`https://github.com/cowprotocol/cow-sdk`,
  `https://github.com/cowprotocol/contracts`, and
  `https://github.com/cowprotocol/services`) at the commits
  recorded in `parity/source-lock.yaml` as independent git
  worktrees outside the cow-rs tree. Pass `--into <dir>` to
  override the default sibling-directory layout; the script is
  idempotent and leaves existing destinations untouched. The
  `docs/parity-sources.md` provenance guide now describes the
  three-layer contract — the committed source-lock as
  authoritative provenance, the independently-materialized
  upstream worktrees as the verification target, and the
  provisioning script as the supported reviewer path.
- ADR 0021 (`Narrow Order.total_fee And Read-Only Legacy
  Executed-Fee Surface`) ships under `docs/adr/`, paired with a
  new `Order.executed_fee_amount_legacy: Option<String>`
  read-only sibling on `cow_sdk_orderbook::Order` that
  deserializes the deprecated `executedFeeAmount` wire field
  through `#[serde(rename = "executedFeeAmount")]`.
  `Order.total_fee` continues to equal the canonical
  executed-fee component normalized through
  `calculate_total_fee`; the legacy field is never folded into
  the canonical sum. Consumers that need the legacy summation
  read both fields and add them at the call site. The
  `docs/adr/README.md` index lists the entry in numerical
  order after `0020`, and `docs/parity-matrix.md` records the
  `Order.total_fee` divergence under the `Orderbook DTO
  defaults` section.
- `build_app_data` stamps a Rust-identified default
  `metadata.utm` attribution block when the caller does not
  supply `metadata.utm`, carrying `utmSource = "cowmunity"`,
  `utmMedium = "cow-rs@<crate-version>"`,
  `utmCampaign = "developer-cohort"`, `utmContent = ""`, and
  `utmTerm = "rs"` so downstream analytics can attribute
  traffic to the Rust SDK and its published version. Any
  caller-supplied `metadata.utm` key — partial or full — fully
  replaces the default block and is carried through
  byte-identical.
- ADR 0015 (`Typed Client-Side Order-Bounds Validator On Every
  Trading Submission Seam`), ADR 0016 (`Split SellTokenSource And
  BuyTokenDestination Into Distinct Side-Specific Enums`), and ADR
  0017 (`Typed OrderbookRejection Parser With Permanent Unknown-Tag
  Fallback`) ship under `docs/adr/`. ADR 0015 ships alongside a
  standing audit at
  `docs/audit/trading-order-bounds-validator-audit.md` whose
  `Related docs` block reciprocates the ADR's `Proven by`
  cross-link, and the `docs/adr/README.md` index lists the three
  new entries in numerical order after `0014`.
- Typed client-side validator on every trading submission path.
  `cow_sdk_trading::OrderBoundsValidator` pairs
  `OrderValidityBounds` (with the published services defaults
  `min = 60s`, `max_market = 3h`, `max_limit = 1y`) and a
  `SubmissionClass` discriminator to enforce the reviewed protocol
  invariants at the client before any bytes cross the wire. The
  typed `ClientRejection` enum ships with `#[non_exhaustive]` and
  the full launch variant set `ValidToInsufficient`,
  `ValidToExcessive`, `MissingFrom`, `AppdataFromMismatch`,
  `SameBuyAndSellToken`, `InvalidNativeSellToken`, `ZeroAmount`
  (discriminated by the typed `AmountSide` enum), and
  `OwnerMismatch`. `TradingSdkBuilder::with_order_bounds` is an
  additive setter that defaults to
  `OrderValidityBounds::SERVICES_DEFAULT`, and
  `TradeParameters::validate` / `LimitTradeParameters::validate`
  expose the builder-level subset of the protocol-invariant matrix
  for callers that assemble an order outside the hot submission
  path. `Amount::is_zero` is now exposed on
  `cow_sdk_core::types::Amount` for predicate-style checks.
- Typed flash-loan hints and signer fields on the app-data metadata
  shape. `cow_sdk_app_data::FlashloanHints` is a new
  `#[non_exhaustive]` Rust type with five required fields —
  `liquidityProvider`, `protocolAdapter`, `receiver`, `token`, and
  `amount` — that narrow the reviewed flash-loan hint envelope from a
  free-form JSON object into a byte-identical camelCase typed
  struct. The companion `FlashloanHints::new` constructor and
  `FlashloanHints::validate` method enforce the published bounds by
  rejecting a zero `amount` and every zero-address field before a
  document would fail the reviewed schema. `AppDataParams` gains two
  typed sub-metadata fields — `signer: Option<Address>` and
  `flashloan: Option<FlashloanHints>` — that carry the reviewed
  `metadata.signer` and `metadata.flashloan` positions on the wire;
  the open-ended `AppDataParams.metadata` slot remains available for
  every other metadata sub-object, and a new typed
  `AppDataError::InvalidFlashloanHints { field, reason }` variant
  reuses the shared `cow_sdk_core::ValidationReason` enum so callers
  can pattern-match on the validation-failure mode without parsing
  free-form strings.
- Typed orderbook-rejection enum with structured per-code variants.
  `cow_sdk_orderbook::OrderbookRejection` ships a
  `#[non_exhaustive]` classification of every authoritative
  `errorType` tag emitted by the CoW Protocol orderbook across order
  submission, quoting, cancellation, and price-estimation flows — for
  example `DuplicatedOrder`, `MissingFrom`, `AppdataFromMismatch`,
  `SameBuyAndSellToken`, `InvalidNativeSellToken`,
  `UnsupportedBuyTokenDestination`, `UnsupportedSellTokenSource`,
  `UnsupportedOrderType`, `UnsupportedToken`, `NonZeroFee`,
  `InsufficientBalance`, `InsufficientAllowance`, `InvalidSignature`,
  `SellAmountOverflow`, `TransferSimulationFailed`, `WrongOwner`,
  `InvalidEip1271Signature`, `ZeroAmount`,
  `IncompatibleSigningScheme`, `TooManyLimitOrders`, `TooMuchGas`,
  `QuoteNotVerified`, `QuoteNotFound`, `InvalidAppData`,
  `AppDataHashMismatch`, `MetadataSerializationFailed`,
  `OldOrderActivelyBidOn`, `ExcessiveValidTo`, `InsufficientValidTo`,
  `Forbidden`, `NoLiquidity`, `TradingOutsideAllowedWindow`,
  `TokenTemporarilySuspended`, `InsufficientLiquidity`,
  `CustomSolverError`, `AlreadyCancelled`, `OrderFullyExecuted`,
  `OrderExpired`, `OrderNotFound`, `OnChainOrder`, and
  `InternalServerError`. The single wire variant that carries
  machine-readable data — `SellAmountDoesNotCoverFee` — exposes the
  services `data.fee_amount` payload through a typed `fee_amount:
  Amount` field. The tail variant `Unknown { code, message }`
  preserves forward compatibility so a newly-introduced services
  tag never silently coerces to a default placeholder, and the
  accompanying free function
  `cow_sdk_orderbook::parse_rejection(status, body)` exposes the
  same classification at the byte-slice level for consumers that
  hold a raw HTTP response instead of an `OrderBookApiError`.
  `OrderbookRejection` and `parse_rejection` re-export through the
  `cow-sdk` facade and `cow_sdk::prelude`.
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
  is only available once the chain-id marker is `Set`. A `TradingSdkMode`
  enum (`Ready` or `HelperOnly`) plus the new
  `TradingError::HelperOnlyMode` variant fail
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

### Documentation

- The `cow-sdk-transport-wasm` crate now ships a per-crate README that
  renders on docs.rs alongside the inline `lib.rs` doc comments. The
  `ContractId` enum documentation now names the Pascal-case convention and
  the version-suffix style for new variants.
- Getting Started, the facade crate README, and the trading crate README now
  document helper-only `TradingSdk` construction for allowance, approval,
  pre-sign, and on-chain cancellation workflows that do not need quote or
  submission flows.
- EIP-1271 verification helpers now document the
  no-pre-interaction-simulation caveat for watchtower consumers.
- Three new standing audits cover the workspace `unsafe_code = deny`
  lint posture, the panic-free public surface contract, and the
  workflow security posture (CI action pinning, permissions
  discipline, and `pull_request_target` zero-tolerance).

### Fixed

- Legacy compatibility helpers in the contracts crate that produced
  protocol-incorrect digests by zeroing amounts before hashing have
  been removed. Order digest computation now flows exclusively through
  the canonical unsigned-order to order path, which produces
  byte-identical output to the upstream service for the same input.
- Native ethflow simulation example now passes the typed
  order-validity bounds and the optional app-data signer through
  the native-currency posting seam so the reviewed submission
  contract is exercised end-to-end on the native evidence
  surface. The scenario computes its `valid_to` against the
  current wall clock so the sample stays inside
  `OrderValidityBounds::SERVICES_DEFAULT` indefinitely without
  a hard-coded timestamp drift.
- Browser wallet console and SDK verification console examples
  now compose the fetch-backed transport inside a wasm32 build
  branch and fall back to the default reqwest transport on the
  host target, so both examples build as the `rlib` targets
  declared in their manifests without referencing the wasm-only
  transport crate root from host code. A narrow compile-time
  symbol smoke in each example's test directory names the
  transport types under a wasm32 gate so future export drift
  surfaces at build time.
- SDK verification console now unwraps the validated
  `PartnerFeePolicy::volume` constructor at the typed-defaults
  composition site so the demo payload always carries a
  `PartnerFee` value produced through the typed partner-fee
  bounds. A narrow regression in the example's test directory
  locks the typed-defaults round-trip so the validator contract
  cannot silently drift.
- `scripts/check-release-docs-agree.sh` and
  `scripts/fetch-upstream-pins.sh` carry executable file mode in
  the tracked index so the release-gate docs-agreement check and
  the documented upstream-provisioning tool run by their bare
  paths on every contributor platform without a shell prefix.
- Trading submission seam no longer panics on
  `wasm32-unknown-unknown` when the typed order-bounds validator
  reads the current instant. The internal `current_unix_seconds`
  helper now reads the clock through the same dual-target
  `std::time` on native and `web_time` on wasm32 shape already
  used by the order-derivation surface, matching the reviewed
  cross-runtime contract so browser-wallet-backed submission
  flows stay live on wasm32 builds.
- Client-side order validation on the non-native-currency posting
  path now runs before the app-data document is uploaded and
  before the signer is prompted, so rejected orders no longer
  persist pre-submission work.
- IPFS base-URI preflight now fails closed symmetrically across the
  read and write paths. The `cow_sdk_app_data::pin_json_in_pinata_ipfs`
  helper rejects an empty, whitespace-only, or slash-only `write_uri`
  with a typed `AppDataError::Transport { class:
  TransportErrorClass::Builder, detail: "ipfs write base uri must
  not be empty" }` before any bytes cross the upload transport,
  matching the existing read-side guard that surfaces the
  corresponding `"ipfs read base uri must not be empty"` detail for
  a malformed read base URI. Valid inputs are normalized identically
  on both sides: leading and trailing whitespace is stripped and a
  single trailing `/` is trimmed, so `https://api.pinata.cloud/` and
  `https://api.pinata.cloud` build the same
  `https://api.pinata.cloud/pinning/pinJSONToIPFS` upload URL.
- Eth-flow submission validation now reads the client-side `from`
  identity from the signer-derived owner carried on
  `cow_sdk_trading::EthFlowTransaction`, not from
  `order_to_sign.receiver`. The typed bundle gains a
  `from: cow_sdk_core::Address` field populated at transaction
  construction from the existing signer address resolution, and
  `post_sell_native_currency_order_async` feeds that owner into the
  client-side `OrderBoundsValidator` before any transport. Payout
  receivers that legitimately differ from the owner no longer trip a
  false `ClientRejection::AppdataFromMismatch`, and the mismatched
  app-data signer case now reports the owner as the typed rejection's
  `from` field so downstream diagnostics and pattern-matching stay
  aligned with the signing authority rather than the payout recipient.
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

- API-key validation paths now fail with a typed error variant rather than a
  panic that could include the offending bytes. Base-URL override fields are
  redacted from `Debug` formatting so credentials embedded in override URLs no
  longer surface in diagnostic output. A new `sanitize_public_base_url` helper
  in `cow-sdk-core` strips path, query, and fragment from URLs before they
  cross any logging or tracing boundary.
- Dependency audit gate advances past the reachable
  certificate-revocation-list parsing panic reported for
  `rustls-webpki 0.103.12` (RUSTSEC-2026-0104). The reqwest
  transport chain used by the orderbook and subgraph clients now
  resolves through `rustls-webpki 0.103.13` without a workspace
  override. The standing reviewed-upstream ignore contract in the
  dependency-audit gate additionally records the `core2` yanked
  and unmaintained posture reachable through `cid 0.11.1`
  (RUSTSEC-2026-0105) under the same explicit-reason rationale
  that covers the previously-tracked upstream advisories. The
  governing dependency-gate and CID audit records refresh to the
  new posture, and the release-checklist, verification-matrix,
  and verification-guide surfaces quote the full reviewed
  audit-gate invocation.
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

### Removed

- Removed the permissive runtime-validated `TradingSdkBuilder::build` and
  `TradingSdkBuilder::build_partial` terminals; the typestate-gated
  `build_ready` and `build_helper_only` terminals are now the only
  construction paths. Pre-release surface; zero migration cost.

### Changed

- Typestate marker structs across the workspace are now sealed against
  external construction.
- Partner-fee policies now reject the zero address as the recipient through
  app-data validation and trading quote construction before quote transport.
  The client-side order-bounds validator documentation now explicitly frames
  the validator as defence-in-depth and names broader services rejection
  classes that the SDK does not pre-cover.
- Operator-side base-URL override and browser-wallet trust threat surfaces are
  now documented in `SECURITY.md` with explicit consumer-side mitigations.
- Subgraph transport errors now carry a typed class alongside the details
  string, matching the order-book error model. Cancellation events are now
  distinguishable from normal completion via a dedicated `cancelled = true`
  tracing warning when the `tracing` feature is enabled.
- Order-book wire DTO amount fields are now typed; the JSON wire shape is
  unchanged but malformed amount strings now surface as typed deserialization
  failures with the wire-shape error context.
- Public-field types in `cow-sdk-core` are now marked non-exhaustive so
  future protocol-driven field additions ship as additive minor changes.
- The pre-release stability sweep is now consolidated across
  core, contracts, orderbook, browser-wallet, signing, app-data,
  and async-provider surfaces: public DTOs and enum heads use
  forward-compatible constructors where needed, credential-bearing
  builder and IPFS upload boundaries redact secret material,
  signed amounts use arbitrary-precision storage behind the existing
  wire shape, reviewed orderbook rejection tags and retry cooldowns
  have typed handling, browser EIP-1271 cache timing is wasm-safe,
  legacy digest shims are removed, codec fuzz targets start from
  committed corpora, and read-only async providers are separated from
  signer-capable providers.
- The async provider surface now separates read-only chain RPC from signer
  creation. `AsyncProvider` carries only read methods, while the new
  `AsyncSigningProvider: AsyncProvider` extension owns `type Signer` and
  `create_signer`; wallet-capable providers implement both traits and read-only
  adapters implement only the read-only half.
- Two critical codec fuzz targets — covering the canonical order-uid
  pack-unpack pipeline and the EIP-712 typed-data digest pipeline — now
  ship with non-empty corpora seeded from the parity fixture set, so
  weekly fuzz runs no longer start from libFuzzer random initial inputs.
- Public protocol DTOs in the contracts crate are now marked non-exhaustive and ship with explicit constructors so future protocol field additions land additively.
- Test-suite naming and properties-registry classification now
  match the shipped evidence methodology. Boundary-sweep suites on
  the orderbook, trading, and subgraph crates live at
  `tests/invariant_contract.rs`; codec-crate suites on core,
  contracts, signing, and app-data continue to live at
  `tests/property_contract.rs` for real property-based coverage;
  and the `PROPERTIES.md` `Type` column now distinguishes
  `Property` rows (backed by real `proptest!` coverage on codec
  crates) from `Invariant` rows (backed by curated boundary
  sweeps on orchestration crates). Evidence citations in
  `PROPERTIES.md` follow the rename.
- ADR 0013 (`HTTP Transport Injection Seam And Typestate
  Construction For Orderbook And Subgraph`) now cross-links to
  ADR 0019 in the `Links` section and its `Proven by` block
  carries ADR 0019 alongside the HTTP Transport Contract Audit,
  the Typestate Builder Contract Audit, and the
  recording-transport regression modules at
  `crates/orderbook/tests/api_contract.rs` and
  `crates/subgraph/tests/api_contract.rs`. The HTTP Transport
  Contract Audit's `Related docs` block reciprocates the
  cross-link to ADR 0019 so the two-way proof surface between
  the decision record and the standing audit stays symmetrical.
- The previously published narrow `Order.total_fee` policy
  decision record is available under its final public number
  ADR 0021; the `docs/adr/README.md` index and
  `docs/parity-matrix.md` cross-links cite that number.
- The release checklist, the verification matrix, and the
  quality-gate workflow now enforce the
  `cargo tree --invert alloy-provider` invariant over the full
  nine-crate published family including `cow-sdk-browser-wallet`,
  closing the prior gap between the broader invariant claims and
  the enforced commands.
- The release checklist deterministic browser-wallet lane now
  mirrors the maintained workflow exactly: a Chromium and Firefox
  Playwright install, host-side and direct-bridge wasm tests, the
  WASM build of `cow-sdk` with the `browser-wallet` feature, the
  browser-wallet console WASM build and host-side tests, the
  console wasm-bindgen tests under headless Chrome, and the
  Playwright DOM lane under both engines.
- A new `scripts/check-release-docs-agree.sh` lint guards against
  release-doc and CI drift by extracting the cargo-tree
  alloy-provider package list from the release checklist, the
  verification matrix, and the quality-gate workflow, the
  browser-wallet Playwright install line from the release
  checklist, and the matching install line from the
  browser-wallet end-to-end workflow, then failing on any
  disagreement. The lint is wired into the quality-gate workflow
  as a new `docs-agree-on-release-gates` job so the drift class
  is closed at the workspace level instead of patched once.
- `OrderToSignParams::new(...)` now defaults
  `apply_costs_slippage_and_fees` to `true`, aligning the public
  helper with the internal quote and submission flows that already
  fold cost, slippage, partner-fee, and protocol-fee adjustments
  into the unsigned order amounts. Callers that want raw-amount
  payloads call `.with_apply_costs_slippage_and_fees(false)` to
  opt out explicitly.
- `HttpTransport` is now the sole live-dispatch surface on the
  orderbook and subgraph clients. `OrderBookApi` and `SubgraphApi`
  no longer hold a parallel `reqwest::Client`; every REST and GraphQL
  call dispatches through the injected
  `Arc<dyn HttpTransport + Send + Sync>`, and injected transports —
  including the browser-native `FetchTransport` from
  `cow-sdk-transport-wasm` — observe every live request. The
  orderbook preserves its rate-limit gate, retry-and-backoff wrapper,
  and typed-error classification around the transport call;
  `OrderBookApi::builder().client(reqwest::Client)` stays on native
  targets as a shorthand that wraps a caller-supplied `reqwest::Client`
  into a `ReqwestTransport` so multi-chain consumers keep one shared
  TCP, TLS, and HTTP/2 connection cache across the clients they
  construct.
- `HttpTransport` gains per-call headers and an optional per-call
  timeout on every method, and `TransportError` gains a typed
  `HttpStatus { status, body }` variant so non-2xx responses flow
  through the typed error channel instead of being smuggled into a
  successful `Result<String, TransportError>::Ok`. The native
  `ReqwestTransport` and the browser `FetchTransport` honor the new
  signature by merging per-call headers with any
  constructor-configured defaults, applying the per-call timeout
  (through `RequestBuilder::timeout` on native and an
  `AbortController` hook on the browser) when supplied, and mapping
  non-2xx responses into `TransportError::HttpStatus`. The trait
  additionally gains a `put` method to cover the full `OrderBookApi`
  method set without bypassing the transport seam; downstream crates
  that implemented the prior trait signature must update their
  adapters to the new method signatures. The trait futures are
  `Send` on native targets and `!Send` on `wasm32` targets so the
  transport composes onto multi-threaded native runtimes while the
  browser adapter remains viable.
- Quote-to-post app-data edits now run through a typed merge
  pipeline. `cow_sdk_trading::merge_and_seal_app_data` is the
  canonical merge-and-seal helper: it deserializes the sealed
  quote-derived wire document back into typed
  `cow_sdk_app_data::AppDataParams`, merges the base and override
  as typed values, and re-emits the canonical wire document
  through the existing `generate_app_data_doc` plus
  `get_app_data_info` pipeline. The helper returns both the
  `TradingAppDataInfo` and the typed merged `AppDataParams`, so
  the swap-from-quote submission path reads
  `metadata.signer` directly from the merged typed value rather
  than from a free-form JSON path or the untyped override. The
  companion `cow_sdk_trading::params_from_doc` free function
  exposes the typed re-parse step for consumers that want the
  typed `AppDataParams` without re-sealing. With the typed merge
  in place, `metadata.signer`, `metadata.flashloan`, and
  `metadata.hooks` replacement semantics on the quote-to-post
  path now match the reviewed upstream SDK byte-identical: a
  base-doc signer survives into the submitted wire document and
  feeds the `AppdataFromMismatch` validator, the typed
  flash-loan hints lift into `metadata.flashloan` through either
  the base or the override, and an override that carries
  `metadata.hooks` replaces the base-side hooks envelope in full
  rather than recursively merging pre/post sibling arrays.
- Promote both compile-fail witnesses to live `trybuild` harnesses
  that re-prove the captured compile failure on every `cargo test`
  run. The token-balance split witness at
  `crates/core/tests/token_balance_ui.rs` and the partner-fee bps
  width witness at `crates/app-data/tests/partner_fee_contract.rs`
  now drive `trybuild::TestCases::compile_fail` against their
  pinned witness sources, replacing the earlier
  filesystem-presence-plus-snapshot assertions; the regenerated
  `.stderr` snapshots match the actual source filenames and the
  current compiler diagnostic. `trybuild = "1.0.116"` is pinned
  through `[workspace.dependencies]` and consumed via
  `trybuild.workspace = true` in the consuming crates.
- Tighten the typed client-side trading validator surface.
  `TradingSdkBuilder::with_order_bounds` now flows through to
  `TradingSdk` and the submission seam, so a custom
  `OrderValidityBounds` policy actually applies on
  `post_swap_order`, `post_limit_order`, and the eth-flow
  variants. The `OrderBoundsValidator` accepts a chain-specific
  wrapped-native address through `with_weth_address` and rejects
  the paired sell-WETH / buy-native-sentinel case as
  `ClientRejection::SameBuyAndSellToken { token: weth }` to mirror
  the reviewed services token-pair guard. The eth-flow submission
  path now invokes the validator with a typed `is_eth_flow` flag
  so zero-amount, same-token, owner-mismatch, and lifetime checks
  still fire on native-currency sells while the native-sentinel
  sell-token check is correctly skipped. The validator's
  `app_data_signer` parameter is by value (`Option<Address>`) so
  call sites pass typed addresses without `.as_ref()`. The post
  pipeline reads the typed
  `cow_sdk_app_data::AppDataParams::signer` field directly instead
  of parsing a free-form JSON path.
- `TradingError` gains a typed `ClientRejected(ClientRejection)`
  variant that surfaces the new client-side validator output as a
  structured payload; the prior
  `RecoverableSignatureOwnerMismatch` variant from the recoverable
  signing contract is retired in favour of
  `ClientRejection::OwnerMismatch`, whose typed owner and recovered
  fields preserve the diagnostic information that downstream
  callers pattern-match on.
- Return shape of app-data info construction carries typed
  validation metadata. `cow_sdk_app_data::get_app_data_info` now
  returns `Result<AppDataValidated, AppDataError>`, where
  `AppDataValidated { info: AppDataInfo, validation: AppDataValidation { bytes_used, warnings } }`
  pairs the canonical deterministic result with a typed observation
  channel. A new `AppDataWarning` enum ships with
  `#[non_exhaustive]` and a single launch variant
  `ApproachingSizeLimit { bytes_used, max_bytes }` that fires when
  the stringified deterministic payload reaches or exceeds
  `APP_DATA_APPROACHING_LIMIT_RATIO` (default 0.75) of
  `APP_DATA_MAX_BYTES`; hard errors — unknown keys, schema
  violations, and oversized payloads — remain on the
  `AppDataError` path and `AppDataValidated` is never constructed
  when `AppDataError::TooLarge { actual_bytes, max_bytes }` fires.
  `AppDataValidated` implements `Deref<Target = AppDataInfo>` so
  every existing caller that reads `cid`, `app_data_hex`, or
  `app_data_content` through dot notation continues to compile
  without code change; callers that need to move the underlying
  `AppDataInfo` out destructure `validated.info`.
- Partner-fee validation surface is tightened across the public
  contract. Every basis-point field on
  `cow_sdk_app_data::PartnerFee` and
  `cow_sdk_app_data::PartnerFeePolicy` narrows from `u32` to `u16`, so
  values outside the published partner-fee range are rejected at the
  compiler rather than at the wire, and both enums gain
  `#[non_exhaustive]` so future wire shapes may be introduced as a
  minor change without breaking downstream exhaustive matches. A new
  `PartnerFee::validate` / `PartnerFeePolicy::validate` surface
  enforces the published basis-point ranges (`volumeBps` and
  `maxVolumeBps` in `[1, 100]`; `surplusBps` and
  `priceImprovementBps` in `[1, 9999]`) and rejects the zero address
  as a partner-fee recipient; the three typed constructors
  `PartnerFeePolicy::volume`, `::surplus`, and `::price_improvement`
  now return `Result<Self, AppDataError>` and run `validate` before
  admitting the value. `PartnerFee::from_value` no longer leaks a
  raw `serde_json::Error` across the crate boundary — its signature
  returns `Result<Self, AppDataError>` and surfaces invalid inputs
  through the existing `AppDataError::Json` conversion, and a new
  typed `AppDataError::InvalidPartnerFee { field, reason }` variant
  reuses the shared `cow_sdk_core::ValidationReason` enum so callers
  can pattern-match on the validation-failure mode without parsing
  free-form strings. The deserializer additionally accepts the
  reviewed legacy `{ bps, recipient }` shape and promotes it to the
  modern `Volume { volume_bps, recipient }` shape on input, keeping
  parse-time parity with the reviewed services behaviour while every
  value emitted on the wire continues to use the modern `volumeBps`
  key.
- Orderbook transport error surface now carries a typed rejection
  variant. `cow_sdk_orderbook::OrderbookError` gains
  `Rejected { status: http::StatusCode, rejection: OrderbookRejection,
  source: Box<OrderBookApiError> }`, which the `From<OrderBookApiError>`
  conversion promotes whenever the non-2xx response body carries a
  valid services rejection envelope. Bodies that fail envelope
  decoding, responses without an `errorType` tag, and empty bodies
  continue to surface through the existing
  `OrderbookError::Api(Box<OrderBookApiError>)` arm so no rejection is
  silently misclassified. The raw transport envelope
  `OrderBookApiError` stays on the public surface for telemetry and
  diagnostics, and its fields continue to expose the HTTP status,
  status text, decoded body, and rendered message; the prior
  stringly-typed `OrderBookApiError::error_type() -> Option<&str>`
  helper is retired in favour of the typed `OrderbookRejection`
  classification path, and the multi-environment order-lookup
  fallback honours both `Api` and `Rejected` on a 404 response.
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
  `onchain_cancellation_transaction`, `build_app_data`,
  `merge_and_seal_app_data`, `suggest_slippage_bps`, `TradingSdk`, and
  `protocol_options_for_order` — with per-field `assert_eq!` messages
  that name the fixture case id and the diverging field at once.
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

### Changed

- Continuous integration now enforces an `alloy-*` workspace-pin
  same-minor invariant on every PR, and an inner-workspace WASM pin diff
  against the workspace pins so the example consoles cannot drift away
  from the workspace lock-step.
- The `cow-sdk-browser-wallet-console` crate name no longer carries the
  redundant `-wasm` suffix, matching the `cow-sdk-<capability>-console`
  naming convention.
- The `cow-sdk` prelude now exposes a curated first-touch surface for common
  quote, sign, post, app-data validation, transport/provider wiring, and
  primary error-handling workflows; reach specialized APIs through the
  named-module re-exports. Workspace MSRV bump policy is now documented with
  explicit cadence and notice window.
- The `SignedAmount` type on `cow-sdk-core` now stores its
  value as an arbitrary-precision integer internally and
  exposes typed accessors and arithmetic delegation. The
  decimal-string wire serde shape is preserved, so wire DTOs
  that carry signed amounts are unchanged on the wire.
- Three additional services-emitted rejection tags now flow
  through typed orderbook variants instead of the generic
  unknown-fallback shape: app-data invalid, app-data mismatch
  on registration, and lookup-path not-found responses are each
  routed to a dedicated variant with a documented distinction
  from the cancel-path not-found case.
- Partner API keys on `OrderBookApiBuilder` and
  `SubgraphApiBuilder`, plus IPFS pinning header values at the
  `IpfsUploadTransport::post_json` boundary, now flow through
  `Redacted<String>` wrappers. Builder debug output and Pinata
  upload-header debug formatting no longer expose secret bytes.
  `IpfsUploadTransport::post_json` now receives header values as
  `&[(String, Redacted<String>)]`; transport implementations call
  `.into_inner()` when they need the raw header bytes.
- Public wallet session, event, error payload, discovery, and
  chain-management types in `cow-sdk-browser-wallet` are now
  `#[non_exhaustive]`, and the constructor-backed structs expose
  explicit `new(...)` entry points so future EIP-1193 amendments
  and wallet-side capabilities land additively.
- `cow_sdk_core::SupportedChainId`, `cow_sdk_core::CowEnv`, and
  `cow_sdk_core::UnsignedOrder` are now `#[non_exhaustive]` public
  surfaces so additive chain, environment, and order-shape evolution
  remains semver-compatible ahead of `0.1.0`. Downstream crates now
  construct unsigned orders through `UnsignedOrder::new(...)` and its
  chainable `with_receiver`, `with_app_data`, `with_fee_amount`,
  `with_partially_fillable`, `with_sell_token_balance`, and
  `with_buy_token_balance` setters, while downstream `match` expressions
  over `SupportedChainId` and `CowEnv` must include wildcard fallback
  arms.
- The orderbook crate's ECDSA signing-scheme enum, auction order
  envelope, and request-policy structs are now marked non-exhaustive,
  and the request-policy surface exposes explicit constructors so
  future signing schemes, auction-side fields, and policy settings land
  additively.
- Public-field types in `cow-sdk-app-data`, `cow-sdk-subgraph`,
  `cow-sdk-signing`, and `cow-sdk-trading` are now marked non-exhaustive
  so future protocol-driven field additions ship as additive minor
  changes.
- `TradingSdkBuilder::build_ready()` on `wasm32` targets now fails fast with a typed error when no orderbook client has been injected, instead of deferring the failure to the first quote or post call.
- The release-gate docs-agreement check now guards the `cargo tree` and `cargo audit` invariants across every source-of-truth document and ships with a self-test harness that catches extraction drift in the check itself.
- Shipped WASM consoles now carry a clear acknowledgement of their current dual-authority posture - the publication authority named in the workspace crate metadata and the hosted-build authority named in the footer links - so reviewers can read the two surfaces consistently until the hosted-build rotation completes.

### Fixed

- The orderbook retry orchestrator now honors the `Retry-After`
  header on `429 Too Many Requests` and `503 Service
  Unavailable` responses, waiting for the larger of its
  exponential backoff schedule and the server-provided
  cooldown. Both the native `reqwest` adapter and the browser
  `fetch` adapter now surface non-success response headers
  through the typed transport error variant.
- The EIP-1271 verification cache no longer panics on browser
  targets. The cache time source now uses `web_time::Instant`
  on `wasm32-unknown-unknown` and `std::time::Instant` on
  native builds, matching the time-source pattern used across
  the rest of the SDK.
- Published crate READMEs now compile as doctests on every CI run, and the
  previously broken orderbook, trading, and contracts examples match the
  shipped public API.
- Every ECDSA signature leaving the contracts crate now carries the
  Solidity-compatible `27` / `28` marker expected by on-chain
  verification. Signers that emit modern `0` / `1` markers are
  normalized automatically, and any other trailing byte now fails with a
  typed error before downstream `ecrecover` paths can consume it.

### Removed

- Stale WASM build artifacts have been removed from the verification console
  package directory; the per-pkg gitignore now tracks only the canonical
  wasm-pack outputs.
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

### Security

- Both shipped WASM consoles now declare a `Content-Security-Policy` meta tag
  with explicit `script-src` and `connect-src` allowlists.

### Notes

- `0.1.0` will be recorded here when the first functional crates.io release is
  live.

## [0.1.0] - TBD

Placeholder for the first functional crates.io release of the `cow-rs` crate
family. This section will be populated when that release is published.
