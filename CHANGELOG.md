# Changelog

All notable changes to `cow-rs` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Semantic versioning begins with the first functional crate release.

Reserved-placeholder `0.0.1-reserved.0` name-reservation publishes are
excluded from this version history.

The first functional crate-family release begins at `0.1.0`.

## [Unreleased]

The first functional release of `cow-rs`, a Rust SDK for CoW Protocol. The
sections below describe the public contract a `0.1.0` consumer receives.

### Added

#### Crate family and facade

- The `cow-sdk` facade aggregates the SDK's typed surfaces over CoW Protocol:
  order creation, signing, quoting, submission, app-data handling, orderbook
  access, read-only subgraph queries, browser-compatible WASM workflows, a
  pluggable `HttpTransport` seam with native and browser default adapters,
  shared retry and rate-limit transport policy, a typed deployment registry,
  opt-in native Alloy provider and signer adapters, TypeScript-callable
  wasm-bindgen bindings, and an optional EIP-1271 signature-verification cache.
  Governed by [ADR 0001](docs/adr/0001-multi-crate-sdk-family-with-thin-facade.md).
- The facade is a thin, module-organised surface. Each leaf crate is re-exported
  as a named module and every workflow and identity type is reached on its
  module path — `cow_sdk::core`, `cow_sdk::trading`, `cow_sdk::orderbook`, and
  the rest — matching the no-prelude convention of `alloy`, `reqwest`, and
  `tower`. The crate root retains only the cross-cutting aggregate error
  (`CowError`, `ErrorClass`) and the typed transport, registry, and EIP-1271
  cache leaf surfaces. The workspace's only prelude is the opt-in
  `cow_sdk::core::prelude` of cow primitive newtypes.
- The published crate family: `cow-sdk` (facade), `cow-sdk-core` (shared domain
  types, runtime traits, and the `HttpTransport` seam), `cow-sdk-contracts`,
  `cow-sdk-signing`, `cow-sdk-app-data` (deterministic protocol helpers,
  `alloy::sol!` bindings, the `Registry` authority, and EIP-1271 verification),
  `cow-sdk-orderbook` (typed orderbook transport), `cow-sdk-trading`
  (high-level quote-to-order workflows), `cow-sdk-subgraph` (read-only subgraph
  queries), `cow-sdk-transport-wasm` (browser `FetchTransport`),
  `cow-sdk-browser-wallet`, `cow-sdk-wasm` (TypeScript-callable bindings), the
  opt-in `cow-sdk-alloy-provider` / `cow-sdk-alloy-signer` / `cow-sdk-alloy`
  adapters, and `cow-sdk-test` (in-memory test doubles).
- Reserved-placeholder `0.0.1-reserved.0` entries are live on crates.io and
  docs.rs for the published crate family. They reserve package identity and are
  not the functional SDK release; the functional release begins at `0.1.0`.

#### Trading workflows

- `cow_sdk_trading::Trading::swap` opens a fluent, typed swap lifecycle builder
  (`SwapBuilder`). Named `sell_token` / `buy_token` / `sell_amount` /
  `buy_amount` setters track the required fields in the type system so the two
  token addresses cannot be transposed, and the terminals are reachable only
  once all three are set: `execute(&signer)` quotes, signs, and posts in one
  call, while `quote(&signer)` returns a `QuotedSwap` whose `results()` can be
  inspected before `submit(&signer)`. The owner defaults to the signer address.
  Governed by [ADR 0011](docs/adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md).
- `cow_sdk_trading` exposes one async entry point per public operation —
  `post_swap_order`, `post_limit_order`, `post_limit_order_presign`,
  `post_swap_order_from_quote`, `post_cow_protocol_trade`,
  `post_sell_native_currency_order`, `offchain_cancel_order`,
  `onchain_cancel_order`, `pre_sign_transaction`, `eth_flow_transaction`,
  `quote_results`, `cow_protocol_allowance`, and `approve_cow_protocol`. The
  signer-backed entries accept any signer implementing `cow_sdk_core::Signer`
  (`post_limit_order_presign` is the deliberate signer-less exception), and
  cooperative cancellation composes on every entry through
  `cow_sdk_core::Cancellable::cancel_with(&token)`. App-code-less helper
  flows (allowance, approval, pre-sign, on-chain cancellation) are crate free
  functions that need no trading client.
- `cow_sdk_trading::Trading::post_limit_order_presign(params, advanced)` (and
  the matching crate free function) places a limit order under the `presign`
  signing scheme without consulting a signer — the smart-contract-owner path
  for Safes, vaults, and DAO treasuries. The owner must be explicit on the
  params, the wire `signature` carries the owner address in hex per the
  reviewed upstream convention, and the order becomes fillable once the owner
  activates the on-chain pre-signature flag via `setPreSignature` — for
  example by submitting the transaction built by `pre_sign_transaction`.
- On-chain transaction construction returns the fully-populated
  `cow_sdk_trading::PreparedTransaction { to, data, value, gas_limit }`:
  `pre_sign_transaction` returns it, `EthFlowTransaction.transaction` carries
  it, and `From<PreparedTransaction> for TransactionRequest` makes the bundle
  submittable through any `Signer` without per-field `Option` unwrapping.
  `eth_flow_transaction` reads the chain from `trader.chain_id` rather than a
  separate chain-id parameter, so the transaction and the trader context
  cannot disagree. Governed by
  [ADR 0020](docs/adr/0020-ethflow-owner-threading.md).
- `cow_sdk_trading::build_app_data` takes the typed
  `cow_sdk_orderbook::OrderClass` for the generated document's order-class
  metadata, stamped in its lowercase wire form through the `OrderClass` serde
  representation, so a misspelled or unsupported class string cannot reach the
  wire.
- `cow_sdk_trading::Trading` exposes its stored trader defaults through typed
  read accessors: `chain_id()`, `app_code()`, `env()`,
  `settlement_contract_override()`, and `eth_flow_contract_override()`.
