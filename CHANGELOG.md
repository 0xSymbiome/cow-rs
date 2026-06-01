# Changelog

All notable changes to `cow-rs` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Semantic versioning begins with the first functional crate release.

Reserved-placeholder `0.0.1-reserved.0` name-reservation publishes are
excluded from this version history.

The first functional crate-family release begins at `0.1.0`.

## [Unreleased]

### Added

- `cow-sdk-wasm` resolves its read and quote methods to typed DTOs across the
  JavaScript surface. `OrderBookClient.getOrder` returns `OrderDto`,
  `getOrders` / `getOrdersByOwner` return `OrderDto[]`, `getTrades` returns
  `TradeDto[]`, `getNativePrice` returns `NativePriceResponseDto`, and
  `getQuote` returns `OrderQuoteResponseDto`; `TradingClient.getQuote` returns
  the fully resolved `QuoteResultsDto`, which `postSwapOrderFromQuote` accepts
  back unchanged (the SDK round-trips the native `cow_sdk_trading::QuoteResults`
  shape), and the swap- and limit-order posting methods resolve to
  `OrderPostingResultDto`. The `OrderDto`, `TradeDto`, `QuoteResultsDto`, and
  `OrderPostingResultDto` DTOs and their nested trees (`OrderDataDto`,
  `TradeParametersDto`, `QuoteAmountsAndCostsDto`, `OrderQuoteResponseDto`,
  `TradingAppDataInfoDto`, and the rest) are emitted into the generated
  TypeScript declarations and re-exported from the facade entry points that
  expose the corresponding orderbook or trading capability, so a consumer can
  name each read, quote, and posting shape instead of an opaque value. The
  declaration snapshots under `crates/wasm/snapshots/` are refreshed in the same
  change set.
- `cow-sdk-wasm` `OrderBookClient` adds `getAppData(appDataHash)` and
  `uploadAppData(appDataHash, fullAppData)`. `getAppData` returns the typed
  `AppDataObjectDto`; `uploadAppData` routes through the native
  `cow_sdk_orderbook` content-addressed-write path (which re-derives and
  verifies `keccak256(fullAppData)` against the supplied hash before dispatch)
  and resolves to `{ uploaded: true }`. Both ship in every package flavour that
  bundles the orderbook capability.
- `cow-sdk-wasm` `TradingClient.buildSellNativeCurrencyTx` accepts its `quoteId`
  as a `number`, aligning the native quote identifier's `i64` boundary with the
  JavaScript number surface used across the client.
- `cow-sdk-wasm` carries the coarse `OrderbookRejectionCategoryDto` as the
  optional `category` field on the JavaScript `WasmError` `orderbook` variant,
  mirroring the native `cow_sdk_orderbook::OrderbookRejection::category()`, and
  maps every `cow_sdk_orderbook::OrderbookError` and
  `cow_sdk_trading::TradingError` into a typed `WasmError` through the shared
  `cow_sdk_core::ErrorClass`. The category carries no message, preserving the
  workspace redaction posture, so a JavaScript host can branch on the action a
  rejection calls for without parsing a message. Governed by
  [ADR 0017](docs/adr/0017-typed-orderbook-rejection-parser.md) and
  [ADR 0060](docs/adr/0060-uniform-error-classification.md).
- At the `cow-sdk-wasm` order-input boundary an omitted `receiver` and an
  explicit zero-address `receiver` resolve to the same pay-to-owner sentinel and
  construct byte-identical `cow_sdk_core::OrderData` — and therefore the same
  EIP-712 struct hash and order UID — while a concrete receiver is never
  rewritten to the owner. Recorded in
  [ADR 0061](docs/adr/0061-wasm-abi-receiver-pay-to-owner.md) and pinned as
  `PROP-WB-022` in `PROPERTIES.md`.
- Every public error type the `cow-sdk` facade aggregates now exposes a
  `class() -> ErrorClass` accessor (`CoreError`, `AppDataError`, `SigningError`,
  `ContractsError`, `OrderbookError`, `TradingError`, and `BrowserWalletError`),
  so a caller holding a bare leaf error can obtain the coarse failure class
  without re-implementing the per-variant match. `ErrorClass` now lives in
  `cow-sdk-core` (re-exported from `cow-sdk`, so `cow_sdk::ErrorClass` is
  unchanged), and `SdkError::class()` delegates to the per-type accessors;
  composite errors delegate to the wrapped error so a wrapped 429 stays
  `ErrorClass::RateLimited`. Governed by
  [ADR 0060](docs/adr/0060-uniform-error-classification.md).
- `cow_sdk::ErrorClass::RateLimited` classifies an orderbook response that
  signalled HTTP 429 after the transport layer's retry budget was exhausted, so
  observers can distinguish an outlasting throttle from a generic remote
  response instead of bucketing both as `ErrorClass::Remote`. The transport
  layer already retries 429s with `Retry-After` honoring, so this class is a
  telemetry signal rather than a control-flow change.
- `cow_sdk_orderbook::OrderbookRejection::category()` returns a coarse,
  action-oriented `OrderbookRejectionCategory` (authorization, insufficient
  funds, invalid order, not found, conflict, unfulfillable, server, or unknown).
  The full typed rejection taxonomy is unchanged; the accessor lets callers
  branch on the action a rejection calls for without matching every wire tag,
  and the category carries no message so it never re-exposes a redacted payload.
  Governed by [ADR 0017](docs/adr/0017-typed-orderbook-rejection-parser.md).
- On `wasm32` targets the `cow-sdk` facade re-exports the browser
  `FetchTransport` and `FetchTransportConfig`, mirroring the native
  `ReqwestTransport` re-export, and exposes `cow_sdk::wasm::pure_helpers` on both
  targets so the host-safe helper path resolves the same way regardless of
  build target.
- `cow_sdk_subgraph::SubgraphError::TransportConfiguration` is an additive
  (`#[non_exhaustive]`) variant carrying the transport classification and a
  `Redacted<String>` detail. It is returned by the native default-transport
  `SubgraphApiBuilder::build` path when constructing the backing
  `ReqwestTransport` fails before any request context exists, and is distinct
  from `SubgraphError::Transport`, which carries per-request context for
  failures observed once a query is in flight.
- `cow_sdk_trading::TradingOptions::with_transport_policy` sets the request
  retry, rate-limit, and HTTP-client policy applied to the orderbook client the
  trading SDK builds on its default construction path. An injected orderbook
  client keeps its own transport policy, so the setting is consulted only when
  the SDK builds the client itself. Governed by
  [ADR 0041](docs/adr/0041-transport-policy-l3-layering.md).
- `cow_sdk_orderbook::OrderCreation::from_signed` is the canonical conversion
  from a signed `cow_sdk_core::OrderData` into a submission payload: it copies
  every signed economic field verbatim, wires the order-level fee as `"0"`, and
  derives the wire `appDataHash` from the signed order's `app_data` so the
  submitted hash cannot diverge from the signed commitment. The managed
  trade-posting path builds the submission body through it.
- `cow_sdk_orderbook::Order::signing_order` projects a fetched order back into
  the `cow_sdk_core::OrderData` used for EIP-712 hashing and UID re-derivation,
  resolving an omitted receiver to the pay-to-owner zero-address sentinel. It
  returns `None` for `EthFlow` orders, whose response fields are rewritten for
  display and therefore cannot reproduce the on-chain digest, so the projection
  fails closed rather than yielding a silently wrong hashing input.
- `cow_sdk_contracts::order_eip712_type_hash()` returns the canonical EIP-712
  `Order` type hash (matching the upstream services `OrderData::TYPE_HASH`) for
  callers pinning or verifying an order digest without reaching for a generated
  codec struct.
- `cow_sdk_orderbook::QuoteData` now mirrors the full orderbook
  `OrderParameters` quote payload: it models the network-cost gas estimates
  `gasAmount`, `gasPrice`, and `sellTokenPrice` (decimal-string wire shape), the
  optional `appDataHash` echo, and the `signingScheme` (defaulting to `eip712`
  and always serialized). The gas estimates are read-only quote values surfaced
  through accessors with no public setter, populated from the `/quote` response
  and omitted from serialization when a locally constructed quote leaves them
  empty. `QuoteData` is enrolled in the OpenAPI coverage manifest as the
  `OrderParameters` mirror, so `openapi-coverage --validate` checks the quote
  payload for field-level fidelity instead of treating it as an opaque object.
  The contract is recorded in
  [ADR 0058](docs/adr/0058-typed-quote-request-response-surface.md) and the
  [Quote Response Surface Audit](docs/audit/quote-response-surface-audit.md).
- A quote-amounts projection parity test
  (`cow-sdk-trading/tests/quote_projection_parity.rs`) locks the signable
  sell/buy amounts derived from a quote response: the sell-driven side folds
  the network cost back into the sell amount and the buy-driven side carries
  the network cost on the sell amount, matching the orderbook quote-amounts
  contract.
- `cow_sdk_contracts::onchain_orders` adds typed `CoWSwapOnchainOrders`
  `OrderPlacement` / `OrderInvalidation` event bindings and a fail-closed,
  provider-free log decoder (`decode_order_placement`,
  `decode_order_invalidation`). The decoder validates the topic set, the
  on-chain signing scheme, the EIP-1271 owner-payload length, the eth-flow
  trailing-data length, and the 56-byte UID length, returning a typed
  `ContractsError` rather than panicking on malformed input;
  `OnchainOrderPlacement` resolves the order owner and derives the 56-byte
  order UID through `compute_order_uid`. Topic-0 is byte-locked against an
  independent keccak of the canonical signatures and the order hash against an
  upstream contract vector. The decoding contract is documented in
  [ADR 0054](docs/adr/0054-onchain-order-event-decoding-is-fail-closed.md) and
  the [On-Chain Order Log Decoding Audit](docs/audit/onchain-order-log-decoding-audit.md).
- `cow_sdk_contracts::settlement` adds typed `GPv2Settlement` event bindings and
  a fail-closed, provider-free log decoder (`decode_settlement_log`) that maps
  `Trade`, `Interaction`, `Settlement`, `OrderInvalidated`, and the inherited
  `GPv2Signing` `PreSignature` logs into the typed `SettlementEvent` enum. The
  decoder validates the topic set and indexed arity through a shared topic guard
  and length-checks the 56-byte order UID, returning a typed `ContractsError`
  rather than panicking on malformed input. The five event topic-0 hashes are
  byte-locked against an independent keccak of the canonical signatures. The
  decoding contract is documented in
  [ADR 0056](docs/adr/0056-settlement-event-decoding-is-fail-closed.md) and the
  [Settlement Event Log Decoding Audit](docs/audit/settlement-event-log-decoding-audit.md).
- `cow_sdk_contracts::eth_flow` adds the `CoWSwapEthFlow` `OrderRefund` event
  binding and a fail-closed `decode_order_refund`, plus a unified
  `decode_eth_flow_log` dispatcher and `EthFlowEvent` enum that decode any
  eth-flow lifecycle log (`OrderPlacement`, `OrderInvalidation`, `OrderRefund`)
  into typed Rust. The decoder validates the topic set and length-checks the
  56-byte order UID, returning a typed `ContractsError` rather than panicking on
  malformed input, and the `OrderRefund` topic-0 is byte-locked against an
  independent keccak of the canonical signature. The decoding contract is
  documented in
  [ADR 0054](docs/adr/0054-onchain-order-event-decoding-is-fail-closed.md) and
  the [On-Chain Order Log Decoding Audit](docs/audit/onchain-order-log-decoding-audit.md).
- `cow-sdk-wasm` adds the `decodeSettlementLog` and `decodeEthFlowLog` exports,
  with `EventLogInput`, `SettlementEventDto`, and `EthFlowEventDto` DTOs, that
  reconstruct borrowed log bytes from hex `topics` / `data` and dispatch to the
  fail-closed, provider-free `cow_sdk_contracts` decoders, returning a versioned
  `WasmEnvelope`. The decoders are deterministic and perform no I/O, so a
  JavaScript host that already holds raw chain logs decodes settlement and
  eth-flow events without network access; malformed input returns a typed
  `WasmError`. The helpers are exposed in every package flavour that bundles the
  signing capability.
- `cow-sdk-core` adds an opt-in `LogProvider: Provider` capability supertrait for
  event-log fetching, with `LogQuery` / `RawLog` / `LogMeta` types whose
  single-call `get_logs` issues exactly one backend query over a caller-bounded
  block range and returns raw logs for the fail-closed decoders, never a watcher
  or indexer loop. `cow-sdk-alloy-provider` implements
  `LogProvider` for `RpcAlloyProvider`. The capability mirrors the
  `SigningProvider` split and leaves `Provider`'s shape frozen; it is documented
  in [ADR 0057](docs/adr/0057-log-provider-capability-trait.md), with the
  trait-evolution rule clarified in
  [ADR 0029](docs/adr/0029-trait-evolution-extension-traits.md) and the
  [Log-Provider Capability Audit](docs/audit/log-provider-capability-audit.md).
- `cow_sdk_contracts::weth` adds the `IWrappedNativeToken` (WETH9-family)
  `deposit` / `withdraw` bindings with `wrap_interaction` / `unwrap_interaction`
  helpers that emit the canonical settlement interaction, and
  `cow_sdk_contracts::eth_flow` gains `parse_eth_flow_onchain_data`, the
  `WRAP_ALL_SELECTOR` constant, and `wrap_all_interaction`. The Solidity
  mirrors for the new surfaces are vendored byte-identically under
  `crates/contracts/abi/eth-flow/` and `crates/contracts/abi/weth/` and gated by
  the provenance contract in
  [ADR 0012](docs/adr/0012-alloy-sol-bindings-and-registry-authority.md).
- `cow_sdk_app_data::AppDataParams` gains two fluent terminal methods:
  `into_doc(self) -> AppDataDoc` produces the canonical JSON
  document without running the embedded schema validator, and
  `into_validated(self) -> Result<AppDataValidated, AppDataError>`
  additionally runs the embedded JSON schema validation and
  computes the CID, canonical JSON content, and keccak256 hex
  digest in a single call. The canonical SDK-attribution chain
  reads `AppDataParams::new(code).with_*(...).into_validated()?`
  end to end, mirroring the workspace `into_X` consumption
  convention precedented by
  `cow_sdk_pure_helpers::AppDataDocInput::into_document`. The free
  functions `cow_sdk_app_data::generate_app_data_doc` and
  `cow_sdk_app_data::get_app_data_info` remain available for
  composed flows such as the typed merge pipeline.

- `cow_sdk_core::AppCode` and `cow_sdk_core::AppCodeError` are the
  canonical home for the validated application-identifier newtype.
  `AppCode` joins `cow_sdk_core::Address`, `cow_sdk_core::AppDataHash`,
  `cow_sdk_core::HexData`, and `cow_sdk_core::OrderUid` as cow-owned
  identity primitives in `cow-sdk-core`. The `cow_sdk` root facade
  prelude exposes both types so the canonical user-facing import is
  `use cow_sdk::AppCode;`. The workspace enum-policy manifest
  records the source-of-truth file in `cow-sdk-core`.

### Fixed

- `cow_sdk_trading::Trading::off_chain_cancel_order` and `on_chain_cancel_order`
  tracing spans now record the effective chain and environment resolved from the
  SDK's trader defaults instead of `None` when the caller supplies an
  `OrderTraderParameters` without them, matching the quote-path spans and the
  `chain` field contract in `docs/observability.md`.
- `cow_sdk_orderbook::OrderQuoteRequest::with_app_data_hash` now produces the
  hash-only quote app-data wire form instead of pairing the requested hash with
  the constructor's placeholder document. `OrderQuoteRequest::new` previously
  seeded the zero app-data hash in the document slot, so attaching an explicit
  hash produced a document-plus-hash body whose document the orderbook
  re-hashes and rejects. The constructor now attaches no app-data by default,
  which the orderbook resolves to the zero app-data hash, and composing a full
  document with an explicit hash still yields the document-plus-hash form
  expected by the orderbook `OrderParameters` contract.
- `cow_sdk_orderbook::QuoteAppData` deserialization now resolves a lone
  `appData` that is itself a 32-byte hash into the hash slot, matching the
  orderbook's own app-data parsing, so a hash-only quote request round-trips
  and its `app_data_hash` and `full_app_data` accessors stay accurate for a
  decoded request.

### Changed

- `cow-sdk-wasm` no longer builds the standalone `web` target for the `default`,
  `orderbook`, and `signing` flavors. Those flavors' facade ESM and CommonJS
  entries are backed by the `bundler` and `nodejs` raw builds, so their
  standalone `web` builds were referenced by no package export; the `web` target
  is retained for the Cloudflare flavor, which needs it for the Worker
  precompiled-module initialization. The published package export map is
  unchanged. Governed by
  [ADR 0044](docs/adr/0044-bundle-size-profile-and-flavor-builds.md).
- `cow_sdk::prelude` no longer re-exports `AppCodeError`; reach it through
  `cow_sdk::core::AppCodeError`. The prelude keeps the `AppCode` value type and
  stays focused on the common quote, sign, post, and error-handling workflow
  rather than carrying leaf validation errors.