- `cow_sdk_trading::Trading::swap` and the trading entry points accept an
  orderbook client by value (`TradingBuilder::orderbook`,
  `TradingOptions::with_orderbook`), so the common path no longer wraps it in
  `Arc`; the `Arc<dyn OrderbookClient>` variants remain for an already-shared
  handle. The advanced-seam setters follow the same shape:
  `PostTradeAdditionalParams::with_check_eth_flow_order_exists`,
  `PostTradeAdditionalParams::with_custom_eip1271_signature`, and
  `TradeAdvancedSettings::with_slippage_suggester` take their backend by
  value, with `with_check_eth_flow_order_exists_shared`,
  `with_custom_eip1271_signature_shared`, and `with_slippage_suggester_shared`
  accepting a backend deliberately shared across settings instances.
- EIP-1271 order-signature verification is part of the public API:
  `cow_sdk_trading::verify_eip1271_order_signature`,
  `eip1271_order_verification_request`, and `Eip1271VerificationParams` confirm a
  smart-account wallet's signature over a CoW order against a provider.
- `cow_sdk_trading::LimitTradeParamsFromQuote` is a newtype around
  `LimitTradeParams` that guarantees a non-`None` `quote_id` by construction,
  lifting the prior runtime `MissingQuoteId` check on the eth-flow path to a
  compile-time guarantee at the public boundary. Governed by
  [ADR 0011](docs/adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md).
- `cow_sdk_trading` exposes `submit_and_wait_for_receipt`, `poll_for_receipt`,
  `WaitOptions`, and `WaitError` for composing broadcast acknowledgement with
  mined-receipt observation. `WaitError::reverted()` returns the reverted
  receipt when a wait failed because the mined transaction reverted on-chain, and
  `None` for the transient or environmental variants. Governed by
  [ADR 0038](docs/adr/0038-transaction-lifecycle-types.md).
- `OrderBoundsValidator` runs client-side order-bounds checks at the services
  default policy on every public submission seam, surfacing a typed
  `ClientRejection` channel. `OrderBoundsValidator::services_default_for_chain`,
  `services_default`, and `with_weth_address` are the public constructors.
  Governed by [ADR 0015](docs/adr/0015-client-side-order-bounds-validator.md).

#### Typed primitive and amount layer

- A cow-owned `#[repr(transparent)]` primitive layer wraps the canonical
  `alloy_primitives` types: identity newtypes (`Address`, `Hash32`,
  `AppDataHash`, `HexData`, `OrderUid`) and the unsigned atomic `Amount`, with
  cow-owned `Display`, `Serialize`, and `Deserialize` impls that lock the cow
  wire form (lowercase 0x-prefixed hex for `Address`; strict-decimal for
  `Amount`). `alloy_primitives` is the canonical EVM primitive layer and
  `alloy_sol_types` the canonical EIP-712 / Solidity-binding layer across the
  workspace. Governed by
  [ADR 0052](docs/adr/0052-alloy-primitives-canonical-primitive-layer.md).
- `Amount` exposes a checked arithmetic surface — `checked_add` / `checked_sub`
  / `checked_mul` / `checked_pow` returning `Option`, with explicit
  `saturating_*` clamps — so an overflow can never silently wrap a typed amount;
  raw wrapping is available through `as_u256` / `into_u256`.
- `cow_sdk_core::OrderData` is exhaustive with all-public fields, so consumers
  on the manual signing path construct it as a struct literal — named fields
  make the three addresses and three amounts impossible to transpose. The
  field set is the EIP-712 `Order` struct frozen by the deployed settlement
  contract, so exhaustiveness is not a semver liability. The positional
  `OrderData::new` constructor and `with_*` setters remain available.
- The `cow_sdk_core::address!` macro constructs a compile-time validated
  `Address` from a `0x`-prefixed hex literal — the typed mirror of
  `alloy_primitives::address!` — so well-known addresses live in `const` items
  with malformed hex and wrong lengths rejected at build time instead of
  through a runtime `Address::new` call. The macro takes exactly one lowercase
  wire-form string literal: an EIP-55 checksum cannot be verified during const
  evaluation, so a const guard rejects a mixed-case literal at compile time
  rather than accepting an unverified checksum, and `Address::ZERO` is the
  spelling for the zero address. Both rules are pinned by trybuild cases.
  `cow_sdk::core::prelude` re-exports `address!` beside `Address`, matching
  std's `vec!`-beside-`Vec` precedent, so importing the prelude is enough to
  write compile-time validated address constants.
- `cow_sdk_core::NATIVE_CURRENCY_ADDRESS` is a typed `Address` constant
  carrying the EIP-7528 native-asset sentinel
  (`0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee`), so call sites compare and
  assign it directly; no runtime re-parse of the sentinel string remains in
  the orderbook or trading crates.
- The app-data digest newtype carries exactly one name, `AppDataHash`, across
  the workspace and the facade.
- `cow_sdk_core::Amount::parse_units(value, decimals)` and `format_units(decimals)`
  are the exact decimal token-amount construction and display surface — the
  typed analogues of viem's `parseUnits` / `formatUnits`. `parse_units` scales a
  human-decimal string by `10^decimals` with integer arithmetic only (never
  `f64`), guards inputs alloy is fail-open on, and rejects `decimals > 77`;
  `format_units` is its byte-exact inverse. `Amount::from_units(whole, decimals)`
  is the integer (no-string) companion constructor. Recorded as `PROP-CORE-018`,
  `PROP-CORE-019`, and `PROP-CORE-021` in `PROPERTIES.md`. Governed by
  [ADR 0011](docs/adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md).
- `cow_sdk_core::AppCode` and `AppCodeError` are the canonical validated
  application-identifier newtype, joining the cow-owned identity primitives in
  `cow-sdk-core` and re-exported through `cow_sdk::core`. `AppCode` rejects empty
  strings, NUL bytes, and ASCII control characters.

#### Typed error surface

- Every public error type the facade aggregates exposes `class() -> ErrorClass`
  (`CoreError`, `AppDataError`, `SigningError`, `ContractsError`,
  `OrderbookError`, `TradingError`, and `BrowserWalletError`), so a caller
  holding a bare leaf error obtains the coarse failure class —`Validation`,
  `Transport`, `Remote`, `Signing`, `RateLimited`, `Cancelled`, or `Internal` —
  without re-implementing the per-variant match. `CowError::class()` delegates to
  the per-type accessors, and a wrapped 429 stays `ErrorClass::RateLimited`.
  Governed by [ADR 0060](docs/adr/0060-uniform-error-classification.md).