- The `cow_sdk_orderbook::OrderbookApiBuilder` and
  `cow_sdk_subgraph::SubgraphApiBuilder` typestate markers now carry the value
  they prove is present (chain id, environment or API key, and transport), so
  the build terminals read each input directly from the type-level marker
  instead of unwrapping an `Option`. The builders construct panic-free: the
  native default-transport `build` path returns a typed error
  (`OrderbookError::Transport` or `SubgraphError::TransportConfiguration`)
  instead of panicking when a configured user-agent cannot be encoded as an
  HTTP header value. `OrderbookApiBuilder::base_url` is now reachable only on
  the environment-set typestate, so calling it before `.environment(...)` or
  `.from_context(...)` is a compile error rather than a runtime panic. The
  public construction API and the private-field marker seal are unchanged, so
  no existing caller needs to migrate.
- Renamed the orderbook client types to single-word `Orderbook` casing for Rust
  idiom consistency with the sibling `cow_sdk_orderbook::OrderbookError`,
  `OrderbookClient`, and `OrderbookRejection` types: `OrderBookApi` is now
  `OrderbookApi`, `OrderBookApiBuilder` is now `OrderbookApiBuilder`, and the
  public transport-error type `OrderBookApiError` is now `OrderbookApiError`.
  The `cow-sdk` facade prelude and module re-exports are updated. The
  TypeScript-callable WASM client class intentionally keeps its `OrderBookClient`
  name to match the JavaScript SDK naming convention at the browser boundary.
- Renamed the trading entry types to drop the `Sdk` suffix stutter:
  `TradingSdk` is now `Trading`, `TradingSdkBuilder` is now `TradingBuilder`,
  `TradingSdkOptions` is now `TradingOptions`, and `HelperOnlySdk` is now
  `TradingHelpers`. Construction remains exclusively through the typestate-builder
  terminals (`TradingBuilder::ready(...)` and `TradingBuilder::helper_only(...)`);
  the `cow-sdk` facade prelude re-exports are updated to the new names.
- `cow_sdk_orderbook::OrderbookError::Serialization` now carries a structured
  `{ category, line, column }` triple instead of wrapping the raw
  `serde_json::Error`. The orderbook client surfaces only the serde failure
  category (`syntax`, `data`, `eof`, or `io`) and the 1-based structural
  position of a response-decode failure, so a malformed or unexpected orderbook
  response body can no longer echo decoded bytes through the error's `Display`
  or `Debug` surface (ADR 0025). Construction stays ergonomic through
  `From<serde_json::Error>`.
- `cow_sdk_app_data::AppDataError::Calculation` renders only the stable
  `appDataHex calculation failed` label through `Display` and JSON
  serialization; the typed source stays reachable through
  `std::error::Error::source` for callers that deliberately cross the redaction
  boundary, so a future hashing or CID backend cannot leak caller-derived bytes
  through the default error surface (ADR 0025).
- Malformed `metadata.signer`, `metadata.flashloan`, and `metadata.hooks`
  values in an app-data document now surface a fixed, field-tagged validation
  message that names only the public wire key, never the caller-supplied key or
  value, keeping the app-data document parser inside the credential-redaction
  convention (ADR 0025).
- Renamed the canonical signed-order payload `cow_sdk_core::UnsignedOrder` to
  `cow_sdk_core::OrderData`, aligning the Rust SDK's name with the upstream
  services `model::order::OrderData` it mirrors byte-for-byte. The rename is
  wire-neutral: the EIP-712 field set, type hash, and serialized shape are
  unchanged.
- `cow_sdk_trading::OrderBoundsValidator::validate` now takes the signing order
  (`cow_sdk_core::OrderData`) plus its submission owner (`from: Address`)
  instead of a fully built `cow_sdk_orderbook::OrderCreation`. The validator
  only inspects the order's economic fields and the owner, so the eth-flow
  submission path no longer fabricates a throwaway `OrderCreation` with an empty
  signature solely to validate. The `ClientRejection` variants and every
  enforced invariant are unchanged.
- The remaining `cow-sdk-contracts` order encode and decode surfaces take the
  concrete `cow_sdk_core::OrderData` in place of the removed contracts-crate order
  types: `SettlementEncoder::encode_trade`, the `encode_trade` codec free function,
  and the swap-encoder `encode_trade` accept `&OrderData`; `decode_order` returns
  `OrderData`; and the `cow_sdk_contracts::OnchainOrderPlacement.order` field is an
  `OrderData`. The encoded settlement calldata, the decoded values, and the
  on-chain event shape are unchanged — only the order type moves.
- `cow-sdk-orderbook` solver-competition reads now target the orderbook `v2`
  routes (`/api/v2/solver_competition/{auctionId}`, `/by_tx_hash/{txHash}`, and
  `/latest`) and decode into a fully typed `SolverCompetitionResponse`.
  Addresses, amounts, order UIDs, and transaction hashes use the workspace
  domain newtypes; the response now carries the per-solver `referenceScores`
  and each solution's touched `orders` (a new `SolverCompetitionOrder` type)
  rather than dropping them. A shared `AuctionPrices`
  (`BTreeMap<Address, Amount>`) types the clearing- and reference-price maps,
  and `EthflowData::refund_tx_hash` is now a typed `TransactionHash`.
- `cow_sdk_orderbook::OrderQuoteRequest` now models its quote `oneOf`s as typed
  Rust so an invalid request is unrepresentable rather than rejected at
  validation time, mirroring the orderbook quote schema. The mutually exclusive
  `valid_for`/`valid_to` fields become a `QuoteValidity` (`ValidTo` xor
  `ValidFor`); the `QuoteSide` struct becomes the `OrderQuoteSide` enum with a
  `SellAmount` distinguishing the before-fee and after-fee sell amount; and the
  flat `signing_scheme`/`onchain_order`/`verification_gas_limit` fields become a
  `QuoteSigningScheme` enum that keeps the verification gas limit on EIP-1271
  only and makes an ECDSA on-chain order unrepresentable (rejected on the wire by
  a `try_from` guard). The `with_valid_to`/`with_valid_for`/`with_signing_scheme`/
  `with_onchain_order`/`with_verification_gas_limit` builder methods are retained
  and now drive the typed fields. Recorded in
  [ADR 0058](docs/adr/0058-typed-quote-request-response-surface.md) and the
  [Quote Response Surface Audit](docs/audit/quote-response-surface-audit.md).
- Fixed: a hash-only quote request now serializes the app-data hash under the
  `appData` key (the orderbook's accepted `Hash` form) instead of an
  `appDataHash`-only body that the orderbook rejected with `invalid app data`.
  `OrderQuoteRequest` app-data is encapsulated as `QuoteAppData` and routed
  identically to the signed `OrderCreation` payload, so every form (full / hash /
  both) is wire-correct.
- The quote request now defaults `priceQuality` to `PriceQuality::Optimal`
  (previously `Verified`). `Optimal` is the quote mode used for a quote that
  will be signed and submitted: the orderbook returns a quote identifier for
  order placement, and `Optimal` is the orderbook's own default quote quality.
  The value is always serialized, so the wire request is explicit regardless
  of the default.
- The orderbook, subgraph, and IPFS clients run every HTTP attempt through
  one shared retry driver, `cow_sdk_transport_policy::run_with_retry`, rather
  than three hand-rolled retry loops. The driver owns the attempt loop,
  rate-limit acquisition, exponential backoff, `Retry-After` honoring, and
  retry telemetry, and is generic over the success payload and the caller's
  error type through the `AttemptOutcome`, `RetrySignal`, and `LimiterKey`
  surfaces. A non-retryable transport class returns immediately instead of
  re-dispatching the request until the attempt limit is exhausted.
  Retry-delay computation reads a target-neutral wall clock,
  `cow_sdk_transport_policy::system_now`, so an HTTP-date `Retry-After`
  evaluates against the current time on both native and `wasm32` targets and
  the retry path no longer aborts a browser runtime through the standard
  `SystemTime::now`. See
  [ADR 0041](docs/adr/0041-transport-policy-l3-layering.md).

- The EIP-1271 verification cache is now a positive-only set keyed on the
  full `(verifier, digest, signature_hash)` probe identity. The
  `cow_sdk_contracts::Eip1271VerificationCache` trait replaces its
  `get` / `put` methods with `contains_valid` / `record_valid`:
  `verify_eip1271_signature_cached` folds `keccak256(signature)` into the
  cache key and records only successful magic-value matches, so a probe
  carrying a different signature on the same digest can never be served a
  verdict recorded for another signature, and a magic-value mismatch is
  never cached (a miss means "unknown", never "known invalid"). The
  trait and the dependency-free `NoopEip1271VerificationCache` stay always
  available; the in-memory `InMemoryEip1271VerificationCache` and the
  `parking_lot` / `web-time` dependencies it requires now ship behind the
  new default-off `in-memory-cache` feature on `cow-sdk-signing` and the
  `cow-sdk` facade. See
  [ADR 0014](docs/adr/0014-eip1271-verification-cache.md).

- `cow_sdk_core::Amount` and `cow_sdk_core::SignedAmount` use a checked
  arithmetic surface — `checked_add` / `checked_sub` / `checked_mul` /
  `checked_pow` (returning `Option`) and explicit `saturating_*` clamps
  — rather than bare `Add` / `Sub` / `Mul` operators or a `pow` method,
  so an overflow or underflow cannot silently wrap a typed amount.
  Callers that need raw wrapping use `as_u256` / `into_u256`
  (respectively `as_i256` / `into_i256`).
- `cow_sdk_app_data::get_app_data_info` surfaces the underlying
  JSON-schema validator detail on validation failure. The lossy
  `AppDataError::InvalidAppDataProvided { reason: BadShape { details: "document failed the embedded JSON schema validation" } }`
  envelope is replaced with the typed `AppDataError::Schema { message, source }`
  variant carried directly from the validator boundary. The
  `message` field is now plaintext `String` (was `Redacted<String>`)
  because the rendering is safe-by-construction: instance values
  flow through the underlying validator's masking surface and
  rejected-property-name lists (Draft-7 `additionalProperties: false`
  failures) are rendered as counts rather than names so caller
  content cannot leak through `Display`. The typed
  `jsonschema::ValidationError` source is preserved through the
  `#[source]` chain for callers that need the unmasked rendering
  and explicitly cross the redaction boundary by walking
  `std::error::Error::source`. The `ValidationResult::errors` field
  tightens symmetrically from `Option<Redacted<String>>` to
  `Option<String>` because it carries the same safe-by-construction
  text.

- `cow_sdk_app_data::AppDataParams` exposes a single typed
  construction surface. `AppDataParams::new(app_code: AppCode)`
  replaces the prior five-argument constructor and matches the
  `new()`-as-primary-constructor convention shared by every
  identity newtype in `cow-sdk-core`. The `app_code` field
  itself is now typed as `Option<AppCode>` so the struct
  cannot hold an unvalidated value, the loose
  `with_app_code(impl Into<String>)` setter is removed, and the
  remaining `with_environment` / `with_signer` / `with_flashloan` /
  `with_hooks` / `with_metadata` chain stays unchanged. The trading
  crate's `build_app_data` helper accepts `&AppCode` so validation
  propagates with the typed value rather than relying on
  `Deref<Target = str>` coercion at the boundary. The typed merge
  pipeline (`merge_and_seal_app_data`, `params_from_doc`,
  `merge_app_data_params`) continues to produce and consume
  `AppDataParams` over the same wire shape; only the
  application-identifier slot tightens from `Option<String>` to
  `Option<AppCode>`. The wire form of `appCode` is unchanged
  because `AppCode` serializes as its inner string.

- `cow_sdk_app_data::generate_app_data_doc` interpolates the
  `cow_sdk_app_data::DEFAULT_APP_CODE` constant in place of the
  duplicated `"CoW Swap"` literal at the fallback site so a future
  change to the documented default flows through a single
  authoritative declaration. The fallback value is unchanged.

### Removed

- Removed the client-side IPFS upload seam from `cow-sdk-app-data`: the
  `pin_json_in_pinata_ipfs` helper, the `IpfsUploadTransport` trait, the
  `TransportResponse` type, the `DEFAULT_IPFS_WRITE_URI` constant, the
  `write_uri`, `pinata_api_key`, and `pinata_api_secret` fields on `IpfsConfig`,
  and the `AppDataError::Pinning` and `AppDataError::MissingIpfsCredentials`
  variants. Registering an app-data document is orderbook-mediated: hash the
  document locally with `get_app_data_info`, then submit the full document
  through the orderbook crate's `upload_app_data` content-addressed-write path,
  which stores it under its hash in the orderbook. The IPFS read seam
  (`IpfsFetchTransport`, `fetch_doc_from_cid`, `fetch_doc_from_app_data_hex`) is
  unchanged.
- Removed the unused `cow_sdk_core::TradeModel` alias for `Trade`; it had no
  consumers, so use `cow_sdk_core::Trade` directly.
- Removed the deprecated `availableBalance` field from the
  `cow_sdk_orderbook::Order` response DTO. The orderbook OpenAPI marks it
  deprecated and documents it as unused, always `null`, and slated for removal
  upstream. The field is now ignored on deserialization and never re-emitted; a
  response that still carries it round-trips without it.
- Removed the unused `cow_sdk_app_data::IpfsUploadResult` type. It was never
  constructed, and its documented contract — an app-data digest derived from the
  Pinata upload CID — is not satisfiable: Pinata returns a sha2-256 CIDv0, which
  is not the keccak-256 CIDv1 app-data identifier and is rejected by
  `cid_to_app_data_hex`. The canonical app-data hash comes from
  `get_app_data_info`.
- Removed the unused `cow_sdk_core::Order` envelope (the
  `{ unsigned, owner, uid }` wrapper around `OrderData`); it had no
  constructor caller, reader, or conversion, and no upstream analog. The bare
  `Order` re-export was also dropped from the `cow-sdk` prelude. Reach the
  order types by module path instead: `cow_sdk::core::OrderData` (the EIP-712
  signing and hashing input) and `cow_sdk::orderbook::Order` (the response
  record).
- Removed the `cow_sdk_contracts::Order` and `cow_sdk_contracts::NormalizedOrder`
  order types and the generated `cow_sdk_contracts::GPv2Order` `sol!` struct from
  the public API. EIP-712 order hashing and UID derivation now operate directly
  on the concrete `cow_sdk_core::OrderData`: `hash_order`, `compute_order_uid`,
  and the cancellation hashers take `&OrderData`, and the canonical type hash is
  exposed through `order_eip712_type_hash()`. The contracts crate no longer
  defines its own order type — the macro-emitted EIP-712 codec struct is
  crate-internal machinery with no consumer journey, and the optional-to-concrete
  normalization step it required is gone because a concrete order maps straight
  onto the codec layout. A `receiver` of `address(0)` is hashed verbatim as the
  protocol's pay-to-owner sentinel; the eth-flow construction path keeps its own
  `ContractsError::ZeroReceiver` guard. Recorded in
  [ADR 0059](docs/adr/0059-hash-concrete-orderdata-directly.md).
- Removed `OrderBookApi::get_auction` and the `Auction` response type. The
  `/api/v1/auction` endpoint is not reachable for public clients and upstream
  treats it as a liveness probe rather than a data feed. With no public auction
  feed, the `AuctionOrder` response type and its auction-side `Quote` had no
  reachable producer and are removed as well, collapsing the order-shaped
  response surface to the single `Order` type. Auction retrieval, the
  `AuctionOrder` mirror, and its quote can return as an additive change if the
  endpoint becomes publicly consumable.
- The `cow_sdk_trading` quote cache is removed: the `QuoteCache` trait,
  its `QuoteCacheKey`, the `NoopQuoteCache` and `InMemoryQuoteCache`
  implementations, the `TradingSdkBuilder::with_quote_cache` /
  `TradingSdkOptions::with_quote_cache` setters, and the
  `DEFAULT_QUOTE_CACHE_TTL` / `DEFAULT_QUOTE_CACHE_CAPACITY` constants. The
  seam was never consulted by the quote flow, its key omitted
  quote-determining inputs (the effective app-data document and the
  price-quality variant), and a quote's economic value is too
  time-sensitive to memoize behind a fixed TTL without an authoritative
  on-chain re-check. This also drops the `parking_lot` dependency from
  `cow-sdk-trading`.

- `cow_sdk_trading::AppCode`, `cow_sdk_trading::AppCodeError`, and
  the trading-crate `crates/trading/src/types/app_code.rs` module
  are removed. The canonical types live in `cow_sdk_core` and the
  `cow_sdk` facade re-exports them at the root. Imports update to
  `use cow_sdk::{AppCode, AppCodeError};` or
  `use cow_sdk_core::{AppCode, AppCodeError};`.

- `cow_sdk_app_data::AppDataParams::with_app_code` is removed.
  Construct application-tagged parameters through
  `AppDataParams::new(AppCode::new(value)?)` so the validation seam
  rejects malformed identifiers at the boundary rather than
  silently accepting an unchecked string.

- `cow_sdk_contracts::normalized_ecdsa_signature` is removed.
  Recoverable ECDSA signatures are constructed exclusively through
  the closed-construction `cow_sdk_contracts::RecoverableSignature`
  typestate described below; `RecoverableSignature::to_hex_string`
  provides the hex-encoded wire form when the legacy 65-byte hex
  shape is required.

- `cow_sdk_trading::TradingSdkBuilder::with_owner`,
  `cow_sdk_trading::PartialTraderParameters::owner`, and
  `cow_sdk_trading::PartialTraderParameters::with_owner` are removed.
  The SDK no longer stores a default owner; per-call
  `TradeParameters.owner` and `LimitTradeParameters.owner` (with the
  signer's address as the implicit fallback for signer-backed flows,
  or `TradeAdvancedSettings::quote_request.from` for quote-only
  flows) are the sole owner source. ADR 0011 carries the new
  Must-Remain-True bullet recording the per-trade owner placement,
  with the
  [Trading SDK Runtime Prerequisites Audit](docs/audit/trading-sdk-runtime-prerequisites-audit.md)
  and the
  [Trade-Parameter Lifecycle Audit](docs/audit/trade-parameter-lifecycle-audit.md)
  as the standing current-state proofs.

### Changed

- `cow_sdk_subgraph::SubgraphError`'s `Display` rendering carries
  plaintext structural diagnostic on every variant. The
  `GraphQl` variant additionally surfaces `errors.len()` and, when
  present, the first GraphQL error's first source location formatted
  as `at line:column` through a new private
  `first_graphql_location_suffix` helper. The `Transport`,
  `HttpStatus`, `MissingData`, and `Serialization` variants gain
  `chain {chain_id}` in their templates; `Serialization` additionally
  surfaces the redacted response body's byte count derived from
  `body.as_inner().len()`. The free-form `errors[].message`,
  `context.api`, `body`, and `details` payloads remain behind the
  workspace `Redacted<T>` wrapper and continue to render as the
  workspace redaction placeholder. The exact format string is not a
  stability contract; consumers needing structured access pattern-match
  on the typed variant fields. The pairing rule and the non-tautology
  invariant are pinned by
  [`crates/subgraph/tests/error_contract.rs`](crates/subgraph/tests/error_contract.rs)
  (eleven Display-contract cases) and by
  `crates/sdk/tests/error_redaction_contract.rs::subgraph_display_carries_plaintext_structural_diagnostic`,
  governed by
  [ADR 0025](docs/adr/0025-workspace-url-redaction-convention.md), with the
  [Subgraph Error Display Audit](docs/audit/subgraph-error-display-audit.md)
  as the standing current-state proof.

- `cow_sdk_orderbook::OrderBookApi::upload_app_data` verifies the
  content-addressed-write invariant at both boundaries. The
  inherent method now decodes the `PUT /api/v1/app_data/{hash}`
  response body as `cow_sdk_core::AppDataHash` (a bare hex
  string) for both HTTP 200 (already-existing document) and
  HTTP 201 (newly stored) outcomes, matching the services PUT
  response schema; the legacy `AppDataObject` envelope decode is
  removed from the upload path and `upload_app_data` now returns
  `Result<(), OrderbookError>` on both the inherent method and
  the `cow_sdk_orderbook::OrderbookClient` trait. Before
  dispatching, the SDK re-derives the digest through the new
  `cow_sdk_core::AppDataHash::from_full_app_data(&str)` helper
  and rejects with `OrderbookError::AppDataHashMismatch`
  carrying `{ expected, observed, stage }` when the
  caller-supplied hash disagrees with `keccak256(full_app_data)`;
  the new non-exhaustive `cow_sdk_orderbook::HashMismatchStage`
  enum carries `ClientPrecheck` (no network call) and
  `ServerEcho` (server returned a different hash than what was
  sent) so callers can branch on the disagreement source. The
  new variant is distinct from
  `cow_sdk_orderbook::OrderbookRejection::AppDataHashMismatch`,
  which remains the services-emitted 400-class envelope detected
  server-side. The bare-hex PUT response shape is locked by the
  new `parity/fixtures/orderbook/app_data_upload_response.json`
  regression fixture and the
  `crates/orderbook/tests/parity_contract.rs::app_data_upload_response_fixture_decodes_as_app_data_hash`
  case; coverage in `crates/orderbook/tests/api_contract.rs`
  grows with three rows (client-precheck no-network,
  server-echo mismatch, status-200 already-existing document)
  and
  `crates/orderbook/tests/error_variant_shape.rs::app_data_hash_mismatch_carries_typed_hashes_and_stage_discriminator`
  pins the typed shape of the new variant. `AppDataObject`
  remains the GET response wrapper for `get_app_data`. The
  content-addressed-write invariant is recorded as `PROP-ORD-011`
  in `PROPERTIES.md` and `docs/verification-matrix.md` carries
  the matching evidence row, governed by
  [ADR 0017](docs/adr/0017-typed-orderbook-rejection-parser.md)
  and
  [ADR 0031](docs/adr/0031-wire-dto-openapi-driven-with-order-auction-order-split.md).

- `cow_sdk_contracts` exposes one closed-construction typestate for
  recoverable ECDSA signatures. `RecoverableSignature` holds an
  `alloy_primitives::Signature` behind a private field and accepts
  only inputs whose trailing recovery byte is in `{0, 1, 27, 28}`;
  the wider alloy parity-normalisation input range (which admits the
  EIP-155 chain-encoded `v >= 35` family) is rejected through the
  typed `ContractsError::InvalidSignatureRecoveryByte` variant. The
  constructors `RecoverableSignature::parse_hex`,
  `RecoverableSignature::parse_bytes`, and
  `RecoverableSignature::parse_erc2098` are the sole construction
  paths; canonical serialisation through `to_bytes` /
  `to_hex_string` emits the legacy `r || s || (27 + y_parity)` byte
  layout via `alloy_primitives::Signature::as_bytes`; scheme-aware
  recovery rides on the same value through
  `RecoverableSignature::recover(digest, scheme)`, which applies the
  canonical EIP-191 prehash internally for
  `cow_sdk_contracts::SigningScheme::EthSign` and recovers
  `cow_sdk_contracts::SigningScheme::Eip712` against the supplied
  digest directly; opt-in BIP-62 low-s canonicalisation is available
  through `RecoverableSignature::canonicalized_low_s`; and the
  ERC-2098 compact 64-byte form round-trips through `to_erc2098` /
  `parse_erc2098`. Every signing, alloy-signer, and WASM consumer
  routes through the typestate;
  `cow_sdk_contracts::Signature::recover_ecdsa_address` delegates
  through `RecoverableSignature::parse_hex(...)?.recover(...)`. The
  never-swap fence at
  `.github/workflows/never-swap-gates.yml#gate-ecdsa-v` widens to
  forbid `Signature::from_raw` and `Signature::as_rsy` in the
  contracts and signing trees so the wider alloy
  parity-normalisation surface cannot return through a future call
  site. ADR 0022 carries a new amendment block recording the
  typestate construction, with the
  [ECDSA Signature Normalization Audit](docs/audit/ecdsa-signature-normalization-audit.md)
  as the standing current-state proof.

- `cow_sdk_trading::LimitTradeParametersFromQuote` is a real newtype
  around `LimitTradeParameters` that guarantees a non-`None`
  `quote_id` by construction. The prior transparent type alias is
  removed. The newtype is produced exclusively by
  [`swap_params_to_limit_order_params`](crates/trading/src/order.rs)
  and accepted by
  [`post_sell_native_currency_order`](crates/trading/src/post/native.rs)
  and [`get_eth_flow_transaction`](crates/trading/src/onchain.rs) on
  their public entries, lifting the prior `MissingQuoteId` runtime
  check on the `EthFlow` path to a compile-time guarantee at the
  public boundary. `TradingError::MissingQuoteId("EthFlow order posting")`
  remains the diagnostic for callers that explicitly attempt
  construction from a value missing a quote id. The public accessor
  `quote_id()` returns `i64` without an `Option`, `as_limit()` and
  `into_limit()` provide reference and owned access to the
  underlying `LimitTradeParameters`, and `AsRef<LimitTradeParameters>`
  is implemented for ergonomic interop. ADR 0011 carries a new
  Must-Remain-True bullet recording the lifecycle distinction and
  the newtype invariant, with the
  [Trade-Parameter Lifecycle Audit](docs/audit/trade-parameter-lifecycle-audit.md)
  as the standing current-state proof.

- `cow_sdk_trading` exposes one advanced-settings bundle accepted by
  every public post and quote entry. `TradeAdvancedSettings` carries
  `quote_request`, `app_data`, `additional_params`, and
  `slippage_suggester`. Limit-order callers leave
  `slippage_suggester` as `None`; the limit submission path does not
  apply slippage in the same shape as swaps and the field is
  documented but unused on that flow. The wasm export surface follows
  the same single-type shape.

- `cow_sdk_trading::TradeParameters` and
  `cow_sdk_trading::LimitTradeParameters` share their common `with_*`
  setter bodies through one internal definition that emits inherent
  methods on each public type. Public API shape is preserved: every
  setter remains an inherent method on each public type with the
  same signature, the same `#[must_use]`, the same `const fn`
  qualifier, and the same rustdoc text.
  `cow_sdk_trading::LimitTradeParameters::with_quote_id` remains an
  inherent method on the limit type because it is limit-only.

- `cow_sdk_core` exposes a single async trait family for the signer
  and provider boundaries: `Signer`, `Provider`, `SigningProvider`,
  `Owner`, `TypedDataSigner`, `DigestSigner`, and `Eip1193`. The
  unsuffixed names carry the async contract, matching the Alloy
  convention. Signer creation lives on the
  [`SigningProvider`](crates/core/src/traits/provider.rs) extension
  trait so read-only providers stay free of signer dependencies.

- `cow_sdk_signing` and `cow_sdk_contracts` expose one async entry
  per public signing or verification operation: `sign_order`,
  `sign_order_with_scheme`, `sign_order_cancellation`,
  `sign_order_cancellation_with_scheme`, `sign_order_cancellations`,
  `sign_order_cancellations_with_scheme`, `verify_eip1271_signature`,
  and `ensure_contract_code`. The trading-level helper
  `cow_sdk_trading::post::verify_eip1271_order_signature` is async on
  the same shape. The cached EIP-1271 verifier ships as
  `verify_eip1271_signature_cached` alongside the uncached helper in
  the same crate.

- `cow_sdk_trading` ships one async entry point per public operation:
  `post_swap_order`, `post_limit_order`, `post_swap_order_from_quote`,
  `post_cow_protocol_trade`, `post_sell_native_currency_order`,
  `off_chain_cancel_order`, `cancel_order_onchain`,
  `onchain_cancellation_transaction`, `get_pre_sign_transaction`,
  `get_eth_flow_transaction`, `get_quote_results`,
  `get_cow_protocol_allowance`, and `approve_cow_protocol`. Each
  function is `pub async fn` and accepts any signer that implements
  `cow_sdk_core::Signer`. The trait method set is the one pinned by
  [ADR 0029](docs/adr/0029-trait-evolution-extension-traits.md).
  Cooperative cancellation composition through
  `cow_sdk_core::Cancellable::cancel_with(&token)` is supported on
  every entry. Tracing span endpoint fields use
  `trading.post_swap_order`, `trading.post_swap_order_from_quote`,
  `trading.post_limit_order`,
  `trading.post_sell_native_currency_order`,
  `trading.get_quote_results`, `trading.off_chain_cancel_order`,
  `trading.on_chain_cancel_order`, `trading.get_pre_sign_transaction`,
  `trading.get_cow_protocol_allowance`, and
  `trading.approve_cow_protocol`. Browser-wallet flows per
  [ADR 0040](docs/adr/0040-wallet-provider-callback-boundary-for-js-consumers.md)
  bind on `Signer` and the wasm-bindgen surface (`postSwapOrder`,
  `postLimitOrder`, `getCowProtocolAllowance`, and the rest) keeps
  its JS contract.

- `cow_sdk_trading` removes the public bounds-customization surface.
  `TradingSdkBuilder::with_order_bounds` and its paired getter are
  removed; the SDK's stored bounds field is removed. The three
  module-level submission helpers no longer carry `order_bounds`
  parameters and the matching companion functions
  (`post_swap_order_with_bounds`, `post_limit_order_with_bounds`,
  `post_swap_order_from_quote_with_bounds`) are deleted in the same
  change. `OrderBoundsValidator` continues to run at the reviewed
  `OrderValidityBounds::SERVICES_DEFAULT` policy on every public
  submission seam; the typed `ClientRejection` channel is unchanged.
  The new public constructor
  `OrderBoundsValidator::services_default_for_chain(chain_id)`
  builds a validator with the chain-specific wrapped-native-token
  address attached for the same-token paired guard.
  `OrderBoundsValidator::services_default` and
  `OrderBoundsValidator::with_weth_address` remain the public
  validator constructors. See the
  [ADR 0015](docs/adr/0015-client-side-order-bounds-validator.md)
  amendment.

- `cow_sdk_trading::TradeParameters` and
  `cow_sdk_trading::LimitTradeParameters` carry the protocol-level
  fields only. The `sell_token_decimals` and `buy_token_decimals`
  fields and their positional `u8` arguments on `::new` are removed,
  and the same fields are removed from the wasm input DTOs
  `cow_sdk_wasm::SwapParametersInput` and
  `cow_sdk_wasm::LimitTradeParametersInput`. The generated TypeScript
  declaration snapshots are refreshed in the same change set.
  `cow_sdk_core::DecimalAmount` remains the canonical
  typed-amount-boundary home for token decimals across display and
  user-input flows per
  [ADR 0011](docs/adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md).
  Callers that previously passed positional `u8` decimal arguments
  drop those arguments at the call site.

- `cow_sdk_trading::InMemoryQuoteCache::new` now takes
  `(ttl, capacity)` instead of `(ttl)`. The previous unbounded posture
  grew the cache map without limit on long-running sessions; the new
  constructor enforces a capacity bound and an oldest-first eviction
  policy by insertion timestamp. Callers who want the prior default
  call `InMemoryQuoteCache::default()` (5-minute TTL,
  `DEFAULT_QUOTE_CACHE_CAPACITY` of 256 entries); callers who want a
  tighter or wider bound pass an explicit `usize`. The
  `Mutex<HashMap>` storage is replaced with
  `parking_lot::RwLock<HashMap>`, which removes the manual
  `PoisonError::into_inner` recovery path without changing observable
  lookup or insert semantics. Lazy expiry on lookup remains the
  observable contract.

- `AppDataHash::to_cid` now produces the canonical `CIDv1` multibase
  string through `cid::Cid::new_v1` and `multihash::Multihash::wrap`
  instead of the prior byte-stitched hand-roll. The output is
  byte-identical to the prior form on every input. Two new `.expect`
  call sites are accepted as statically infallible by the type
  invariant of `AppDataHash` and the unconditional support of
  `Base16Lower` multibase encoding; both sites are recorded in the
  canonical panic-allowlist with inline safety comments naming the
  invariant.

### Added

- `cow_sdk_trading::cache` now ships a capacity-bounded, TTL-respecting
  `InMemoryQuoteCache` that mirrors the cache primitive pattern
  [ADR 0014](docs/adr/0014-eip1271-verification-cache.md) established
  for `InMemoryEip1271VerificationCache`. The cache exposes a `Clock`
  trait with a default `SystemClock` and a blanket `Fn() -> Instant`
  impl, a `with_clock` constructor for deterministic TTL tests, `ttl`,
  `capacity`, `len`, `is_empty`, and `clear` accessors, and a `Default`
  impl that uses `DEFAULT_QUOTE_CACHE_TTL` (5 minutes) and
  `DEFAULT_QUOTE_CACHE_CAPACITY` (256 entries). Storage is
  `parking_lot::RwLock<HashMap<QuoteCacheKey, _>>` and eviction is
  oldest-first by insertion timestamp on every insert past the
  capacity bound. `wasm32-unknown-unknown` support is covered by the
  new `crates/trading/tests/wasm_cache_contract.rs` contract test that
  mirrors the signing-side wasm cache contract.

- `AppDataHash::try_from_cid(&str) -> Result<Self, CoreError>` parses a
  canonical `CIDv1` multibase string back into the cow newtype,
  completing the round-trip seam between the app-data hash and the
  canonical CID string form. The accepted shape is `CIDv1`, raw codec
  (`0x55`), keccak-256 multihash (`0x1b`), 32-byte digest,
  multibase-encoded in lowercase base16. Every other shape is rejected
  through the new `CoreError::InvalidCid` variant.

- `CoreError::InvalidCid` variant on the `cow-sdk-core` error enum
  surfaces the typed rejection from `AppDataHash::try_from_cid`. The
  variant is additive on the existing `#[non_exhaustive]` enum.

- `cid 0.11.3` and `multihash 0.19.3` are promoted from per-crate pins
  on `cow-sdk-app-data` to workspace dependencies, so the workspace's
  CID stack now resolves through a single source of truth. Both
  `cow-sdk-core` and `cow-sdk-app-data` consume the shared pins; future
  bumps land atomically across the two crates.

- `.github/workflows/encode-prefixed-grep-gate.yml` adds two CI grep
  gates that mechanically fence the `alloy_primitives::hex::encode_prefixed`
  canonical contract from
  [ADR 0052](docs/adr/0052-alloy-primitives-canonical-primitive-layer.md).
  The first job rejects any production-source
  `format!("0x{}", alloy_primitives::hex::encode(...))` hand-roll; the
  second rejects unqualified `use alloy_primitives::hex::encode`
  imports in production sources so the call-site regex's coverage
  envelope stays honest. Both jobs filter `//`-prefixed lines so
  doc-comment narratives that name the forbidden symbol cannot
  self-trigger them. The
  [Dependency Gate Audit](docs/audit/dependency-gate-audit.md)
  validation-surface block enumerates the gate alongside the existing
  release-gating commands.

- `docs/alloy-doctrine.md` is published as the canonical human-readable
  consolidation of the cow-rs ↔ alloy classification. The doctrine
  documents the three-bucket rule (ALWAYS-ALLOY, COW-OWNED,
  BOUNDARY-ADAPTER), the decision tree for assigning a new primitive
  to a bucket, six worked examples (EIP-2930 access lists, adding a
  new chain, a new wallet provider, EIP-4844 blob transactions,
  post-quantum signing, and an alloy major U256 API change), and the
  canonical roster of nine never-swap exceptions. The doc is
  read-only over the ADR set; doctrine evolution requires an ADR
  amendment plus a refresh of the supporting-ADR lists in
  `.github/config/principle-adr-map.yaml`. `docs/README.md` and
  `docs/principles.md` cross-link the doctrine under "Focused
  Reviews And Design History", "Chain-RPC Runtime Neutrality", and
  "Canonical Contract Bindings".

- `.github/workflows/never-swap-gates.yml` adds eight CI grep gates
  that mechanically fence the never-swap surfaces: ECDSA `v`
  normalization (ADR 0022), `Amount` and `SignedAmount` radix
  sniffing (ADR 0052), `Address::Display` lowercase (ADR 0052),
  the `alloy-chains` workspace-dependency ban (ADR 0005, ADR 0011),
  `TypedDataDomain` DTO field shape (ADR 0052, ADR 0040), the
  EIP-1271 Shape A vs Shape B encoder distinctness (ADR 0050), the
  REST transport versus alloy JSON-RPC fence (ADR 0010, ADR 0019,
  ADR 0041, ADR 0046), and a census gate that locks the count of
  inline `DO NOT SWAP` comment blocks at ten. The five
  source-scanning gates filter out `//`-prefixed lines so the
  explanatory text inside the `DO NOT SWAP` blocks does not
  self-trigger them.

- Ten inline `DO NOT SWAP` comment blocks anchor the doctrine at the
  load-bearing call sites across `crates/contracts/src/signature.rs`
  (above `RecoverableSignature::parse_bytes`),
  `crates/core/src/types/amount.rs`
  (paired blocks above `Amount::new` and `SignedAmount::new`),
  `crates/core/src/types/identity.rs` (above `impl fmt::Display for
  Address`), `crates/core/src/config/chains.rs` (paired blocks above
  `SupportedChainId` and `api_path`),
  `crates/core/src/traits/typed_data.rs` (above
  `pub struct TypedDataDomain`),
  `crates/signing/src/eip1271/sol_types.rs` (above the `sol!`
  block),
  `crates/transport-policy/src/retry_after.rs` (above
  `parse_retry_after`), and `crates/transport-wasm/src/fetch.rs`
  (module-level, anchoring the `AbortController` lifecycle). Each
  block cites the binding ADR, the corresponding doctrine row, and
  the CI gate that mechanizes the fence.

- New CI gate `cargo parity-verify-sol-provenance` enforces a
  byte-identity contract on every `.sol` file under
  `crates/contracts/abi/`. All 40 shipped files are byte-identical
  mirrors of a single upstream source pinned in
  `parity/source-lock.yaml`; each `vendored:` manifest row carries the
  upstream path under the repository root and the SHA-256 of the
  upstream bytes at the pinned commit. The gate rejects any drift
  between the on-disk SHA and the manifest SHA, and (when run with
  `--upstream-root <path>`) any drift between the manifest SHA and the
  live upstream bytes at the pinned commit via `git show <commit>:<path>`
  against a local upstream checkout. A second cross-check mode
  (`--upstream-github`) fetches each `vendored:` row from
  `https://raw.githubusercontent.com/<owner>/<repo>/<commit>/<upstream-path>`
  (parsed from the row's `remote:` field) and asserts the bytes match the
  manifest SHA-256, so CI verifies the manifest against GitHub canonical
  content on every run without anyone needing to clone the upstream
  repositories locally. Mirrors are sourced from four
  upstream repositories: `cowprotocol/contracts` (settlement,
  vault-relayer, EIP-1967 proxy slots, ERC-20), `cowprotocol/ethflowcontract`
  (EthFlow contract and order library), `cowprotocol/composable-cow`
  (conditional-order framework including the Safe Global
  `ExtensibleFallbackHandler` reached transitively through
  composable-cow's `lib/safe` submodule SHA), and `cowdao-grants/cow-shed`
  (account-abstraction proxy and hooks). The gate ships as a subcommand
  of the `parity-maintainer` binary and is wired into the workspace
  quality-gate workflow, alongside the existing source-lock validators.
  `.gitattributes` LF-normalises every `.sol`, `.yaml`, `.yml`, and
  `.json` fixture path that participates in a byte-stable contract so
  the gate stays deterministic across Windows, macOS, and Linux
  checkouts. A reviewer's audit of the abi tree is `sha256sum` on every
  file against the matching `vendored:` row. The provenance discipline
  is documented in
  [ADR 0012](docs/adr/0012-alloy-sol-bindings-and-registry-authority.md)
  and the [Contract Bindings Parity Audit](docs/audit/contract-bindings-parity-audit.md).

### Fixed

- `cow-sdk-orderbook` rejection-parser rustdoc and [ADR 0017](docs/adr/0017-typed-orderbook-rejection-parser.md)
  now correctly describe the `parse_rejection` non-JSON fallback path. When a
  non-2xx response body does not deserialize as a rejection envelope, the
  `From<OrderBookApiError>` promotion in `cow_sdk_orderbook::error` falls back
  to `OrderbookError::Api(Box<OrderBookApiError>)` (preserving the decoded
  `ResponseBody`, including the `Text` variant for plain-text bodies, and the
  derived public message), not `OrderbookError::Transport`. The `parse_rejection`
  free function docstring, the `OrderbookRejection::Unknown` variant docstring,
  and the ADR's "Must Remain True" paragraph are aligned with the shipped
  behaviour. No runtime behaviour changes.

- Cross-ABI DTOs that carry Rust `BTreeMap` fields declare the matching
  TypeScript shape as `Record<string, ...>`, so the generated declaration
  matches the plain JavaScript object that the `serde_wasm_bindgen`
  json-compatible serializer emits at runtime. The override applies to
  `TypedDataEnvelopeDto::types` in `cow-sdk-wasm` (the EIP-712 envelope
  shape carried by `signOrderWithTypedDataSigner`, `signOrderWithEip1271`,
  and the EIP-1271 callback request payload) and to the trading-client
  settlement and EthFlow contract-override maps on `SwapParametersInput`,
  `LimitTradeParametersInput`, and `OrderTraderParametersInput`. TypeScript
  callers reach map entries through plain-object indexing
  (`types["Order"]`) so the declared shape matches the value the runtime
  emits byte-for-byte.

- EthFlow on-chain order construction now refuses `receiver == address(0)`
  at the cow SDK boundary rather than producing calldata the deployed
  `CoWSwapEthFlow` contract rejects. `EthFlowOrderData::new` and
  `EthFlowOrderData::from_unsigned_order` return
  `Result<Self, ContractsError>`, surfacing `ContractsError::ZeroReceiver`
  on the structurally-illegal input. The construction-time rejection
  mirrors the deployed contract's `ReceiverMustBeSet()` revert (selector
  `0xefc9ccdf`), raised from `EthFlowOrder.toCoWSwapOrder` on both the
  `createOrder` and `invalidateOrder` write paths through the shared
  library function. The predicate lives in one private `reject_zero_receiver`
  helper invoked by the `EthFlowOrderData` construction paths; the general
  order hash path hashes a `receiver` of `address(0)` verbatim as the
  protocol's pay-to-owner sentinel and does not reject it.
  Downstream encoders in `cow-sdk-trading` and the `cow-sdk-wasm` bridge
  propagate the typed error through their existing `Result` surfaces;
  the trading-layer `OrderBoundsValidator` pre-empts the case before any
  encoder call, so the new error surface is reachable only through
  direct `cow-sdk-contracts` consumption.

- App-data canonical JSON serialisation now sorts object keys by UTF-16 code
  unit value per RFC 8785 (JSON Canonicalization Scheme), via `serde_jcs`.
  This closes a latent divergence with the upstream `@cowprotocol/cow-sdk`
  TypeScript canonical form for documents whose object keys carry code points
  whose UTF-16 ordering and UTF-8 byte ordering disagree (for example
  non-BMP code points that sort after most BMP code points in UTF-8 byte
  ordering but before them in UTF-16 code-unit ordering). Documents with
  ASCII-only object keys are unchanged; documents with non-ASCII keys may
  now hash to a different canonical CID than before. The new parity fixture
  `parity/fixtures/app_data/canonical_json_utf16.json` pins the canonical
  output and the matching app-data CID for the documented divergence.

- HTTP `Retry-After` header parsing now delegates to `httpdate::parse_http_date`
  per RFC 7231 section 7.1.1.1, which additionally accepts the legacy RFC 850
  date form (`Sunday, 06-Nov-94 08:49:37 GMT`) and the ANSI C `asctime`
  date form (`Sun Nov  6 08:49:37 1994`) that the previous IMF-fixdate-only
  parser rejected. Pre-1970 HTTP-date values now surface as the documented
  `None` ("ignore the header") path rather than as a zero-delay clamp,
  matching the upstream parser's rejection of pre-epoch dates. The new
  parity fixtures `parity/fixtures/retry_after/imf_fixdate_accept.json`,
  `parity/fixtures/retry_after/imf_fixdate_reject.json`, and
  `parity/fixtures/retry_after/legacy_rfc850.json` pin the accept and
  reject byte contracts.

### Removed

- The `async-lock = "3.4.2"` workspace dependency declaration is
  removed from the root `Cargo.toml`. No first-party crate consumed
  the pin; `cargo tree --workspace --all-features --invert async-lock`
  prints no dependency path. The lockfile node retires on the next
  `cargo update`. The
  [Dependency Gate Audit](docs/audit/dependency-gate-audit.md)
  outcome-summary row records the retirement.

- The `full` package flavor and `flavor-full` Cargo feature are removed from
  `cow-sdk-wasm`. The flavor activated the same feature set as `default`
  (`orderbook`, `signing`, `app-data`, `ipfs`, `cancellation`,
  `transport-policy`, `trading`, `subgraph`) and published a duplicate
  artifact under the `./full` package subpath. The shipped flavor enumeration
  is now `default`, `orderbook`, `signing`, and `cloudflare`; callers that
  previously imported `@cowprotocol/cow-sdk-wasm/full` use
  `@cowprotocol/cow-sdk-wasm` for the same surface. ADR 0044 is amended to
  match the four-flavor enumeration.

- `cow_sdk_core::Address::normalized_key` is removed. The accessor body was
  identical to `Address::to_hex_string` because the cow `Address` newtype
  already canonicalises every input to its lowercase 0x-prefixed hex form
  at construction time. The duplicate accessor was originally preserved for
  callers that historically routed through it; pre-1.0 with no published
  consumers, the duplicate is retired and every call site is updated to use
  `Address::to_hex_string` directly. ADR 0052 is amended to drop the
  per-type case-insensitive-key accessor from the canonical inherent-method
  surface enumeration.

- The `cow_sdk_core::config::chains` compile-time hex decoders `hex_decode_20`
  and `decode_nibble` are removed. Both were file-private `const fn` helpers
  that drove the ten `WRAPPED_NATIVE_*_BYTES: [u8; 20]` constants behind the
  `wrapped_native_token` accessor. Each constant now decodes through the
  `alloy_primitives::hex!` macro, which carries the same `0x`-prefix tolerance
  and the same compile-time panic surface (length, prefix, and per-nibble
  validation now fire through the macro's `const` evaluator rather than
  through cow-owned helpers). The ten address constants retain their
  identifiers, their `const [u8; 20]` type, and every byte value across the
  eleven supported chains, so `wrapped_native_token` and every downstream
  consumer of the per-chain byte form are unchanged. The matching
  `decode_nibble` entry is removed from `.github/config/panic-allowlist.yaml`
  because the cow-owned panic surface no longer exists; the wrapped-native
  byte constants are now guarded by the alloy macro's compile-time evaluator
  instead. This is a strict instance of executing the ADR 0052
  alloy-primitives canonical layer mandate, and the line-705 historical entry
  documenting the original introduction of the `hex_decode_*` family remains
  in place as the audit trail for the prior shape.

- Pre-1.0 breaking change. `cow_sdk_contracts::function_magic_value` is
  removed from the shipped surface. The runtime keccak-over-signature
  helper has no remaining production callers: the encoder paths in
  `cow-sdk-trading` and `cow-sdk-wasm` now route through the
  workspace `alloy::sol!` bindings, and the helper survives only as
  a crate-private parity oracle inside the `cow-sdk-contracts` test
  module. Production callers reach the same four bytes through the
  `SELECTOR` constant emitted by the matching `sol!` binding
  (for example, `IERC1271::isValidSignatureCall::SELECTOR`,
  `IERC20::approveCall::SELECTOR`, or
  `IGPv2Settlement::setPreSignatureCall::SELECTOR`) per
  [ADR 0012](docs/adr/0012-alloy-sol-bindings-and-registry-authority.md).

- Pre-1.0 breaking change. `cow_sdk_contracts::EIP1271_MAGICVALUE` is
  removed from the shipped surface. Consumers comparing against the
  EIP-1271 success magic value should reach the typed selector emitted
  by the `sol!`-generated `IERC1271` binding through
  `<cow_sdk_contracts::IERC1271::isValidSignatureCall as alloy_sol_types::SolCall>::SELECTOR`,
  which is the `[u8; 4]` constant the production verifier already
  compares against. The four-byte payload `[0x16, 0x26, 0xba, 0x7e]`
  is unchanged; only the parallel `&'static str` declaration is
  removed.

- `cow_sdk_cow_shed::address::user_salt` is removed. The helper body
  was byte-identical to `alloy_primitives::Address::into_word`
  (`[0; 32]` followed by `copy_from_slice` over `[12..]`). The single
  production caller in `cow_sdk_cow_shed::address::proxy_of` now
  inlines `user.into_word()` directly into the
  `alloy_primitives::Address::create2` call. The 30-row
  `parity/fixtures/cow_shed/proxy_addresses.json` fixture (five users
  across two deployed versions across three chains) continues to pass
  byte-for-byte; the helper had no external consumers.

### Changed

- `cow-sdk-transport-wasm`, `cow-sdk-wasm`, and `cow-sdk-browser-wallet`
  retire three hand-rolled `Promise + setTimeout + Closure` timer
  scaffolds in favor of the maintained `gloo-timers` crate. The
  browser fetch transport's `AbortController` timeout, the wasm-side
  wallet-response timeout, and the EIP-6963 provider-detection
  deadline now share a single drop-guarded `Timeout` shape, so timer
  cancellation runs through the same upstream-tested cleanup path
  on every return.

- `cow-sdk-core` adds `transport::join_request_url`, the canonical
  request-URL join helper used by every workspace `HttpTransport`
  implementation. The three former byte-identical `resolve_url`
  bodies in the native reqwest transport, the browser fetch
  transport, and the JS-callback transport collapse onto this single
  free function. Credential-bearing base URLs continue to flow
  through `Redacted::as_inner()` at the dispatch seam.

- `cow-sdk-contracts` exposes a new `hex_field` module with two
  `pub fn` helpers for decoding `0x`-prefixed hexadecimal payloads
  into raw bytes:
  - `decode_hex_field(field, value) -> Result<Vec<u8>, ContractsError>`
  - `decode_hex_field_exact::<const N: usize>(field, value) ->
    Result<[u8; N], ContractsError>`

  Both raise typed `ContractsError::InvalidHexPrefix`,
  `ContractsError::DecodeHex`, and (for the exact-length variant)
  `ContractsError::InvalidDecodedLength` with a `&'static str` field
  discriminator. The `_exact` helper returns a fixed-size byte array
  through a const generic so callers receive `[u8; N]` rather than a
  `Vec<u8>` that still needs a runtime length check. The underlying
  [`alloy_primitives::hex::FromHexError`] is preserved through
  `#[source]` on `DecodeHex` so consumers can introspect the exact
  decoder failure.

- Twenty-three call sites across `cow-sdk-alloy-provider`,
  `cow-sdk-alloy-signer`, `cow-sdk-app-data`, `cow-sdk-browser-wallet`,
  `cow-sdk-contracts`, `cow-sdk-trading`, and `cow-sdk-wasm` collapse
  the legacy `format!("0x{}", alloy_primitives::hex::encode(...))`
  shape into the single-call
  `alloy_primitives::hex::encode_prefixed(...)` form anchored by
  [ADR 0052](docs/adr/0052-alloy-primitives-canonical-primitive-layer.md).
  The cascade covers twenty production sites and three sites inside
  `#[cfg(test)] mod tests {}` blocks embedded in `src/`. The emitted
  hex strings remain byte-identical.

- Workspace-wide hex retirement. The upstream `hex` crate is removed
  from the workspace dependency graph: the `[workspace.dependencies]`
  pin in the root `Cargo.toml` and every per-crate
  `hex.workspace = true` declaration across `cow-sdk-contracts`,
  `cow-sdk-signing`, `cow-sdk-app-data`, `cow-sdk-browser-wallet`,
  `cow-sdk-wasm`, `cow-sdk-trading`, `cow-sdk-cow-shed`,
  `cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, `cow-sdk-alloy`,
  and the `cow-sdk` facade are deleted. Every production and test
  hex encode and decode callsite now resolves through
  `alloy_primitives::hex::{encode, decode}`, which routes to the
  `const-hex 1.18.x` re-export carried by `alloy-primitives 1.5.x`.
  Output is byte-identical on every input; the change is internal
  to the dependency closure. The standalone example workspace at
  `examples/native/Cargo.toml` adopts an `alloy-primitives` direct
  dependency for the same canonical resolution and drops its local
  `hex` declarations. The single `hex` node that remains in
  `Cargo.lock` is a transitive resolution of `const-hex`'s wide
  compatibility range and is not a direct edge from any first-party
  manifest.

- `cow-sdk-wasm`: the cancellation calldata path
  (`buildPresignTx`, `buildCancelOrderTx`) routes through the
  `IGPv2Settlement::setPreSignatureCall` and
  `IGPv2Settlement::invalidateOrderCall` bindings emitted by the
  workspace `alloy::sol!` block; the previous hand-rolled selector
  plus head/length/padding word emitter is removed. Calldata bytes
  are byte-identical at the wire layer; the change is internal.
  Malformed JS-supplied order UIDs now surface as the typed
  `OrderUid::new` validation error rather than the previous ad-hoc
  hex-decode rejection, which is a strict improvement in error shape.

- `cow-sdk-trading`: the ERC-20 approve calldata path
  (`approval_transaction`, `approve_cow_protocol`,
  `approve_cow_protocol_async`) routes through the
  `IERC20::approveCall` binding emitted by the workspace
  `alloy::sol!` block; the previous keccak-over-signature plus
  address-word plus uint-word assembler is removed. Calldata bytes
  are byte-identical at the wire layer; the change is internal. The
  byte equivalence is pinned by the existing
  `contracts-erc20-approve-calldata` fixture row exercised at
  `crates/contracts/tests/parity_contract.rs::assert_erc20_approve_calldata`.

- Pre-1.0 breaking change. `cow_sdk_contracts::SALT` is re-typed from
  `&'static str` to `alloy_primitives::B256`. The 32-byte payload
  (`Mattresses in Berlin!` ASCII followed by eleven zero bytes,
  `0x4d61...0000`) is unchanged and continues to drive the canonical
  CREATE2 derivation of every Settlement, VaultRelayer, and EthFlow
  deployment address recorded in `crates/contracts/registry.toml`. The
  constant is now emitted by the `alloy_primitives::fixed_bytes!`
  compile-time macro and reaches `Address::create2_from_code` as a
  typed `B256` directly. Callers that consumed the string form should
  reach the byte form through `SALT.as_slice()` or render to the
  canonical hex through `format!("{SALT:#x}")`.

- Pre-1.0 breaking change. `cow_sdk_contracts::DEPLOYER_CONTRACT` is
  re-typed from `&'static str` to `alloy_primitives::Address`. The
  20-byte payload (`0x4e59b44847b379578588920ca78fbf26c0b4956c`, the
  Arachnid deterministic-deployment proxy) is unchanged. The constant
  is now emitted by the `alloy_primitives::address!` compile-time macro,
  and `deterministic_deployment_address` consumes
  `DEPLOYER_CONTRACT.create2_from_code(SALT, &init_code)` directly
  without intermediate hex decoding.

- EIP-1967 storage-slot byte constants in `cow_sdk_contracts::proxy`
  are emitted via `alloy_primitives::fixed_bytes!` as the single byte
  source of truth. The `SlotBytes` alias is re-pointed to
  `alloy_primitives::FixedBytes<32>` (the same type as the previous
  `alloy_sol_types::private::FixedBytes<32>` re-export). The
  `Eip1967Slot::as_hex_str` accessor continues to return a
  `&'static str` because the cow-side `Provider::get_storage_at`
  trait method takes the slot as a hex string; the new
  `eip1967_slot_hex_strings_match_their_byte_forms` test in
  `crates/contracts/tests/proxy_contract.rs` pins the round-trip
  contract between the byte form and the hex string. The existing
  keccak-minus-one parity test continues to pin both forms against
  the canonical EIP-1967 derivation.

- `cow-sdk-contracts`: the `ContractsError::DecodeHex { source }` typed
  source field is now `alloy_primitives::hex::FromHexError` (a re-export
  of `const_hex::FromHexError`). The variant remains `#[non_exhaustive]`
  through the enum-level marker, and the public Display rendering of
  the variant itself is unchanged on every input. The inner
  `OddLength` source variant's `Display` rendering changes from
  `"Odd number of digits"` (upstream `hex` crate) to `"odd number of
  digits"` (alloy primitive layer); the `InvalidHexCharacter { c,
  index }` and `InvalidStringLength` source variants' renderings are
  byte-stable. Downstream consumers that pin the typed source via a
  `match` against `hex::FromHexError` variants or via
  `std::error::Error::source()` downcast must update their type path;
  consumers that match the variant wildcard or extract the outer
  `Display` rendering are unaffected.

- `cow-sdk-contracts` and `cow-sdk-signing`: the production-graph
  dependency on the upstream `hex` crate is retired. Both crates now
  resolve `hex` only through the `alloy-primitives → const-hex`
  transitive path. Every production `hex::encode` and `hex::decode`
  callsite under `crates/contracts/src/**` and
  `crates/signing/src/**` (covering EIP-1271 signature payload
  encoding, normalized ECDSA signatures, vault role hashes,
  settlement codec, EIP-1967 proxy storage decode, deployment address
  derivation, EIP-712 envelope assembly, and the domain-separator
  hex serialization) is re-pointed to `alloy_primitives::hex::*`.
  Output is byte-stable on every input. The integration test suites
  of both crates continue to consume the upstream `hex` crate through
  new `[dev-dependencies]` declarations, so the integration-test
  files are unchanged in this release.

- The native composed `cow-sdk-alloy` adapter no longer duplicates the
  read-contract and EIP-712 typed-data conversion modules from the
  leaf adapters. `AlloyClient::read_contract` consumes
  `cow_sdk_alloy_provider::__seam::execute_read_contract` and lifts
  the leaf's `AsyncProviderError` into `AlloyClientError` through the
  existing `From` impl. The umbrella's typed-data conversion module
  is reduced to a thin re-export shim over both leaf adapters'
  inter-crate seams. The previous 461-line
  `crates/alloy/src/read_contract.rs` and the duplicated ~90-line
  typed-data block in `crates/alloy/src/conversion.rs` are retired.
  Behaviour is byte-identical, and the workspace
  `alloy_read_contract_parity_invariant` integration test continues
  to assert byte-for-byte equality between the umbrella and the
  provider for pinned ABI fixtures as a regression pin against any
  future re-fork. ADR 0037's Stability section is amended to
  describe the seam-based consumption posture.

- `cow-sdk-alloy-provider`'s `#[doc(hidden)] __seam` module exposes
  `execute_read_contract` so sibling adapter crates reuse the
  canonical dynamic-ABI encode, dispatch, decode, and JSON
  serialization path without copying it. Inter-crate seam, not a
  stable consumer API; the seam may change without notice in any
  minor release per ADR 0035.

- `cow-sdk-alloy-signer` introduces a `#[doc(hidden)] pub mod __seam`
  module following the provider's posture. The seam re-exports the
  EIP-712 typed-data conversion helpers
  (`cow_typed_data_payload_to_alloy`, `cow_flat_to_alloy_typed_data`)
  and the shared signature normalizer (`alloy_signature_to_hex`).
  Inter-crate seam, not a stable consumer API; the seam may change
  without notice in any minor release per ADR 0036.

- `cow-sdk-alloy-provider::RpcAlloyProvider::get_storage_at` and
  `cow-sdk-alloy::AlloyClient::get_storage_at` build the
  0x-prefixed 64-hex storage value through
  `alloy_primitives::B256::from(value).to_string()` instead of a
  manual width-64 format string. Output is byte-identical; the new
  `crates/alloy-provider/tests/seam_contract.rs::storage_value_hex_matches_legacy_width_64_format`
  test pins the equivalence against four representative `U256`
  values (`ZERO`, `1`, `0xdeadbeef`, `MAX`).

- `cow_sdk_core::TypedDataDomain` carries an inherent
  `into_alloy_domain(&self) -> alloy_sol_types::Eip712Domain` adapter
  that returns the canonical four-field EIP-712 domain shape — `name`,
  `version`, `chainId` (encoded as `uint256`), and
  `verifyingContract` — with `salt` set to `None`. This matches the
  `GPv2` settlement-contract domain, the
  `EIP712Domain(string name,string version,uint256 chainId,address
  verifyingContract)` type string used by every shipped digest path,
  and the EIP-1193 `eth_signTypedData_v4` wire shape expected by JS
  wallets. Every typed-data path inside the workspace
  (`cow_sdk_signing::domain::domain_separator_for`,
  `cow_sdk_contracts::order::hash::hash_order` and
  `hash_order_cancellations`, and the
  `cow_typed_data_payload_to_alloy` converters inside
  `cow_sdk_alloy_signer` and `cow_sdk_alloy`) routes its
  `alloy_sol_types::Eip712Domain` construction through the new adapter,
  so the canonical bridge from the cow `TypedDataDomain` newtype to the
  alloy domain primitive lives in a single place. The
  `cow_sdk_core::traits::typed_data::tests::into_alloy_domain_emits_the_canonical_five_field_shape`
  unit test locks the field-by-field byte contract, and the existing
  `crates/signing/tests/fixtures/domain_separator_parity.json` row and
  `parity/fixtures/eip712/order_digests.json` rows pin the per-chain
  and per-order digest byte contracts.

- The four fixed-width identity newtype constructors in `cow_sdk_core`
  (`Address::new`, `AppDataHash::new`, `Hash32::new`, and
  `OrderUid::new`) parse the canonical `0x`-prefixed lowercase
  hexadecimal wire form through
  `alloy_primitives::FixedBytes::<N>::from_str` and a private cow-owned
  classifier (`classify_alloy_hex_error`) that converts the
  `alloy_primitives::hex::FromHexError` discriminants into the cow
  `ValidationError::InvalidHexLength` and
  `ValidationError::InvalidHexCharacters` variants by `match` rather
  than `#[from]`/`?` lift, so the alloy
  `FromHexError::InvalidHexCharacter { c: char, index: usize }`
  payload (one byte of caller-supplied input plus its byte offset) is
  dropped at the cow error boundary and never appears in any `Display`,
  `Debug`, or `Serialize` rendering. The strict `0x`-prefix-lowercase
  gate is preserved at the constructor entry point, so bare-hex inputs
  and uppercase `0X`-prefixed inputs continue to fail closed with
  `ValidationError::InvalidHexPrefix` exactly as before. The
  `crates/sdk/tests/error_redaction_contract.rs::fixed_width_identity_constructors_drop_offending_input_character_and_offset`
  sentinel feeds `Address::new("0xZZ...")` (40 `Z` characters) into the
  constructor and asserts neither `'Z'` nor the literal `"index"`
  appears in any rendered surface of the returned `CoreError`.
  `cow_sdk_core` no longer carries a direct workspace dependency on the
  `hex` crate (the dependency edge is removed from
  `crates/core/Cargo.toml`); the two remaining `hex::decode` and
  `hex::encode` call sites inside `cow_sdk_core::types::identity`
  (`HexData::new` and `AppDataHash::to_cid`) and the proptest hex
  encoding helpers under `crates/core/tests/property_contract.rs` route
  through `alloy_primitives::hex` which re-exports the same `const-hex`
  implementation that previously backed the workspace `hex` dep.

- `cow_sdk_core::DecimalAmount::to_decimal_string` renders the
  canonical decimal-point form through
  `alloy_primitives::utils::format_units`. The `decimals == 0` arm
  emits the integer form unchanged (no decimal point); the
  `decimals > 0` arm pads the fractional substring to length
  `self.decimals` so the canonical 1-ether row
  `(atoms = 10^18, decimals = 18)` renders as
  `"1.000000000000000000"` and the emitted string can be parsed back
  into the original `(atoms, decimals)` pair without ambiguity by any
  lossless decimal parser (the cow trailing-zero preservation
  contract holds; ethers/viem-style `"1.0"` trimming is explicitly
  out of scope per ADR 0052). The contract test at
  `crates/core/tests/types_contract.rs::decimal_amount_to_decimal_string_preserves_trailing_zeros_byte_identically`
  pins ten wire-byte rows from `(U256::ZERO, 0)` through
  `(U256::MAX, 77)`, the four invariants in
  `crates/core/tests/property_contract.rs::decimal_amount_to_decimal_string_pins_fractional_length_invariants`
  pin the structural contract across the full `(U256, 0..=77)`
  domain, and the matching panic-allowlist row at
  `.github/config/panic-allowlist.yaml` for
  `DecimalAmount::to_decimal_string` names the `format_units` panic
  site under the structural invariant `MAX_DECIMALS == 77 ==
  alloy_primitives::utils::Unit::MAX`.

- [ADR 0052](docs/adr/0052-alloy-primitives-canonical-primitive-layer.md)
  documents the cow strict-decimal-only fail-closed contract: it
  applies to the `Deserialize` wire boundary AND to
  `cow_sdk_core::SignedAmount::new`, which accepts only the grammar
  `-?[0-9]+` and rejects every `0x`/`0X`/`0o`/`0O`/`0b`/`0B` prefix
  that the alloy `I256::from_str` would otherwise silently accept,
  so the strict JSON-decimal-only signed wire contract holds at the
  constructor as well as at the deserialiser. The constructor for
  `cow_sdk_core::Amount::new` remains lenient (accepts both decimal
  and `0x`-prefixed hex; explicitly rejects `0o`/`0b`).

- `cow_sdk_contracts::IERC1271` is now a typed `alloy_sol_types::sol!` binding
  for the canonical EIP-1271 `isValidSignature(bytes32,bytes)` interface, with
  Solidity provenance at `crates/contracts/abi/cow-shed/IERC1271.sol` (a
  byte-identical mirror of `cowdao-grants/cow-shed`'s
  `src/interfaces/IERC1271.sol` gated by `cargo parity-verify-sol-provenance`).
  The
  macro-emitted
  `<IERC1271::isValidSignatureCall as alloy_sol_types::SolCall>::SELECTOR`
  constant replaces the previously hand-rolled `EIP1271_MAGICVALUE_BYTES`
  byte-array constant at every byte-comparison site in
  `cow_sdk_contracts::signature` and `cow_sdk_contracts::verify`. The
  `function_selector` private helper is retired; its only consumer
  `function_magic_value` now keccak-encodes the runtime-string signature
  directly through `alloy_primitives::keccak256`. The public
  `EIP1271_MAGICVALUE` hex-string constant remains in place as the upstream
  parity anchor.

- The `cow_sdk_cow_shed` EIP-712 signing-digest path now flows through a single
  public entry point. The previous `execute_hooks_message_hash` and
  `hash_to_sign` helpers are replaced by `execute_hooks_signing_hash`, which
  delegates to `<ExecuteHooks as alloy_sol_types::SolStruct>::eip712_signing_hash`
  on the macro-emitted `ExecuteHooks` struct in
  `crates/cow-shed/src/eip712/sol_types.rs`. The new `cow_shed_eip712_domain`
  helper exposes the underlying `alloy_sol_types::Eip712Domain` value for
  callers that need the typed-data domain directly; `cow_shed_domain_separator`
  is retained as a thin wrapper returning the same domain's `.separator()`
  byte. The cow-owned hand-rolled 66-byte EIP-712 envelope is removed; the
  canonical envelope is now produced entirely by the macro-emitted `SolStruct`
  impl. The `parity/fixtures/cow_shed/execute_hooks_digest.json` rows produce
  byte-identical output across every supported chain and version row.

- The `cow_sdk_contracts::order_kind_name`, `cow_sdk_contracts::sell_balance_name`,
  and `cow_sdk_contracts::buy_balance_name` helpers are now part of the public
  `cow-sdk-contracts` surface. Each helper returns the canonical EIP-712 label
  for one protocol enum variant (`OrderKind`, `SellTokenSource`,
  `BuyTokenDestination`); the labels keccak into the type-hash preimage that
  every signer, hasher, and EIP-1271 verifier in this workspace routes through.
  The previously private duplicates in `cow-sdk-signing` are removed and the
  signing crate now imports the canonical helpers from `cow-sdk-contracts`.
  The matching signing-side entries in
  `.github/config/panic-allowlist.yaml` are removed because the corresponding
  panic sites no longer exist; the contracts-side entries remain.

### Added

- `cowprotocol/ethflowcontract` joins the parity-source catalog as a
  Primary capability evidence pin in `parity/source-lock.yaml`, with
  producer paths `src/CoWSwapEthFlow.sol` and
  `src/libraries/EthFlowOrder.sol`. The pin anchors the upstream Solidity
  authority for the EthFlow construction-time invariants recorded in
  ADR 0020. The `docs/parity-scope.md` Source Lock table and the
  `docs/parity-sources.md` Pinned Revisions and Primary sources lists
  reflect the new entry, and the `cow-sdk-contracts` ABI bindings under
  `crates/contracts/abi/eth-flow/` now trace to a fixed upstream SHA
  rather than the unpinned upstream repository.

- `PROP-CON-018`: `EthFlowOrderData::new` and
  `EthFlowOrderData::from_unsigned_order` reject `receiver == Address::ZERO`
  with `ContractsError::ZeroReceiver`, pre-empting the deployed
  `CoWSwapEthFlow` contract's `ReceiverMustBeSet()` revert (selector
  `0xefc9ccdf`) raised by `EthFlowOrder.toCoWSwapOrder` on both the
  `createOrder` and `invalidateOrder` write paths. The rule lives in the
  private `reject_zero_receiver` helper invoked by the `EthFlowOrderData`
  construction paths; the general order hash path treats `address(0)` as the
  pay-to-owner sentinel and hashes it verbatim. Governed by ADR 0020.

- ADR 0052 (`docs/adr/0052-alloy-primitives-canonical-primitive-layer.md`)
  publishes `alloy_primitives` as the canonical EVM primitive layer and
  `alloy_sol_types` as the canonical EIP-712 / Solidity-binding layer
  across the workspace.

- Cow-owned `#[repr(transparent)]` newtypes for the cow-named identity
  types (`Address`, `Hash32`, `AppDataHash`, `HexData`, `OrderUid`) and
  numeric types (`Amount`, `SignedAmount`) over the corresponding
  `alloy_primitives` type, plus the non-transparent `DecimalAmount`
  newtype pairing an `Amount` with a `decimals: u8` scale for display
  and user-input flows.

- Cow-owned `Display`, `Serialize`, and `Deserialize` impls on `Address`,
  `Amount`, `SignedAmount`, and `DecimalAmount` lock the cow wire form
  (lowercase 0x-prefixed hex for `Address`; strict-decimal for the
  numeric family); the other byte-typed identity newtypes forward to
  alloy defaults that already match the cow lowercase contract.

- Cow-owned arithmetic operator impls (`Add`, `Sub`, `Mul`, `AddAssign`,
  etc.) on `Amount` and `SignedAmount` plus `checked_*` / `saturating_*`
  variants and `pow` / `checked_pow` / `saturating_pow`, with `MAX`,
  `MIN`, and `ZERO` constants and `#[track_caller]` annotations on the
  operator impls so debug-mode overflow panics redirect to the user call
  site.

- Strict-decimal-only fail-closed `Deserialize` for `Amount` and
  `SignedAmount` rejects `0x`, `0o`, and `0b`-prefixed input that alloy's
  underlying `ruint::Uint::FromStr` would otherwise accept; the cow
  `Amount::new` and `SignedAmount::new` constructors stay lenient to
  preserve the existing constructor contract.

- `DecimalAmount::new` rejects `decimals > 77` so the `10.pow(decimals)`
  scale used by `DecimalAmount::to_decimal_string` is structurally
  guaranteed to fit `U256`; `to_decimal_string` preserves trailing zeros
  in the fractional substring so the cow form carries full precision for
  lossless external decimal parsing.

- `PROP-ORD-010`: the `OrderCreation` `Serialize` impl routes the
  `(app_data, app_data_hash)` pair onto the services
  `OrderCreationAppData` untagged-enum variants (`Both`, `Hash`, `Full`),
  with the hash-only case keying the hash hex under the `appData` JSON
  key per the services `Hash` variant. Pinned by
  `parity/fixtures/orderbook-requests/order_creation.json`.

- `PROP-BWL-007`: the cow `TypedDataDomain` `Serialize` impl emits the
  canonical EIP-1193 `eth_signTypedData_v4` second-parameter wire shape
  (numeric `chainId`, lowercase-hex `verifyingContract`, no `salt`)
  byte-identically against
  `parity/fixtures/signing/eth_sign_typed_data_request.json`.

- Added the `cow-sdk-cow-shed` crate with typed COW Shed core types,
  generated ABI bindings, versioned CREATE2 proxy derivation, EIP-712 domain
  and message hashing, and calldata builders for hook execution. The crate is
  a provider-neutral leaf with opt-in ENS and Gnosis bindings, plus parity tests
  for proxy addresses, selectors, calldata, type hashes, domain separators, and
  init-code construction.

- Added the composable and COW Shed contract bindings to `cow-sdk-contracts`:
  the deployment registry now binds the byte-identical composable
  conditional-order framework Solidity mirrors and COW Shed
  account-abstraction proxy Solidity mirrors (both vendored under
  `crates/contracts/abi/` and gated by `cargo parity-verify-sol-provenance`),
  registers eleven new capability contract identifiers
  (ComposableCow, ExtensibleFallbackHandler, CurrentBlockTimestampFactory,
  the four non-TWAP handlers, the TWAP handler, the COW Shed
  implementation, the COW Shed factory, and the Gnosis-only
  COWShedForComposableCoW forwarder), publishes 111 per-chain capability
  rows alongside the existing GPv2 rows, and records 24 absence and
  exclusion records (Ink not-deployed and Optimism not-supported) in the
  separate coverage manifest. Selector parity tests pin the twelve
  canonical composable custom-error selectors and the COW Shed
  signature-verifier muxer interface identifier.

- Added schema v2 deployment registry readiness for composable and COW Shed
  contracts, including separated verification and coverage taxonomies,
  pinned helper source evidence, reserved leaf crate manifests, and public
  ADR/audit records for the orchestration and signing boundaries.

- Added a cow-sdk-wasm comparative benchmark validation note at
  `docs/audit/cow-sdk-wasm-comparative-benchmark-validation-note.md`
  documenting the measured package-size, correctness, runtime, and
  support-boundary tradeoffs against the upstream `@cowprotocol/cow-sdk`
  TypeScript SDK. The note is `Status: Current` with a defined refresh-
  trigger lifecycle.
- Added a consumer routing matrix to `README.md`,
  `docs/getting-started.md`, `docs/integrations.md`, and
  `crates/wasm/README.md` clarifying when the WASM package fits a use case
  versus when the upstream TypeScript SDK is the recommended choice.

- `cow-sdk-transport-policy` now owns shared HTTP retry, rate-limit,
  `Retry-After`, jitter, and transport-error classification policy for
  orderbook and subgraph clients. The `cow-sdk` facade exposes the policy
  surface through `cow_sdk::http`, including the optional reqwest classifier
  behind the `http-classifier` feature.

- `cow-sdk-wasm` now ships as the TypeScript-callable wasm-bindgen leaf crate
  for JavaScript and TypeScript consumers. The surface includes pure protocol
  helpers, typed wallet and signer callbacks, callback HTTP transport,
  orderbook, subgraph, IPFS, and trading clients, plus the
  facade-resolves-callback EIP-1271 path for smart-account signatures.
  The staged npm package layout includes web, bundler, nodejs, Cloudflare, and
  optional Deno export targets, declaration snapshots, package export
  verification, and a placeholder-name publish guard.

- The TypeScript-callable WASM package now has public architecture records,
  standing audits, and property rows for pure-helper extraction, internal
  callback registries, per-flavor package builds, narrowed signer callbacks,
  JavaScript transport policy configuration, and the TypeScript facade as the
  stable public package surface.

- A runnable native cancellation example demonstrates
  `Cancellable::cancel_with(&token)` against a delayed orderbook response.

- Native Alloy adapter support now ships as three opt-in crates:
  `cow-sdk-alloy-provider` for read-only RPC, `cow-sdk-alloy-signer` for local
  private-key signing, and `cow-sdk-alloy` for the composed provider-plus-signer
  client used by `TradingSdk` async helper flows. The default facade remains
  provider-neutral unless `alloy-provider`, `alloy-signer`, or `alloy` is
  enabled; the adapter family is native-only, hard-fails on WASM targets,
  follows the Alloy runtime `2.0` and ABI/core `1.5` pin policy, normalizes
  ECDSA recovery bytes through the shared contracts helper, and propagates
  cooperative cancellation through typed adapter errors.

- A public `ROADMAP.md` lists planned capability releases, including the
  alloy adapter crates, composable and TWAP orders, permit signing, bridging,
  flash-loans, weiroll, and hardware-wallet support.

- A native `transaction_lifecycle` example demonstrates broadcast-hash
  transaction submission beside helper-based mined receipt waiting.

- The public parity scope now carries a first-release scope section that names
  the crate families, DTO shapes, browser runtime surfaces, and explicitly
  deferred upstream packages covered by the first functional release.

- A standing WASM browser-runner determinism audit documents the pinned
  Chrome-for-Testing runner, setup command, freshness gate, and workflow
  evidence used by browser-targeted `wasm-pack` lanes.

- Release-readiness now includes blocking registry confirmation,
  parity-maintainer provenance, and CI-success aggregation lanes; routine
  quality gates include a locked native-examples lane.

- The public principle charter now includes Forward-Compatible Public
  Surfaces, Credential Redaction by Construction, Cooperative Cancellation
  Coverage, and Minimum-Viable Panic Surface.

- ADRs 0029 through 0033 now publish the accepted decisions for extension-trait
  evolution, workspace-locked versioning, OpenAPI-driven wire DTO coverage,
  machine-readable deployment provenance, and the minimum-viable panic surface.

- `TradingSdkBuilder::ready` and `TradingSdkBuilder::helper_only` provide
  ergonomic construction terminals for total ready-state trader parameters and
  helper-only chain authority.

- Release readiness automation now generates a SLSA provenance attestation
  alongside the existing software-bill-of-materials artifact so downstream
  consumers can verify the provenance of every published crate tarball.

- Continuous integration now reports semantic-versioning compatibility against
  the most recent published version of every workspace crate so accidental
  public-API regressions surface during code review.

- Continuous integration now runs a weekly drift detection lane against the
  upstream CoW services repository so newly-added error tags and request or
  response shapes surface as a tracked report before they reach the release
  window.

- Core redaction primitives cover credential-bearing URL maps,
  optional URL maps, and response-body snippets before those values reach
  public `Debug`, `Display`, `Serialize`, or error text. Dispatch paths
  retain explicit raw-value accessors, while diagnostic paths emit the
  stable `[redacted]` marker or a bounded sanitized response body.

- `cow_sdk_orderbook::OrderbookRejection` represents the services
  `InvalidTradeFilter`, `InvalidLimit`, and `LIMIT_OUT_OF_BOUNDS` wire
  tags as dedicated typed variants. The modeled rejection surface contains
  49 enum variants including the forward-compatible `Unknown` fallback.

- Contracts and signing domain-separator helpers are pinned to the same
  shared EIP-712 parity fixture so the order digest boundary cannot drift
  between the two crates.

- EIP-1271 asynchronous verification emits a tracing span plus cache-hit,
  cache-store, cache-skip, and completion events without
  recording verifier addresses, digests, signatures, or response bytes.

- Orderbook and subgraph base-URL overrides enforce canonical-host
  guard rails by default, with explicit opt-in policies for reviewed
  external hosts and loopback test routes.

- Browser wallet EIP-1193 provider construction is origin-aware: detected
  wallet origins are accepted, anonymous transports must supply an
  explicitly reviewed trusted origin, and rejected anonymous origins fail
  closed with a typed error.

- `cow_sdk_contracts::Signature` exposes scheme-aware ownership helpers:
  `recover_ecdsa_address` recovers EIP-712 and EthSign ECDSA signers from
  the supplied digest, while `declared_address` returns the declared owner
  for PreSign and EIP-1271 signatures.

- App-data metadata now exposes a typed `HookList` slot on `AppDataParams`
  for hook-bearing documents (cow-shed, flash-loans, bridging). The
  `OrderbookClient` trait is now reachable from `cow-sdk-orderbook` so
  capability consumers can compose against the trait without the trading-crate
  dependency.

- Order quote requests now pre-validate the `(signingScheme,
  onchainOrder)` pair locally so incompatible ECDSA/on-chain
  combinations fail with a typed error before the HTTP call.

- `OrderCreation` carries an opt-in
  `with_full_balance_check(bool)` builder method matching the upstream
  services policy while preserving the existing wire shape when unset.

- `cow_sdk_orderbook` response DTOs now carry OpenAPI-inventory coverage for
  order, auction-order, quote-response, trade, stored-quote, and on-chain
  order-data shapes. The source-lock-pinned services OpenAPI is vendored under
  `parity/openapi/`, six per-schema inventories and fixtures pin the modeled
  fields, and `transform_contract` asserts field-level round-trips for every
  covered DTO.

- The lowest-level transport seam on both the native and browser adapters now
  emits one tracing span per request with method, endpoint (path-only, never
  the full URL), and byte counts when the `tracing` feature is enabled.

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
  new `Order.executed_fee_amount: Amount`
  read-only sibling on `cow_sdk_orderbook::Order` that
  deserializes the deprecated `executedFeeAmount` wire field
  through the standard camelCase DTO mapping.
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
  supply `metadata.utm`, carrying `utmSource = "cow-sdk"`,
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
  tag never silently coerces to a generic default, and the
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
  `cow_sdk_contracts::verify_eip1271_signature_cached` consults before
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
  deployment accepts, and the byte-identical Solidity mirror at
  `crates/contracts/abi/erc20/IERC20.sol` (vendored from
  `cowprotocol/contracts` and gated by `cargo parity-verify-sol-provenance`)
  preserves upstream provenance for reviewers; the `IERC20Permit` interface
  is declared inline in `crates/contracts/src/erc20.rs` because EIP-2612 has
  no canonical upstream pinned in `parity/source-lock.yaml`.

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
  `Serialize` emitting the literal `[redacted]` marker and an
  `into_inner` escape for deliberate access. Secret-bearing configuration
  fields migrated to `Redacted<T>`: `ApiContext::api_key`,
  `ApiContextOverride::api_key`, and the internal `SubgraphApi` API key.

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
  from byte-identical Solidity mirrors committed under
  `crates/contracts/abi/**/*.sol`, gated by
  `cargo parity-verify-sol-provenance` against SHA-256 rows in
  `parity/source-lock.yaml`, and locked further by a byte-identity
  parity regression against fixtures derived from the upstream
  TypeScript SDK, so the encoded call-data output is always sourced from
  the upstream CoW Protocol Solidity surface rather than a parallel Rust
  reimplementation.

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
  is only available once the chain-id marker is `Set`. The ready terminal
  returns `TradingSdk`, while the helper terminal returns `HelperOnlySdk` so
  quote, post, order lookup, and off-chain cancellation methods are absent
  from the helper-only type. Chain-bound helpers (pre-sign transaction
  construction, allowance reads, approval submission, and on-chain
  cancellation) stay fully usable. A runnable
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
  the trading-first surface so later additive fields no longer require a
  major version bump. `cow-sdk-orderbook` now annotates `OrderCreation`,
  `OrderQuoteRequest`, `OrderQuoteResponse`, the wire `Order` and `Trade`
  DTOs, `EthflowData`, `QuoteSide`, `QuoteData`, `GetOrdersRequest`,
  `GetTradesRequest`, `OrderCancellations`, `NativePriceResponse`,
  `TotalSurplus`, `AppDataObject`, `CompetitionOrderStatus`,
  `CompetitionAuction`, `SolverCompetitionResponse`, `SolverSettlement`, and
  `SolverExecution`. `cow-sdk-trading` annotates
  `TradeParameters`, `LimitTradeParameters`, `TraderParameters`,
  `PartialTraderParameters`, `OrderTraderParameters`, `QuoterParameters`,
  `QuoteResults`, `QuoteRequestOverride`, `OrderPostingResult`,
  `TradeAdvancedSettings`,
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
  `WithCancellation<'t, F>` async wrapper, and the `Cancelled`
  marker error; the `cow-sdk` prelude re-exports `Cancellable` and
  `Cancelled` so `use cow_sdk::prelude::*` reaches the combinator.
  `cow-sdk-core` also re-exports
  `tokio_util::sync::CancellationToken` as
  `cow_sdk_core::CancellationToken` so every public crate routes
  cancellation through a single typed import. Every public
  long-running async method on `OrderBookApi`, `SubgraphApi`, and
  `TradingSdk` composes with `.cancel_with(&token)` at the call
  site; the combinator's poll implementation performs a biased check
  against `token.is_cancelled()` before polling the inner operation,
  so cancellation is observed before the next `.await` and the
  in-flight request handle is dropped promptly rather than waiting
  for the request deadline. `CoreError`, `OrderbookError`,
  `TradingError`, `SubgraphError`, `SigningError`, and
  `BrowserWalletError` each carry a typed `Cancelled` variant and
  implement `From<cow_sdk_core::Cancelled>` so the combinator yields
  the crate-level error directly; `SdkError::class()` routes every
  such variant to `ErrorClass::Cancelled` exhaustively.
  `docs/architecture.md` records the cancellation contract under a
  dedicated Cancellation subsection.

- `cow_sdk_signing` exports the public `Clock` trait, the
  `SystemClock` default implementation, and
  `InMemoryEip1271VerificationCache::with_clock(ttl, capacity, clock)`
  constructor so deterministic-time tests and embedders can drive cache
  expiry without sleeping. Native builds use `std::time::Instant`; WASM
  builds use `web_time::Instant`.

- `cow_sdk_trading::deployment_address_hash_input` is publicly re-exported
  for consumers that build EthFlow or pre-sign deployment-derived address
  hashes outside the SDK.

- Every `*_with_cancellation` partner method on `OrderBookApi`,
  `SubgraphApi`, and `TradingSdk` emits a structured `tracing::debug`
  event when cancellation is observed at the call-site combinator.

- `cow_sdk_core::IpfsConfig` carries a redacted `Display` implementation
  that omits credential-bearing query strings from observability sinks.

### Changed

- The workspace clippy lint set adds
  `clippy::allow_attributes_without_reason = "warn"`, so every `#[allow(...)]`
  and `#[expect(...)]` attribute in shipped sources must carry an explicit
  `reason = "..."` field. The existing bare suppressions were either
  rewritten as `#[expect(name, reason = "...")]` where the lint fires
  reliably (the seven async-fn-in-trait trait definitions on
  `cow-sdk-core` plus the constructor sites that exceed the
  `clippy::too_many_arguments` threshold and the camelCase / partial-field
  fixture rows under contract and parity tests), as `#[allow(name, reason
  = "...")]` where the lint may not fire on every build (the six shared
  `tests/common/mod.rs` helper modules and the example provider
  scaffold in `cow-sdk-trading`), or deleted entirely where the lint
  never fired (the four `impl AsyncProvider | AsyncSigner |
  AsyncSigningProvider` blocks in `cow-sdk-browser-wallet` and
  `cow-sdk-alloy-provider`, plus three constructors at or below the
  seven-argument threshold). Future bare `#[allow]` additions fail the
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  gate so the rationale always travels with the call site.

- Cryptographic primitives across the workspace route through
  `alloy_primitives` and `alloy_sol_types` per ADR 0052: EIP-712 domain
  separators and message digests through `Eip712Domain::separator` and
  `SolStruct::eip712_signing_hash`; ECDSA signature byte representation
  and compact-form handling through `alloy_primitives::Signature`;
  CREATE2 address derivation through `alloy_primitives::Address::create2`;
  EIP-191 message hashing through `alloy_primitives::eip191_hash_message`.

- App-data document canonicalisation routes through `serde_jcs::to_vec`
  per RFC 8785 (the UTF-16 ordering gap closure is documented in the
  lead `### Fixed` entries below).

- `Retry-After` HTTP-date parsing routes through
  `httpdate::parse_http_date` (the legacy date-form acceptance is
  documented in the lead `### Fixed` entries below).

- Workspace duplicate consolidation per ADR 0052: the cow `keccak256`
  wrapper paths route through `alloy_primitives::keccak256`, the cow
  `parse_u256` parsers route through `Amount::new`, the cow
  `encode_address` paths route through `Address::to_hex_string`, the
  two `SigningScheme` enums (cow-sdk-contracts and cow-sdk-orderbook)
  interoperate through a typed `From` / `TryFrom` bridge, and the
  duplicate `Call` ABI structs converge on one canonical declaration.

- The cow identity types (`Address`, `Hash32`, `AppDataHash`, `HexData`,
  `OrderUid`) and numeric types (`Amount`, `SignedAmount`) are now
  cow-owned `#[repr(transparent)]` newtypes per ADR 0052. Equality,
  hash, and ordering route to the packed-byte comparison the alloy
  primitive provides; cow-owned trait impls cover the surfaces where
  alloy defaults diverge from the cow wire contract.

- The canonical String accessor on the cow identity newtypes is
  `to_hex_string(&self) -> String`, following the Rust stdlib convention
  that `to_*` returns owned and `as_*` returns a borrow. Existing
  callsites are normalised onto canonical accessors (`Display`,
  `to_string`, `as_slice`, `is_zero`, `to_hex_string`).

- `cow_sdk_core::types::Address` `Display` always emits lowercase
  0x-prefixed hex regardless of input casing, overriding the alloy
  default EIP-55 checksum casing so the cow lowercase wire-form
  invariant holds.

- `cow_sdk_core::traits::typed_data::TypedDataDomain` is a cow-owned
  `#[non_exhaustive]` struct with cow-owned `Serialize` / `Deserialize`
  impls; the cow `Serialize` emits the canonical EIP-1193
  `eth_signTypedData_v4` wire shape directly. The cow-side
  `crates/alloy-signer/src/conversion.rs` adapter bridges
  `TypedDataDomain` to `alloy_sol_types::Eip712Domain` at the
  alloy-signer seam where the alloy-primitive form is needed for ECDSA
  signing.

- Workspace-level `sha3` and `num-bigint` declarations now carry zero
  first-party production consumers: `sha3` is scoped to
  `[dev-dependencies]` on the parity-oracle test consumers, and
  `num-bigint` reaches the workspace only transitively through `cid`,
  `multihash`, and `jsonschema`.

- Clarified public guidance so cow-sdk-wasm is positioned as a specialized
  Rust-parity TypeScript-callable surface, while the upstream
  `@cowprotocol/cow-sdk` TypeScript SDK remains the recommended option for
  standard browser dapps, web apps, CowSwap-style UIs, and most TypeScript
  applications.
- Updated Cloudflare Workers guidance to separate compressed-size
  compatibility (the cloudflare flavor's gzip artifact is below the current
  Cloudflare Workers Free compressed-size limit at the time of measurement)
  from Worker startup and deployment validation (separate refresh gates
  tracked in the validation note). The package release gate enforces an
  explicit byte budget for the cloudflare flavor's gzip size that tracks
  Cloudflare's currently published Workers limits.
- Amended `docs/adr/0039-typescript-callable-wasm-sdk-surface.md` and
  `docs/adr/0044-bundle-size-profile-and-flavor-builds.md` with positioning
  paragraphs clarifying that cow-sdk-wasm is the canonical TypeScript-
  callable surface for cow-rs's WASM package — not the default CoW Protocol
  TypeScript SDK for consumers — and cross-linked the new comparative
  benchmark validation note as evidence.
- Refreshed `docs/audit/wasm-performance-budget-audit.md` with comparative
  context, the byte-budget gate description, and the explicit release-bundle
  and startup-time gates that remain separate from the size gate.
- Updated `docs/browser-runtime-proof-posture.md` to bound runtime proof to
  the environments the release pipeline currently exercises (Node 22 and
  Node 24 as the supported LTS lines, with Node 25 measurements documented
  as point-in-time diagnostic; esbuild as the only exercised bundler; size-
  compatible Cloudflare Workers gate; modeled LCP only).
- Updated `docs/providers/README.md` so TypeScript consumer guidance
  recommends the upstream `@cowprotocol/cow-sdk` for standard browser dapps,
  with cow-sdk-wasm appropriate for specialized cases.

- CI hardening adds a forbidden-import gate for `cow-sdk-wasm`, centralizes the
  standard nextest runner on Ubuntu, macOS, and Windows with
  `fail-fast: false`, removes duplicate single-host jobs from routine and
  release-readiness workflows, and adds an IpfsFetch static gate that requires
  `.await` on every `fetch_doc_from_*` call and `async fn get` on every
  `IpfsFetchTransport` implementation.

- ADR amendments: 0007 widens the browser-wallet leaf-local rule for the WASM
  leaf-crate ecosystem; 0010 records `JsCallbackHttpTransport`,
  reqwest-stays-in-core posture, async `IpfsFetchTransport`, and string
  `schemaVersion`; 0013 covers JS callback transport and
  `cow-sdk-transport-policy`; 0019 extends sole dispatch to JS callback
  transport; 0028 records the EIP-1271 facade-resolves-callback pattern and
  `OrderUid` `as_str()` contributor rule; 0026 extracts the rehearsal and
  rollback runbook to `docs/alloy-major-release-runbook.md`; 0027 drops
  forward-speculative post-quantum scheme names; 0034 applies the canonical
  ADR heading template; and 0038 normalizes tag and cross-reference style.

- Public error variants across the SDK now redact secret-shaped payloads
  through dedicated safe wrappers or sanitized construction paths before
  those values reach `Display`, `Debug`, or `Serialize`.

- Order-creation request deserialization now rejects non-zero `feeAmount`
  values with a stable diagnostic while preserving the serialization contract
  that writes `"feeAmount": "0"`.

- `cow-sdk-trading` now validates trading `appCode` values through a typed
  `AppCode` newtype that rejects empty strings, NUL bytes, and ASCII control
  characters without imposing a length cap or printable-ASCII restriction.

- `TradingSdkBuilder::build_helper_only` now returns `HelperOnlySdk`, a
  narrower helper surface for allowance, approval, pre-sign, and on-chain
  cancellation workflows. Quote, post, order lookup, and off-chain
  cancellation methods remain available on `TradingSdk`.

- On `wasm32`, `TradingSdkBuilder::build_ready()` continues to fail fast with
  `TradingError::MissingInjectedOrderbookClient` unless an orderbook client is
  injected before the terminal is called.

- Transaction submission and observation are now split into distinct public
  types: `TransactionBroadcast` carries the broadcast transaction hash from
  signer-backed submission, while `TransactionReceipt` represents receipt
  observation with optional `status`, `block_number`, `block_hash`,
  `gas_used`, `from`, and `to` fields plus constructor and builder APIs.
  `Signer::send_transaction` and `AsyncSigner::send_transaction` now return
  `TransactionBroadcast`; provider receipt lookups continue returning
  `TransactionReceipt`.

- The composed native Alloy signer now returns the broadcast hash from Alloy's
  pending transaction handle without waiting for confirmation.

- Alloy provider and browser-wallet receipt adapters now populate rich
  `TransactionReceipt` fields, including status, block, gas, sender, and
  recipient, while browser-wallet parsing fails closed on malformed present
  fields.

- `cow-sdk-trading` now exports `submit_and_wait_for_receipt`,
  `poll_for_receipt`, `WaitOptions`, and `WaitError` for workflows that need
  to compose broadcast acknowledgement with mined receipt observation.

- Workspace dependency hygiene now keeps `cow-sdk-orderbook`,
  `cow-sdk-subgraph`, and `cow-sdk-trading` off direct `reqwest` and `tokio`
  macro dependencies on `wasm32`; source files now use the canonical `http`
  and `url` crates directly where `reqwest` previously re-exported those
  types, while native-only `reqwest::Error` classification remains available
  behind target gates.

- `cow-sdk-app-data`'s `IpfsFetchTransport` trait is now `async`. The four
  `fetch_doc_from_*` free functions are now `async fn`, and consumers must
  `.await` each call. The dual-gate `async_trait` pattern (`?Send` on wasm32;
  `Send` on native) matches the workspace's public async transport traits.

- The workspace dependency table now centralizes shared pins for `wiremock`,
  `web-time`, `gloo-timers`, `futures-timer`, and
  `console_error_panic_hook`.

- The workspace pins for `tokio`, `reqwest`, and `bytes` are updated to
  `1.52.2`, `0.13.3`, and `1.11`, respectively.

- `cow-sdk-core` reserves `TransportErrorClass::Upgrade` for future HTTP
  protocol-upgrade classification; no in-tree transport currently produces
  it.

- EthFlow order posting now documents quote-id propagation at the top-level
  trading SDK methods and in the getting-started guide, including the
  `TradingError::MissingQuoteId` failure mode.

- The `HttpTransport` trait now states explicitly that retry, jitter, rate
  limiting, and `Retry-After` handling live at the orderbook policy layer.

- `ReqwestTransportConfig::new` now documents that bare configs use
  `timeout: None` and that the orderbook builder applies the policy default.

- Deployment-registry documentation now carries the per-chain provenance
  record instead of splitting that authority into a parallel document.

- The contributor guide now documents `cargo nextest` installation and common
  workspace test commands.

- Native Alloy documentation now covers native-only target support, the
  two-family Alloy pin policy, ECDSA recovery-byte normalization, cooperative
  cancellation propagation, and the WASM hard-fail posture for Alloy features.

- ADR 0038 and the transaction receipt shape audit document the split between
  broadcast acknowledgement and mined receipt observation across adapters.

- Release documentation now describes the reproducible-build posture and the
  path to binary reproducibility for the WebAssembly artifacts.

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

- Partner fee policies now document and enforce zero-address recipient
  rejection through the typed validation paths.

- Order-book retry instrumentation now emits the documented `quote_id`,
  `attempts`, and `status` tracing fields at the call sites advertised by
  the field registry.

- Transport diagnostic surfaces (`Display`, `Debug`, and error text) redact
  URL `userinfo` before emission. `RedactedUrlMaps` and
  `RedactedOptionalUrlMaps` also derive `Hash`, allowing sanitized URL maps
  to serve as cache keys without exposing credentials.

- Orderbook DTOs `TotalSurplus.total_surplus` and
  `SolverExecution.executed_*_amount` are now `Option<Amount>` so
  partially-settled auction responses can represent absent solver-execution
  data without coercing it into a concrete amount.

- `cow_sdk_subgraph::SubgraphGraphQlError` now carries GraphQL
  `extensions` data when the upstream subgraph emits it.

- The subgraph wire boundary rejects non-finite scalar literals (`NaN`,
  `Infinity`, and `-Infinity`) with a typed `SubgraphError` instead of
  silently coercing them to default values.

- `cow_sdk_browser_wallet::Origin::new` tightens accepted schemes and
  rejects non-authority schemes such as `data:` and `blob:`.

- `BrowserWallet::from_transport` has been renamed to
  `BrowserWallet::from_transport_or_panic`, while README examples now use
  `BrowserWallet::from_trusted_transport` as the canonical fallible
  constructor for reviewed local transports. The public API lint gate now
  treats missing docs, missing debug implementations, unreachable public items,
  and unnameable types as hard errors.

- The project now tracks a register of post-1.0 type-system improvements
  queued for a future major release.

- Typestate marker structs across the workspace are now sealed against
  external construction.

- Public-field types across `cow-sdk-core`, `cow-sdk-app-data`,
  `cow-sdk-subgraph`, `cow-sdk-signing`, and `cow-sdk-trading` are now marked
  non-exhaustive so later protocol-driven field additions ship as additive
  minor changes.

- The `cow-sdk` prelude now exposes a curated first-touch surface for common
  quote, sign, post, app-data validation, transport/provider wiring, and
  primary error-handling workflows; reach specialized APIs through the
  named-module re-exports. Workspace MSRV bump policy is now documented with
  explicit cadence and notice window.

- Default-constructed transports now apply a `cow-sdk/<version>` user-agent
  and a 60-second TCP keepalive aligned with the upstream services defaults.

- Continuous integration now enforces an `alloy-*` workspace-pin same-minor
  invariant on every PR, and an inner-workspace WASM pin diff against the
  workspace pins so the example consoles cannot drift away from the workspace
  lock-step.

- The `cow-sdk-browser-wallet-console` crate name no longer carries the
  redundant `-wasm` suffix, matching the `cow-sdk-<capability>-console`
  naming convention.

- Partner-fee policies now reject the zero address as the recipient through
  app-data validation and trading quote construction before quote transport.
  The client-side order-bounds validator documentation now explicitly frames
  the validator as defence-in-depth and names broader services rejection
  classes that the SDK does not pre-cover.

- Subgraph transport errors now carry a typed class alongside the details
  string, matching the order-book error model. Cancellation events are now
  distinguishable from normal completion via a dedicated `cancelled = true`
  tracing warning when the `tracing` feature is enabled.

- Order-book wire DTO amount fields are now typed; the JSON wire shape is
  unchanged but malformed amount strings now surface as typed deserialization
  failures with the wire-shape error context.

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

- Public protocol DTOs in the contracts crate are now marked non-exhaustive and ship with explicit constructors so later protocol field additions land additively.

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
  workspace's published family including `cow-sdk-browser-wallet`,
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
  `#[non_exhaustive]` so additional wire shapes can be introduced as a
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
  Every `OrderCreation`, `OrderData`, `QuoteData`, `Order`,
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
  exposes optional fluent setters for the shared `TransportPolicy`,
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
  optional fluent setters for the shared `TransportPolicy`, partner
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
  Arbitrary-derived `OrderData` shapes and exercises
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
  backward-compatibility `TypedOrder = OrderData` alias in
  `cow-sdk-signing` is retired; the canonical `OrderData` type is the
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
  transport variants on `AppDataError` and `OrderbookError`). A new
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
  table pin ensures every later consumer of the `alloy::sol!` macro idiom
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
  baseline, and the byte-identical Solidity mirror at
  `crates/contracts/abi/settlement/GPv2Settlement.sol` (vendored from
  `cowprotocol/contracts` and gated by `cargo parity-verify-sol-provenance`)
  preserves upstream provenance for reviewers.