- `OrderbookError` exposes two retry-decision accessors next to `class()`:
  `is_retryable() -> bool` reports whether retrying the same request may succeed
  (keying off the retained HTTP status `408`/`425`/`429`/`500`/`502`/`503`/`504`
  set or the transport failure's transient class), and
  `backoff_hint() -> Option<Duration>` returns the server-suggested wait parsed
  from the response's `Retry-After` header (RFC 7231 delta-seconds or HTTP-date).
  `TradingError` and the facade `CowError` delegate both accessors, so a consumer
  driving its own retry loop need not re-derive the retryable-status set.
  Governed by [ADR 0060](docs/adr/0060-uniform-error-classification.md).
- `cow_sdk_orderbook::OrderbookRejection::category()` returns a coarse,
  action-oriented `OrderbookRejectionCategory` (authorization, insufficient
  funds, invalid order, not found, conflict, unfulfillable, server, or unknown)
  so callers can branch on the action a rejection calls for without matching
  every wire tag; the category carries no message and never re-exposes a redacted
  payload. The full typed rejection taxonomy models 49 variants including the
  forward-compatible `Unknown` fallback. Governed by
  [ADR 0017](docs/adr/0017-typed-orderbook-rejection-parser.md).
- `cow_sdk_orderbook::OrderStatus` gains `is_terminal()` and `is_open()`
  `const fn` predicates, centralizing terminal/live classification in the
  defining crate. Public error and growth-state enums carry `#[non_exhaustive]`
  so additive variants stay semver-compatible, and error variants redact
  secret-shaped payloads through `Redacted<T>` wrappers and structured
  `{ category, line, column }` serde diagnostics before those values reach
  `Display`, `Debug`, or `Serialize`. Governed by
  [ADR 0025](docs/adr/0025-workspace-url-redaction-convention.md).

#### Transport and HTTP policy

- `cow_sdk_core::HttpTransport` is the sole live-dispatch seam on the orderbook
  and subgraph clients, dyn-compatible through `async-trait` and composed as
  `Arc<dyn HttpTransport>`. The native `ReqwestTransport` and the browser
  `FetchTransport` (from `cow-sdk-transport-wasm`) are the default adapters;
  per-call headers, an optional per-call timeout, and a typed
  `TransportError::HttpStatus { status, body }` flow through the typed channel.
  Trait futures are `Send` on native and `!Send` on `wasm32`. Governed by
  [ADR 0013](docs/adr/0013-http-transport-injection-and-typestate-builders.md)
  and [ADR 0019](docs/adr/0019-sole-http-dispatch.md).
- The orderbook, subgraph, and IPFS clients run every HTTP attempt through one
  shared retry driver that owns the attempt loop, rate-limit acquisition,
  exponential backoff, `Retry-After` honoring, and retry telemetry. A
  non-retryable transport class returns immediately. Retry-delay computation
  reads a target-neutral wall clock so an HTTP-date `Retry-After` evaluates
  correctly on both native and `wasm32` targets. `Retry-After` HTTP-date parsing
  accepts the IMF-fixdate, legacy RFC 850, and ANSI C `asctime` date forms.
  Governed by [ADR 0041](docs/adr/0041-transport-policy-l3-layering.md).
- `OrderbookApi` and `SubgraphApi` are constructed exclusively through their
  typestate builders, which carry the value each marker proves is present so the
  build terminals construct panic-free, returning a typed error rather than
  panicking when a configured user-agent cannot be encoded. On native targets a
  `.build()` overload defaults the transport to `ReqwestTransport`; on `wasm32`
  the caller supplies a `FetchTransport`. Default-constructed transports apply a
  `cow-sdk/<version>` user-agent and a 60-second TCP keepalive. Orderbook and
  subgraph base-URL overrides enforce canonical-host guard rails by default,
  with explicit opt-in policies for reviewed external hosts and loopback test
  routes.
- The native Alloy provider adapters gain an opt-in RPC retry seam.
  `cow_sdk_alloy_provider::RetryConfig` and `with_retry` wrap the JSON-RPC client
  in a bounded exponential-backoff layer that transparently retries transient,
  rate-limited reads. Retry is off by default, preserving the runtime-neutral
  posture of
  [ADR 0010](docs/adr/0010-runtime-neutral-async-and-transport-posture.md).
  Governed by [ADR 0035](docs/adr/0035-alloy-provider-adapter.md).

#### Contracts, signing, and on-chain decoding

- `cow_sdk_contracts` derives every contract binding from inline
  `alloy::sol!` interface blocks proven byte-for-byte by parity fixtures under
  `parity/fixtures/`, mirroring upstream Solidity pinned by commit in
  `parity/source-lock.yaml`: `GPv2Settlement`, `CoWSwapEthFlow`, `IERC20`,
  `IERC1271`, and the `IWrappedNativeToken` (WETH9-family) `deposit` / `withdraw`
  surface with `wrap_interaction` / `unwrap_interaction` helpers. Governed by
  [ADR 0012](docs/adr/0012-alloy-sol-bindings-and-registry-authority.md).
- EIP-712 order hashing and UID derivation operate directly on the concrete
  `cow_sdk_core::OrderData`: `hash_order`, `compute_order_uid`, and the
  cancellation hashers take `&OrderData`, the canonical type hash is exposed
  through `order_eip712_type_hash()`, and a `receiver` of `address(0)` is hashed
  verbatim as the protocol's pay-to-owner sentinel. Governed by
  [ADR 0059](docs/adr/0059-hash-concrete-orderdata-directly.md).
- `cow_sdk_core::Signer` and the narrow `TypedDataSigner` capability are
  payload-only for typed data: `sign_typed_data_payload(&TypedDataPayload)` is
  the single required typed-data method, and the payload carries the domain,
  the full types map, the primary-type name, and the message — everything a
  backend needs to compute the canonical EIP-712 digest. Field-based signing
  is not a trait obligation, because a `(domain, fields, message)` triple
  cannot name its primary type or carry nested type definitions; the
  browser-wallet signer keeps the reviewed two-layout field-based conversion
  as its inherent `sign_typed_data_compatibility` helper. Governed by
  [ADR 0068](docs/adr/0068-payload-only-typed-data-signing.md).
- `cow_sdk_contracts::Interaction` converts directly into a
  `cow_sdk_core::TransactionRequest` through `From`, so a decoded or
  hand-built settlement interaction is submittable through any `Signer`
  without field-by-field copying.
- `cow_sdk_contracts` exposes one closed-construction `RecoverableSignature`
  typestate for recoverable ECDSA signatures. It accepts only inputs whose
  trailing recovery byte is in `{0, 1, 27, 28}`, rejecting the wider alloy
  parity-normalisation range through `ContractsError::InvalidSignatureRecoveryByte`.
  `parse_hex` / `parse_bytes` / `parse_erc2098` are the sole construction paths;
  `recover(digest, scheme)` carries scheme-aware recovery, and the ERC-2098
  compact form round-trips through `to_erc2098`. Governed by
  [ADR 0022](docs/adr/0022-ecdsa-signature-recovery.md).
- Fail-closed, provider-free log decoders reconstruct typed Rust from raw chain
  logs without network access. `decode_order_placement` / `decode_order_invalidation`
  / `decode_order_refund` (and the unified `decode_eth_flow_log`) cover the
  on-chain order lifecycle; `decode_settlement_log` covers `GPv2Settlement`
  `Trade`, `Interaction`, `Settlement`, `OrderInvalidated`, and `PreSignature`
  events. Each decoder validates the topic set and indexed arity and returns a
  typed `ContractsError` rather than panicking; every topic-0 hash is byte-locked
  against an independent keccak of the canonical signature. Governed by
  [ADR 0054](docs/adr/0054-onchain-order-event-decoding-is-fail-closed.md) and
  [ADR 0056](docs/adr/0056-settlement-event-decoding-is-fail-closed.md).
- `cow-sdk-core` adds an opt-in `LogProvider: Provider` capability supertrait for
  event-log fetching, whose single-call `get_logs` issues exactly one backend
  query over a caller-bounded block range and returns raw logs for the
  fail-closed decoders — never a watcher or indexer loop. `cow-sdk-alloy-provider`
  implements it for `RpcAlloyProvider`. The capability mirrors the
  `SigningProvider` split and leaves `Provider`'s shape frozen. Governed by
  [ADR 0057](docs/adr/0057-log-provider-capability-trait.md).
- The EIP-1271 verification cache is a positive-only set keyed on the full
  `(verifier, digest, signature_hash)` probe identity. The `Eip1271Cache` trait
  exposes `contains_valid` / `record_valid` and records only successful
  magic-value matches, so a probe carrying a different signature on the same
  digest is never served a verdict recorded for another signature. The
  always-available `NoopEip1271Cache` ships alongside the `InMemoryEip1271Cache`,
  which lives behind the default-off `in-memory-cache` feature. Governed by
  [ADR 0014](docs/adr/0014-eip1271-verification-cache.md).
- The typed `cow_sdk_contracts::deployments::Registry` resolves the settlement,
  vault-relayer, and eth-flow CREATE2 singletons from committed address constants
  through `Registry::address(ContractId, chain, env)`, with `Registry::with_override`
  for local-dev deployment addresses. Governed by
  [ADR 0012](docs/adr/0012-alloy-sol-bindings-and-registry-authority.md).

#### Subgraph and account-abstraction features

- `cow-sdk` gains an off-by-default `subgraph` feature that re-exports the
  read-only `cow-sdk-subgraph` surface as `cow_sdk::subgraph` and lifts
  `SubgraphError` into a feature-gated `CowError::Subgraph` variant.
  `SubgraphError::class()` lets an enabled subgraph surface join the uniform
  error family. The default `cow-sdk` closure stays trading-first; subgraph
  access also remains usable as the standalone `cow-sdk-subgraph` crate.
- CoW Shed account-abstraction hooks ship behind the off-by-default `cow-shed`
  feature (re-exported as `cow_sdk::cow_shed`) of `cow-sdk-contracts`, reachable
  through `cow_sdk_contracts::cow_shed`. The `CowShedHooks` orchestrator resolves
  the owner from an owned `Signer`, derives the deterministic CREATE2 proxy,
  signs the `ExecuteHooks` EIP-712 payload, and encodes `factory.executeHooks`
  calldata in one `sign` call, returning a `SignedCowShedCall` that submits
  directly or becomes a CoW order pre/post hook through `to_app_data_hook`. The
  deterministic building blocks (`proxy_of`, `cow_shed_factory`,
  `cow_shed_eip712_domain`, the calldata encoders, and `CowShedVersion::ALL`) are
  public; the surface stays off the default `cow-sdk` closure. Governed by
  [ADR 0049](docs/adr/0049-cow-shed-account-abstraction-proxy.md).

#### Orderbook and app-data

- `cow_sdk_orderbook::OrderQuoteRequest` models the quote schema's `oneOf`s as
  typed Rust so an invalid request is unrepresentable: `QuoteValidity`
  (`ValidTo` xor `ValidFor`), the `OrderQuoteSide` enum, and a
  `QuoteSigningScheme` enum that keeps the verification gas limit on EIP-1271
  only and makes an ECDSA on-chain order unrepresentable. App-data is
  encapsulated as `QuoteAppData` and routed identically to the signed
  `OrderCreation`, so every form (full / hash / both) is wire-correct. The
  request defaults `priceQuality` to `PriceQuality::Optimal`. Recorded in
  [ADR 0058](docs/adr/0058-typed-quote-request-response-surface.md).
- `cow_sdk_orderbook::OrderCreation::from_quote` consumes the full
  `OrderQuoteResponse` and threads the response's quote id straight onto the
  submission payload, so the posted order settles against the quote the user
  approved instead of a fresh server-side rebind; `with_quote_id` remains the
  explicit override for hand-built payloads.
  `OrderCreation::presign_from_quote` builds the `presign` submission shape —
  no cryptographic signature; the order becomes fillable once the owner
  activates the on-chain pre-signature flag on the settlement contract — for
  smart-contract owners such as vaults and DAO treasuries.
- `cow_sdk_orderbook::OrderCreation::from_signed` is the canonical conversion
  from a signed `OrderData` into a submission payload, deriving the wire
  `appDataHash` from the signed commitment so the submitted hash cannot diverge.
  `cow_sdk_orderbook::Order::signing_order` projects a fetched order back into
  the `OrderData` used for EIP-712 hashing, returning `None` for `EthFlow` orders
  whose display fields cannot reproduce the on-chain digest.
- `cow_sdk_orderbook::OrderbookApi::upload_app_data` verifies the
  content-addressed-write invariant at both boundaries: it re-derives
  `keccak256(full_app_data)` before dispatch and rejects with
  `OrderbookError::AppDataHashMismatch { expected, observed, stage }`, decoding
  both the HTTP 200 and 201 outcomes as a bare `AppDataHash`. Recorded as
  `PROP-ORD-011` in `PROPERTIES.md`.
- `cow-sdk-orderbook` solver-competition reads target the orderbook `v2` routes
  and decode into a fully typed `SolverCompetitionResponse` carrying per-solver
  reference scores and each solution's touched orders, with addresses, amounts,
  order UIDs, and transaction hashes as workspace domain newtypes.
- `cow_sdk_app_data::validate_app_data_doc(&AppDataDoc)` returns
  `Result<(), AppDataError>`: a valid document is `Ok(())` and a failure is
  the typed, field-named error, so there is no result struct to unpack at the
  Rust boundary (the TypeScript-callable layer keeps its JavaScript
  result-object DTO). `app_data_info` runs that validation exactly once on its
  path. Governed by
  [ADR 0064](docs/adr/0064-app-data-typed-validation.md).
- `cow_sdk_app_data::AppDataParams::new(app_code: AppCode)` is the single typed
  construction entry, with fluent `into_doc()` and `into_validated()` terminals;
  the latter runs the typed document validation and computes the CID,
  canonical JSON, and keccak256 digest in one call. App-data canonical JSON
  sorts object keys by UTF-16 code unit per RFC 8785 (JCS), closing a latent
  divergence with the upstream canonical form. `PartnerFee` and
  `PartnerFeePolicy` narrow their basis-point fields to `u16`, expose typed
  `validate` surfaces enforcing the published ranges, and reject the zero address
  as a fee recipient.

#### TypeScript-callable WASM

- `cow-sdk-wasm` is the TypeScript-callable wasm-bindgen leaf crate for
  JavaScript and TypeScript consumers, exposing deterministic Rust SDK logic
  through a TypeScript facade, typed DTOs, explicit signing and HTTP callbacks,
  per-call cancellation, and per-call timeouts. The staged npm layout ships
  `default`, `orderbook`, `signing`, and `cloudflare` flavors with declaration
  snapshots, export verification, and a placeholder-name publish guard. Governed
  by [ADR 0039](docs/adr/0039-typescript-callable-wasm-sdk-surface.md).
- The read and quote surface resolves to typed DTOs: `getOrder`/`getOrders`
  return `OrderDto`(`[]`), `getTrades` returns `TradeDto[]`, `getNativePrice`
  returns `NativePriceResponseDto`, and `getQuote` returns the resolved DTO,
  which the swap-posting methods accept back unchanged. `getAppData` /
  `uploadAppData` route through the content-addressed-write path, and
  `decodeSettlementLog` / `decodeEthFlowLog` dispatch to the fail-closed
  contracts decoders without network access.
- The TypeScript-callable `WasmError` maps every `OrderbookError` and
  `TradingError` through the shared `ErrorClass`, projects the retry verdict to
  JavaScript as `retryable` plus optional `retryAfterMs`, and carries the coarse
  `OrderBookRejectionCategoryDto` with no message, preserving the redaction
  posture. An input-DTO deserialization failure normalizes to the `invalidInput`
  kind, with the offending field name surfaced. Governed by
  [ADR 0060](docs/adr/0060-uniform-error-classification.md).
- At the WASM order-input boundary an omitted `receiver` and an explicit
  zero-address `receiver` resolve to the same pay-to-owner sentinel and construct
  byte-identical `OrderData`. Recorded as `PROP-WB-022` in `PROPERTIES.md` and
  [ADR 0061](docs/adr/0061-wasm-abi-receiver-pay-to-owner.md).

#### Browser wallet

- `cow-sdk-browser-wallet` integrates an EIP-1193 wallet for WASM targets.
  Provider construction is origin-aware: detected wallet origins are accepted,
  anonymous transports must supply an explicitly reviewed trusted origin, and
  rejected origins fail closed with a typed error. `BrowserWallet::from_trusted_transport`
  is the canonical fallible constructor for reviewed local transports. Governed
  by [ADR 0007](docs/adr/0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md).

#### Native Alloy adapters

- Native Alloy adapter support ships as three opt-in crates:
  `cow-sdk-alloy-provider` for read-only RPC, `cow-sdk-alloy-signer` for local
  private-key signing, and `cow-sdk-alloy` for the composed provider-plus-signer
  client used by `Trading` async helper flows. The default facade stays
  provider-neutral unless `alloy-provider`, `alloy-signer`, or `alloy` is
  enabled; the family is native-only and hard-fails on WASM targets. Governed by
  [ADR 0035](docs/adr/0035-alloy-provider-adapter.md),
  [ADR 0036](docs/adr/0036-alloy-signer-adapter.md), and
  [ADR 0037](docs/adr/0037-alloy-umbrella-adapter.md).
- `cow_sdk_alloy_signer::LocalAlloySigner` (with `LocalAlloySignerBuilder` and
  `LocalAlloySignerBuilderError`) is the local signer's shipped name, named
  for what it holds — a locally-held private key; the adapter never loads
  keystore files. Recorded in the
  [ADR 0036](docs/adr/0036-alloy-signer-adapter.md) amendment.
- `cow_sdk_alloy_provider::RpcAlloyProviderBuilder::build()` is synchronous:
  constructing the HTTP-backed provider performs no network I/O, so no
  `.await` is required at the build terminal. The chain-checked construction
  path stays async (`cow_sdk_alloy::AlloyClientBuilder::build_checked`)
  because it performs an `eth_chainId` round-trip before returning the client.
- Transaction submission and observation are split into distinct public types:
  `TransactionBroadcast` carries the broadcast hash from signer-backed
  submission, while `TransactionReceipt` represents receipt observation with
  optional `status`, `block_number`, `block_hash`, `gas_used`, `from`, and `to`
  fields. Governed by
  [ADR 0038](docs/adr/0038-transaction-lifecycle-types.md).

#### Cancellation, observability, and testing

- Cooperative cancellation composes on every long-running SDK operation through
  `cow_sdk_core::Cancellable::cancel_with(&token)`. `cow-sdk-core` re-exports
  `CancellationToken`; each crate-level error carries a typed `Cancelled` variant
  with a `From<Cancelled>` bridge, and `CowError::class()` routes every such
  variant to `ErrorClass::Cancelled`. The combinator observes cancellation before
  the next `.await` and drops the in-flight request handle promptly.
- An opt-in `tracing` feature family spans the public crate graph. Every
  long-running operation on `OrderbookApi`, `SubgraphApi`, and `Trading`, the
  canonical signing entry points, the wallet-mediated chain operations on
  `BrowserWallet`, the WASM signing/EIP-1271/subgraph/IPFS exports, the app-data
  IPFS read helpers, `CowShedHooks::sign`, and the receipt-wait helpers emit
  spans using a documented safe field registry (`chain`, `env`, `endpoint`,
  `method`, `scheme`, `order_uid`, and related identifiers). Spans use
  `skip_all`, so no signer, signature, payload, calldata, owner, or wallet input
  is captured. With the feature off the SDK emits zero spans and adds no
  dependency. `docs/observability.md` documents the full registry.
- `cow-sdk-test` is a published crate of in-memory test doubles for the SDK's
  public trait seams, so a downstream application can test its CoW Protocol
  integration without a live orderbook, RPC endpoint, or wallet. `MockOrderbook`,
  `MockSigner`, and `MockProvider` are recording, canned-response doubles, each
  paired with a builder and a recorded-call view; the `trading(chain, app_code)`
  helper wires them into a real `Trading` client. Failure injection exercises a
  consumer's error handling, and every canned value is built through infallible
  constructors with no `unwrap` / `expect` / `panic`. Reach it directly in
  `[dev-dependencies]` or through the facade's off-by-default `testing` feature.
  Governed by
  [ADR 0063](docs/adr/0063-published-consumer-test-doubles-crate.md).

#### Verification, provenance, examples, and documentation

- The native example scenarios build as the `cow-sdk-examples-native` workspace
  member, so they share the workspace lockfile and lint policy and every
  workspace-wide check covers them; run any scenario from the repository root
  with `cargo run -p cow-sdk-examples-native --example <name>`.
- Generator-backed `proptest` property tests run on the deterministic-codec
  crates with committed regression seeds; fixture-driven parity regressions load
  `parity/fixtures/<surface>.json` and carry the upstream case id into every
  assertion; and a `cargo-fuzz` harness in a standalone `fuzz/` crate covers the
  codec boundaries (order-UID pack/unpack, typed-data digest determinism,
  app-data CID round-trip, signing-scheme classification, subgraph error
  decoding) outside the workspace members so the stable toolchain is never forced
  onto nightly. `cow_sdk_orderbook` response DTOs carry OpenAPI-inventory
  coverage against the source-lock-pinned services OpenAPI vendored under
  `parity/openapi/`.
- Release readiness confirms each recorded deployment is live on-chain through a
  read-only `eth_getCode` presence probe, generates an SLSA provenance
  attestation alongside a software-bill-of-materials, and aggregates the
  workspace CI-success lanes. `parity/source-lock.yaml` is the authoritative
  record of the exact upstream pins for `cowprotocol/cow-sdk`,
  `cowprotocol/contracts`, and `cowprotocol/services`. The public principle
  charter records Forward-Compatible Public Surfaces, Credential Redaction by
  Construction, Cooperative Cancellation Coverage, Type The Lifecycle, and
  Minimum-Viable Panic Surface, backed by the accepted ADR set, standing audits
  under `docs/audit/`, and the `PROPERTIES.md` invariant registry.
- Runnable native examples demonstrate the fluent swap flow, cooperative
  cancellation, and transaction-lifecycle submission with mined-receipt waiting;
  the `cow-trader-dioxus` browser-wallet example demonstrates consumer-side input
  and economic hygiene over the WASM surface (governed by
  [ADR 0065](docs/adr/0065-canonical-browser-wallet-example.md)). A public
  `ROADMAP.md` lists planned capability releases (composable and TWAP orders,
  permit signing, bridging, flash-loans, weiroll, and hardware-wallet support),
  `docs/parity.md` carries the first-release scope and `Intentionally
  Out-of-Scope` exclusion list, and a consumer routing matrix clarifies when the
  WASM package fits a use case versus the upstream TypeScript SDK.

### Changed

- Caller-built request and configuration structs no longer carry
  `#[non_exhaustive]` (`TradeParams`, `LimitTradeParams`, `OrderCreation`,
  `OrderQuoteRequest`, `RetryPolicy`, `TransportPolicy`, `WaitOptions`,
  `SubgraphConfig`, `AppDataParams`), so direct struct construction and
  exhaustive matching are supported; `new()` plus `with_*()` builders keep
  additive fields compatible. `#[non_exhaustive]` is retained on error enums,
  server-extensible protocol enums, SDK-constructed response DTOs, and the
  EIP-712 wire structs.
- Public accessors and domain fetch methods drop the non-idiomatic `get_` prefix
  (`OrderbookApi::quote`, `order`, `Trading::quote_only`, `app_data_info`,
  `signing::domain`, signer `address()`). The chain-RPC `Provider` and
  `LogProvider` methods keep `get_` because they mirror the Ethereum JSON-RPC
  method names. The TypeScript and npm export names are unchanged. Governed by
  [ADR 0067](docs/adr/0067-idiomatic-accessor-naming.md).
- Construction-builder setters name themselves by the bare configuration noun
  rather than a `with_` prefix (`TradingBuilder` / `OrderbookApiBuilder` /
  `SubgraphApiBuilder`: `chain_id`, `app_code`, `env`, `transport`,
  `external_host_policy`, and the rest), matching the standard-library builder
  convention. The `with_*` convention is retained on owned-value parameter and
  configuration types. Governed by
  [ADR 0011](docs/adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md)
  and
  [ADR 0013](docs/adr/0013-http-transport-injection-and-typestate-builders.md).
- `cow_sdk_orderbook::OrderbookError::Api` renders the HTTP status on its public
  message (`orderbook request failed (<status>)`) so an unclassified API failure
  no longer surfaces as a bare `[redacted]`; the response body and derived
  message stay redacted. `cow_sdk_browser_wallet::BrowserWalletError` surfaces
  the EIP-1193 RPC method name (a closed-set protocol identifier, not a
  credential) on every variant that carries one, while the wallet's free-form
  message stays redacted. Governed by
  [ADR 0017](docs/adr/0017-typed-orderbook-rejection-parser.md) and
  [ADR 0025](docs/adr/0025-workspace-url-redaction-convention.md).
- `cow_sdk_contracts::ContractsError::class()` partitions its variants:
  caller-supplied shape and range failures report `ErrorClass::Validation`,
  serialization/ABI/decode invariants report `ErrorClass::Internal`, and the
  EIP-1271, provider, and ECDSA-recovery operations report `ErrorClass::Signing`.
  Governed by [ADR 0060](docs/adr/0060-uniform-error-classification.md).
- `cow_sdk_orderbook::OrderbookRejection::category()` classifies
  `SellAmountDoesNotCoverFee` as `Unfulfillable` rather than `InvalidOrder`,
  naming the correct consumer action (re-quote, wait, or resize) for an economic
  quote-time shortfall. Governed by
  [ADR 0017](docs/adr/0017-typed-orderbook-rejection-parser.md).
- `cow_sdk_core::OrderBalance` is replaced with two side-specific enums,
  `SellTokenSource { Erc20, External, Internal }` and
  `BuyTokenDestination { Erc20, Internal }`, mirroring the services types
  byte-identically on the wire so quote-derived and direct order construction
  cannot silently rewrite the buy-side destination; the type system rejects any
  cross-side assignment at compile time.
- `cow_sdk_subgraph::SubgraphApi` exposes a single `with_config_override(self)`
  returning a reconfigured client (replacing the per-call `*_with_config`
  twins), mirroring `OrderbookApi::with_context_override`, and the production
  deployment ids consolidate into one source of truth.
- `cow_sdk_subgraph::SubgraphError`'s `Display` carries plaintext structural
  diagnostic on every variant; free-form `errors[].message`, `body`, and
  `details` payloads stay behind `Redacted<T>`. Governed by
  [ADR 0025](docs/adr/0025-workspace-url-redaction-convention.md).
- Workspace cryptographic primitives route through `alloy_primitives` and
  `alloy_sol_types` per
  [ADR 0052](docs/adr/0052-alloy-primitives-canonical-primitive-layer.md):
  EIP-712 domain separators and digests, ECDSA signature byte representation,
  CREATE2 derivation, EIP-191 hashing, and hex encode/decode. The upstream `hex`
  crate is retired from the production dependency graph in favor of
  `alloy_primitives::hex`; output is byte-identical on every input. Eight CI grep
  gates fence the never-swap surfaces, and ten inline `DO NOT SWAP` comment
  blocks anchor the doctrine at the load-bearing call sites; `docs/alloy-doctrine.md`
  is the canonical human-readable consolidation.
- The workspace clippy lint set requires every `#[allow(...)]` / `#[expect(...)]`
  to carry an explicit `reason = "..."`, enforced by the
  `cargo clippy --workspace --all-targets --all-features -- -D warnings` gate.
  The public-API lint gate treats missing docs, missing debug implementations,
  unreachable public items, and unnameable types as hard errors.
- The workspace pins `tokio`, `reqwest`, and `bytes` at `1.52.2`, `0.13.3`, and
  `1.11`, and centralizes the CID stack (`cid`, `multihash`) and shared transport
  and test pins through `[workspace.dependencies]`.

### Removed

- Removed the `cow-sdk-pure-helpers` crate by folding its deterministic,
  FFI-free protocol helpers into `cow-sdk-wasm::helpers`; the wasm crate was its
  only consumer and the `cow_sdk::wasm::helpers` facade path is unchanged. The
  `cow-sdk-transport-policy` crate is likewise folded into
  `cow_sdk_core::transport::policy` behind the off-by-default `transport-policy`
  feature, and the standalone `cow-sdk-cow-shed` crate into
  `cow_sdk_contracts::cow_shed` behind the off-by-default `cow-shed` feature. The
  public types are reached on their new module paths.
- Removed `cow_sdk_core::SignedAmount` and `cow_sdk_core::DecimalAmount`. The
  signed `int256` newtype was load-bearing for no shipped flow and has no
  upstream analogue, and the decimals-paired wrapper carried an `f64`-lossy
  approximation seam; `Amount` remains the single canonical amount type, and
  token-decimal construction and display are the exact, integer-only
  `Amount::parse_units` / `format_units`. Governed by
  [ADR 0011](docs/adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md).
- Removed the trading quote cache (`QuoteCache`, `QuoteCacheKey`, the `Noop` and
  `InMemory` implementations, the builder setters, and the TTL/capacity
  constants). The seam was never consulted by the quote flow, its key omitted
  quote-determining inputs, and a quote's economic value is too time-sensitive to
  memoize behind a fixed TTL; this also drops `parking_lot` from
  `cow-sdk-trading`.
- Removed the helper-only trading client (`TradingHelpers` and the helper-only
  builder terminal). It duplicated methods already on `Trading` and added no
  capability the crate free functions did not provide; the app-code-less helper
  flows are the crate free functions, and `Trading` still exposes them as
  conveniences. Governed by
  [ADR 0011](docs/adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md).
- Removed the settlement encoder and trade/order flag codec from
  `cow-sdk-contracts` (`SettlementEncoder`, `encode_trade`, the flag codecs, and
  their DTOs). Settlement-calldata encoding is a solver/backend concern absent
  from the upstream SDK and with no SDK consumer; the trader-facing
  `setPreSignature` and `invalidateOrder` calls encode directly from the
  binding, and the fail-closed `decode_settlement_log` decoder is retained.
  Governed by
  [ADR 0034](docs/adr/0034-interaction-encoder-target-policy.md).
- Collapsed the deployment registry to a const table, removing the
  `registry.toml` file, the `build.rs` schema validator, and the runtime TOML
  parser. `ContractId` narrows to `Settlement`, `VaultRelayer`, and `EthFlow`,
  and `cow-sdk-contracts` drops its `toml` and `serde_yaml` dependencies.
  Governed by
  [ADR 0012](docs/adr/0012-alloy-sol-bindings-and-registry-authority.md).
- Removed the EIP-2612 `IERC20Permit` binding, `permit_typed_data_hash`, and
  `PERMIT_TYPE_HASH` from `cow-sdk-contracts`; the permit surface had no consumer
  and is absent from the upstream SDK. The `IERC20` approve and balance/transfer
  bindings the allowance flow uses are unchanged.
- Removed the client-side IPFS upload seam from `cow-sdk-app-data`. Registering
  an app-data document is orderbook-mediated: hash the document locally with
  `app_data_info`, then submit it through the orderbook crate's
  content-addressed-write path. The IPFS read seam is unchanged.
- Removed `OrderbookApi::auction` and the `Auction` response type; the
  `/api/v1/auction` endpoint is not reachable for public clients and upstream
  treats it as a liveness probe. Auction retrieval can return as an additive
  change if the endpoint becomes publicly consumable.
- Removed the `full` package flavor from `cow-sdk-wasm`. It activated the same
  feature set as `default` and published a duplicate artifact; callers that
  imported the `/full` subpath use the base package for the same surface. The
  shipped flavor enumeration is `default`, `orderbook`, `signing`, and
  `cloudflare`. Governed by
  [ADR 0044](docs/adr/0044-bundle-size-profile-and-flavor-builds.md).
- Removed several internal-only or duplicate surfaces with no consumer journey:
  `AppDataHash::to_cid` / `try_from_cid` in `cow-sdk-core` (the canonical CID
  helpers live in `cow-sdk-app-data`; drops the `cid` / `multihash` /
  `multibase` deps from core), the `cow_sdk_core::Order` envelope and the
  unused user-domain `QuoteRequest` / `QuoteResponse` / `Trade` types, the
  deprecated always-`null` `availableBalance` field on the orderbook `Order`
  DTO, the unused `getrandom` dependency on `cow-sdk-browser-wallet`, and the
  inert `tracing` feature on the native Alloy adapter crates.

### Fixed

- `cow-sdk-wasm` compiles for `wasm32-unknown-unknown` under the default feature
  flavor. A `signing`-gated error conversion matched a removed
  settlement-interaction-target variant; the conversion now maps every
  `ContractsError` to the typed signing error, and the wasm workflow gained a
  default-flavor `wasm32` compile check so the gap cannot recur.
- The hand-written `cow-sdk-wasm` TypeScript facade `CowError` `orderbook` member
  declares the `retryable` and `retryAfterMs` fields the Rust `WasmError` emits,
  so a TypeScript consumer reads the retry verdict type-safely; a package test
  compares the facade union against the generated declaration snapshot. Governed
  by [ADR 0047](docs/adr/0047-typescript-facade-architecture.md) and
  [ADR 0060](docs/adr/0060-uniform-error-classification.md).
- `cow_sdk_orderbook::OrderQuoteRequest::with_app_data_hash` produces the
  hash-only quote app-data wire form instead of pairing the requested hash with
  a placeholder document that the orderbook re-hashes and rejects. A lone
  `appData` carrying a 32-byte hash deserializes into the hash slot, matching the
  orderbook's own parsing so a hash-only request round-trips.
- EthFlow on-chain order construction refuses `receiver == address(0)` at the
  SDK boundary rather than producing calldata the deployed `CoWSwapEthFlow`
  contract rejects: `EthFlowOrderData::new` and `from_unsigned_order` return
  `ContractsError::ZeroReceiver`, mirroring the contract's `ReceiverMustBeSet()`
  revert. The general order hash path still hashes `address(0)` verbatim as the
  pay-to-owner sentinel. Recorded as `PROP-CON-018` in `PROPERTIES.md` and
  governed by [ADR 0020](docs/adr/0020-ethflow-owner-threading.md).
- `cow_sdk_trading::Trading` cancellation spans record the effective chain and
  environment resolved from the trader defaults instead of `None` when the caller
  supplies an `OrderTraderParams` without them, matching the quote-path spans and
  the `chain` field contract.
- Cross-ABI DTOs carrying Rust `BTreeMap` fields declare the matching TypeScript
  shape as `Record<string, ...>`, so the generated declaration matches the plain
  JavaScript object the runtime serializer emits byte-for-byte.
- `cow-sdk-wasm` normalizes an input-DTO deserialization failure from JavaScript
  (unknown enum variant, missing required field, or wrong field type) to the
  `invalidInput` `CowError` kind instead of `internal`, surfacing the offending
  field name; the `internal` kind stays reserved for genuine SDK-side faults.

### Security

- Credential-bearing values never reach a public `Debug`, `Display`,
  `Serialize`, or error-text surface. Core redaction primitives cover
  credential-bearing URL maps and response-body snippets behind the `Redacted<T>`
  wrapper; dispatch paths retain explicit raw-value accessors while diagnostic
  paths emit the stable `[redacted]` marker or a bounded sanitized body. Partner
  API keys on `OrderbookApiBuilder` and `SubgraphApiBuilder` flow through
  `Redacted<String>`, transport diagnostic surfaces strip URL `userinfo`, and the
  fixed-width identity constructors drop the offending input character and offset
  from any rendered error. Governed by
  [ADR 0025](docs/adr/0025-workspace-url-redaction-convention.md).
- EIP-1271 verification and the tracing surface never record verifier addresses,
  digests, signatures, signers, calldata, owners, or response bytes; spans use
  `skip_all` and the cache records only positive magic-value matches.
- Orderbook and subgraph base-URL overrides enforce canonical-host guard rails by
  default, and browser-wallet provider construction fails closed on an unreviewed
  anonymous origin, so a misconfigured host or origin is rejected rather than
  silently trusted.

[Unreleased]: https://github.com/cowdao-grants/cow-rs