- `cow-sdk-contracts` now derives its `GPv2VaultRelayer` authorization-role
  bindings from an `alloy::sol!` interface block that declares the canonical
  GPv2 Vault Relayer surface alongside the partial Balancer V2 Vault ABI the
  relayer proxies (`manageUserBalance` and `batchSwap`). Vault role hashes
  returned by `required_vault_roles` now source their 4-byte method selectors
  from the generated typed interface and derive the role digest through the
  `alloy-sol-types` ABI-encoded `(address, bytes4)` tuple, keeping the
  role-hash byte output identical to the pre-migration baseline. The
  byte-identical Solidity mirror at
  `crates/contracts/abi/vault-relayer/GPv2VaultRelayer.sol` (vendored
  from `cowprotocol/contracts` and gated by
  `cargo parity-verify-sol-provenance`) preserves upstream provenance for
  reviewers.

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
  for downstream consumers, and the byte-identical Solidity mirror at
  `crates/contracts/abi/eth-flow/CoWSwapEthFlow.sol` (vendored from
  `cowprotocol/ethflowcontract` and gated by
  `cargo parity-verify-sol-provenance`) preserves upstream provenance for
  reviewers.

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
  committed Solidity mirror at
  `crates/contracts/abi/eip1967/GPv2EIP1967.sol` (a byte-identical mirror
  of `cowprotocol/contracts`'s `src/contracts/libraries/GPv2EIP1967.sol`
  gated by `cargo parity-verify-sol-provenance`) preserves upstream
  provenance for reviewers.

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
  delivers inside its poll implementation.

- The Credential Surface Contract Hygiene Audit is refreshed to cover the
  `Redacted<T>` wrapper and the transport-level error redaction path.

- `docs/release-checklist.md` now describes the functional `0.1.0` crates.io
  release publish sequence in finished-product language, naming the
  published `cow-sdk` crate family the sequence publishes in dependency
  order.

- The public principle charter now records the amended Strong Typed Public
  Surfaces, Sole Construction Seam, and Evidence-Backed Public Claims
  contracts.

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
  `SubgraphApiBuilder` now flow through `Redacted<String>` wrappers,
  so builder debug output no longer exposes secret bytes.

- Public wallet session, event, error payload, discovery, and
  chain-management types in `cow-sdk-browser-wallet` are now
  `#[non_exhaustive]`, and the constructor-backed structs expose
  explicit `new(...)` entry points so later EIP-1193 amendments
  and wallet-side capabilities land additively.

- `cow_sdk_core::SupportedChainId`, `cow_sdk_core::CowEnv`, and
  `cow_sdk_core::OrderData` are now `#[non_exhaustive]` public
  surfaces so additive chain, environment, and order-shape evolution
  remains semver-compatible ahead of `0.1.0`. Downstream crates now
  construct unsigned orders through `OrderData::new(...)` and its
  chainable `with_receiver`, `with_app_data`, `with_fee_amount`,
  `with_partially_fillable`, `with_sell_token_balance`, and
  `with_buy_token_balance` setters, while downstream `match` expressions
  over `SupportedChainId` and `CowEnv` must include wildcard fallback
  arms.

- The orderbook crate's ECDSA signing-scheme enum, auction order
  envelope, and request-policy structs are now marked non-exhaustive,
  and the request-policy surface exposes explicit constructors so
  later signing schemes, auction-side fields, and policy settings land
  additively.

- Public-field types in `cow-sdk-app-data`, `cow-sdk-subgraph`,
  `cow-sdk-signing`, and `cow-sdk-trading` are now marked non-exhaustive
  so later protocol-driven field additions ship as additive minor
  changes.

- `TradingSdkBuilder::build_ready()` on `wasm32` targets now fails fast with a typed error when no orderbook client has been injected, instead of deferring the failure to the first quote or post call.

- The release-gate docs-agreement check now guards the `cargo tree` and `cargo audit` invariants across every source-of-truth document and ships with a self-test harness that catches extraction drift in the check itself.

- Shipped WASM consoles now carry a clear acknowledgement of their current dual-authority posture - the publication authority named in the workspace crate metadata and the hosted-build authority named in the footer links - so reviewers can read the two surfaces consistently until the hosted-build rotation completes.

- Upstream-diff triage compared the source-lock-pinned commits against
  current upstream `services`, `contracts`, and `cow-sdk` HEADs on
  2026-04-29. `cow-sdk` had seven producer-path updates requiring a parity
  refresh plus three test-only changes, `services` had two producer-path
  updates requiring a parity refresh, and `contracts` had no drift.

- Refreshed `parity/source-lock.yaml` to the current upstream HEADs for
  `cowprotocol/cow-sdk`, `cowprotocol/contracts`, and
  `cowprotocol/services`; regenerated all parity fixtures and re-vendored the
  services OpenAPI against the committed source-lock. The source-lock file is
  the authoritative record for the exact upstream pins.

- The vendored services OpenAPI remains aligned with the source-lock-pinned
  services checkout, and the covered orderbook DTO inventory remains
  unchanged.

- Release provenance records `code_hash` confirmation for `Settlement`,
  `VaultRelayer`, and `EthFlow` rows on chain IDs `1`, `56`, `100`, `137`,
  `8453`, `9745`, `42161`, `43114`, `57073`, `59144`, and `11155111`;
  registry confirmation remains the release-readiness gate for chain
  deployment availability.

- Refreshed `docs/audit/dependency-gate-audit.md` for the lockfile,
  `cargo-deny`, `cargo-audit`, and duplicate-version posture.

- Added `docs/audit/wasm-surface-audit.md`,
  `docs/audit/wasm-type-generation-audit.md`, and
  `docs/audit/wasm-eip1271-parity-audit.md` for the wasm public
  surface, TypeScript declaration generation, and EIP-1271 parity contract.
  Refreshed the cooperative-cancellation, panic-free public-surface,
  credential-surface, URL credential redaction, and dependency-gate audits for
  the wasm callback transport and `WasmError` redaction posture.

- Added `docs/audit/wasm-browser-runner-determinism-audit.md` for the pinned
  browser runner contract used by WASM validation.

- Refreshed `docs/audit/contract-bindings-parity-audit.md` for vault
  role-hash parity and forbidden interaction target coverage.

- Refreshed `docs/audit/eip1271-verification-cache-audit.md` for the
  non-cacheable error matrix, clock injection, and TTL boundary tests.

- Refreshed `docs/audit/cooperative-cancellation-contract-audit.md` for the
  cancellation composition contract across `OrderBookApi`, `SubgraphApi`,
  `TradingSdk`, and the retry/backoff boundary.

- Refreshed `docs/audit/trading-order-bounds-validator-audit.md` for the
  monotonic-window property test, `u32::MAX` boundary coverage,
  `fuzz_order_bounds_validator` corpus, and same-token policy mirror.

- Refreshed `docs/audit/trading-order-construction-integrity-audit.md` for
  the parameter-builder same-token policy mirror.

- Refreshed `docs/audit/wire-dto-coverage-audit.md` for OpenAPI validator
  self-test CI wiring.

- Refreshed `docs/audit/source-lock-provenance-audit.md` for schema-v3
  fixture tests and refreshed pin authority.

- Refreshed `docs/audit/panic-free-public-surface-audit.md` for the
  item-level panic policy gate that enforces ADR 0033 `# Panics` rustdoc and
  `// SAFETY:` comments on allowlisted panic sites.

- The browser-targeted WASM lanes use the committed Chrome-for-Testing Stable
  pin `148.0.7778.56` released on `2026-04-28`, with
  `cargo check-wasm-runner-freshness` blocking stale release candidates.

- Public MSRV remains Rust `1.94.0`. Contributor toolchains are pinned to
  Rust `1.94.1` in `rust-toolchain.toml`.

- ADR 0039 (`Keep The TypeScript-Callable WASM SDK Surface As An Additive Leaf
  Crate`) and ADR 0040 (`Keep Wallet And Provider Interop Behind Typed
  JavaScript Callbacks`) document the `cow-sdk-wasm` surface and callback
  boundary. ADRs 0007, 0010, 0013, 0019, and 0028 are refreshed to reflect the
  wasm peer leaf, `JsCallbackHttpTransport`, sole-dispatch transport posture,
  and `signOrderWithCustomEip1271` smart-account entry point.

- Documentation coherence pass: workspace crate count corrected across
  shipped-surface locations; `docs/parity-matrix.md` publish-order rewritten
  to match `docs/release-checklist.md` with 13 dependency-aware steps;
  `docs/transport.md` `FixtureTransport` pedagogy example completed with
  all four trait methods; minor header arithmetic fix in
  `docs/integrations.md`. Forward-pointer added for the upcoming WASM SDK
  documentation refresh.

- CHANGELOG conformance: the spurious `[0.1.0] - 2026-05-02` block contents
  moved back into `[Unreleased]` because no git tag was cut; the maintainer
  promotes `[Unreleased]` to `[0.1.0] - <release-date>` at tag-cut time,
  with no rc.N intermediate;
  non-standard section headers folded into canonical Keep a Changelog 1.1.0
  sections; duplicate sections within `[Unreleased]` consolidated.

- PROPERTIES.md: broken named-test citations replaced with verified test
  names; PROP-DOCS-001 README inventory extended to 14 publishable crates;
  PROP-CORE-004 ADR backing reference added; PROP-WB-001 through PROP-WB-014
  now cover the TypeScript-callable wasm surface, callback transport,
  EIP-1271 parity, runtime support matrix, and redaction posture.

### Deprecated

- No public APIs are deprecated ahead of the first functional release.

### Removed

- `crates/core/src/types/identity_ext.rs` — the byte-typed extension-trait
  surface; the cow newtypes carry the accessor surface as inherent
  methods.

- `crates/core/src/types/hex.rs` — subsumed by `alloy_primitives::hex`.

- The seven `*Ext` extension traits (`AddressExt`, `Hash32Ext`,
  `AppDataHashExt`, `HexDataExt`, `OrderUidExt`, `AmountExt`,
  `SignedAmountExt`) — the cow newtypes carry their accessor surface as
  inherent methods.

- The cached `inner: AlloyAddress, hex: String` two-field layout on every
  cow identity type — the cow newtypes are now `#[repr(transparent)]`
  single-field structs over their alloy primitive.

- The cow-side hex helpers in `crates/contracts/src/primitives.rs`
  (`parse_hex`, `parse_hex_exact`, `parse_address_bytes`,
  `parse_bytes32_hash`, `parse_hex32`, `normalize_hex_payload`) — hex
  parsing routes through `alloy_primitives::hex::decode` and the cow
  newtype validating constructors.

- The cow-to-alloy conversion helpers in
  `crates/alloy-provider/src/conversion.rs` and
  `crates/alloy/src/conversion.rs` (`cow_to_alloy_address`,
  `alloy_address_to_cow_address`, `cow_to_alloy_hash`,
  `hex_data_from_bytes`, `decode_0x_hex`, `parse_u256_quantity`) — the
  cow newtypes are bit-for-bit layout-compatible with their alloy
  primitive, so adapters consume cow values directly via
  `From::from(value).into()` or `.0` access.

- Redundant `serde_json` dev-dependency declarations across the workspace
  crates that already carry `serde_json` in `[dependencies]`.

- Removed the migration guide at `docs/migration-from-cowprotocol-cow-sdk.md`.
  Specialized adoption guidance now lives with the WASM package docs under
  "When to use" and the consumer routing matrix; salvaged technical content
  (typed-data callback patterns including the `eth_signTypedData_v4`
  example) was moved into `crates/wasm/npm/README.md`.

- `cow-sdk-app-data`'s sync `IpfsFetchTransport::get` has been removed in
  favor of the async equivalent.

- Removed the permissive runtime-validated `TradingSdkBuilder::build` and
  `TradingSdkBuilder::build_partial` terminals; the typestate-gated
  `build_ready` and `build_helper_only` terminals are now the only
  construction paths. Pre-release surface; zero migration cost.

- Stale WASM build artifacts have been removed from the verification console
  package directory; the per-package gitignore now tracks only the canonical
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
  `cow_sdk_orderbook::OrderCreation`, or `cow_sdk_orderbook::Order`. Order
  submissions always wire `"feeAmount": "0"` to satisfy the services
  `NonZeroFee` constraint and
  preserve the EIP-712 struct-hash contract, so callers no longer risk
  constructing an order that the orderbook would reject at submission. The
  network-cost amount returned by `/api/v1/quote` is now accessed through
  the typed `QuoteData::network_cost_amount` getter and the
  `with_network_cost_amount` / `set_network_cost_amount` setters.

- The retired `fullFeeAmount` descriptor has been removed from the orderbook
  order-response DTO. Fee exposure on the response flows through the
  canonical `executedFee` component, while the deprecated
  `executedFeeAmount` wire field is retained as the read-only
  `Order.executed_fee_amount` sibling for historical records.
  Quote-response fee descriptors flow through `protocolFeeBps` only, in
  line with the current services schema. `cow_sdk_orderbook::calculate_total_fee`
  now takes a single `executed_fee` argument and normalizes it into the
  `total_fee` value surfaced on the transformed order.

- `TradingSdk::new` and `TradingSdk::new_partial` have been removed before the
  first functional release. Consumers must use `TradingSdkBuilder::ready`,
  `TradingSdkBuilder::helper_only`, or the fluent typestate builder terminals.

- `cow_sdk_contracts::SettlementEncoder::encode_interaction` now returns
  `Result<…, ContractsError>`. Settlement domains registered with the
  canonical CoW Protocol registry reject interactions whose target equals the
  paired vault-relayer address per ADR 0034, so call sites must propagate or
  intentionally handle the new `Result`.

- `cow_sdk_contracts::ContractsError` gains the
  `ForbiddenInteractionTarget { target: Address }` variant for the settlement
  interaction rejection above. The enum remains `#[non_exhaustive]`, so
  existing exhaustive matches should keep their wildcard fallback.

- `cow_sdk_contracts::SettlementEncoder::encoded_setup` now returns
  `Result<…, ContractsError>` to propagate the same forbidden-target check
  applied by `encode_interaction`.

- `cow_sdk_trading` validator and parameter-builder semantics now mirror
  services `SameTokensPolicy::AllowSell`: sell-side same-token orders and
  sell-side WETH-paired-with-native-sentinel orders are accepted locally,
  while buy-side same-token and buy-side WETH-native-sentinel orders still
  surface
  `TradingError::ClientRejected(ClientRejection::SameBuyAndSellToken { token })`.
  The typed variant shape is unchanged; only the call-site predicate broadens
  to honor `OrderKind`.

- `cow_sdk_transport_wasm::FetchTransport` configured `timeout: Duration` now
  bounds the full request-response lifecycle, including
  `response.text().await`. Browser consumers that relied on the earlier
  header-arrival-only timeout behavior should review their timeout settings.

### Fixed

- Direct wasm32 `getrandom` consumers now use the workspace `0.4.2` pin with
  `wasm_js`, `async-lock` is promoted to a workspace dependency, Alloy
  workspace pins stop enabling `std` globally so the contracts `k256` path
  stays wasm-compatible, the stale `tiny-keccak` license exception is removed,
  and the duplicate-version register documents the remaining upstream-owned
  roots.

- The alloy-provider invariant documentation now describes Cargo's success
  case accurately through the `cargo check-alloy-provider-invariant` wrapper,
  and the alloy provider adaptation guide replaces a placeholder
  `unimplemented!()` with a concrete `chain_id` example.

- `HostPolicyError::UnparsableUrl` corrects the public variant spelling before
  the first functional release. App-data schema panic sites now carry explicit
  invariant rationales and rustdoc panic documentation, and internal hook gas
  limit serde helpers no longer widen the public API surface.

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
  transport types under a wasm32 gate so later export drift
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
  `0.1.0` cycle. The `cow-sdk-core::cancellation` rustdoc also
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

- `cow_sdk_core::Amount::checked_mul` returns `None` on overflow instead of
  panicking, so callers that branch on checked multiplication receive the
  same fallible contract for large products as for other out-of-range amount
  operations.

- `cow_sdk_trading::get_order_to_sign` ignores zero-address receivers instead
  of serializing them as an explicit `0x0000…` recipient, matching the
  upstream behavior where a zero-address receiver means "no override."

- `cow_sdk_signing::InMemoryEip1271VerificationCache` no longer caches
  transient verification errors. Only successful verification and
  `Eip1271MagicValueMismatch` outcomes are stored; every other
  `ContractsError` variant re-checks the chain on the next call.

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

### Security

- Dependency gate documentation now records the refreshed RustSec tolerance
  posture, direct WASM randomness alignment, legacy `thiserror` codegen
  reachability, and warning-free `cargo-deny` configuration after regenerating
  both workspace and native-example lockfiles.

- API-key validation paths now fail with a typed error variant rather than a
  panic that could include the offending bytes. Base-URL override fields are
  redacted from `Debug` formatting so credentials embedded in override URLs no
  longer surface in diagnostic output. A new `sanitize_public_base_url` helper
  in `cow-sdk-core` strips path, query, and fragment from URLs before they
  cross any logging or tracing boundary.

- Both shipped WASM consoles now declare a `Content-Security-Policy` meta tag
  with explicit `script-src` and `connect-src` allowlists.

- Operator-side base-URL override and browser-wallet trust threat surfaces are
  now documented in `SECURITY.md` with explicit consumer-side mitigations.

- Three new standing audits cover the workspace `unsafe_code = deny`
  lint posture, the panic-free public surface contract, and the
  workflow security posture.

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

- The RustSec audit posture now resolves the prior `rand 0.8.5` warning
  through `rand 0.8.6`; the reviewed advisory tolerance register retains only
  the current Alloy `paste` proc-macro exception.
