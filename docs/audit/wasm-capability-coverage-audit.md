# WASM Capability Coverage Audit

Status: Current
Last reviewed: 2026-06-09
Owning surface: `cow-sdk-wasm` capability coverage relative to the native `cow-rs` SDK crates
Refresh trigger: changes to `crates/wasm/src/exports/**`; additions or removals of public operations on the `orderbook`, `trading`, `signing`, `contracts`, `app-data`, or `subgraph` crates; or revisions to the workflow scope in `docs/parity.md`
Related docs:
- [ADR 0039](../adr/0039-typescript-callable-wasm-sdk-surface.md)
- [WASM Surface Audit](wasm-surface-audit.md)
- [Parity Scope](../parity.md)
- [Parity Matrix](../parity.md)

## Scope

This audit covers:

- the mapping between the public capability surface of the native `cow-rs`
  crates (`orderbook`, `trading`, `signing`, `contracts`, `app-data`,
  `subgraph`) and the TypeScript-callable exports of `cow-sdk-wasm`
- the coverage of the workflow scope defined in `docs/parity.md`
- the classification of every native capability that `cow-sdk-wasm` does not
  surface, with the rationale for each boundary
- the shape correspondence between the native Rust signatures and types and the
  generated TypeScript surface — construction, capability injection, inputs,
  output envelopes, primitives, enumerations, errors, and instance lifetime —
  and the specific points where a capability's shape diverges beyond the uniform
  transforms

It does not cover npm packaging, runtime support claims, bundle size, the
exhaustive field-level declaration snapshot and its wire-shape parity, or
error-redaction posture; those are owned by the
[WASM Surface Audit](wasm-surface-audit.md), [ADR 0044](../adr/0044-bundle-size-profile-and-flavor-builds.md),
and the [WASM Type Generation Audit](wasm-type-generation-audit.md).

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Workflow scope | The deterministic-helper, signing, service-client, and trading workflows defined for `cow-sdk-wasm` in ADR 0039 and `docs/parity.md` are exposed and contract-tested | Conforms |
| Surface layering | The four documented layers — deterministic helpers, wallet callbacks, service clients, trading — are present and contract-tested | Conforms |
| Runtime-model boundary | The wasm32 dependency tree excludes the native Alloy adapters and browser-wallet, and exposes no Rust signer that broadcasts or provider that polls (ADR 0039) | Conforms |
| Non-surfaced capabilities | Every native capability without a `cow-sdk-wasm` export is classified, and each class has a stated rationale | Documented |
| Shape correspondence | Native types and signatures map to the WASM DTO and TypeScript surface through a fixed transform set (config-object construction, callback injection, camelCase DTOs, string-typed primitives, versioned envelopes, discriminated-union errors); divergences beyond the uniform transforms are enumerated | Documented |
| Additive candidate | One non-surfaced capability with no runtime-model barrier is recorded for tracking (approval-transaction builder) | Recorded |

## Current Contract

### Exposed surface

`cow-sdk-wasm` exposes four layers, sourced from the native crates rather than
reimplemented:

1. **Deterministic helpers** — domain separator, order typed-data, order-UID
   computation, app-data document/info/validation, CID and hash conversion,
   supported-chain and deployment-address lookup, EIP-1271 payload encoding,
   and the provider-free settlement and eth-flow event-log decoders.
2. **Wallet-callback signing** — typed-data, EIP-1193, digest, EIP-1271, and
   custom EIP-1271 order signing; cancellation signing; and the pre-sign and
   cancellation transaction builders.
3. **Service clients** — `OrderBookClient`, `SubgraphClient`, and `IpfsClient`
   over default or callback HTTP.
4. **Trading** — `TradingClient` quote and post flows, including the
   EIP-1271-backed swap path and the native-currency-sell transaction builder.

The canonical export inventory is enforced by
`crates/wasm/tests/wasm_surface_contract.rs` and
`crates/wasm/tests/wasm_workflow_coverage_contract.rs`.

### Coverage map by crate

Legend: **Surfaced** — a `cow-sdk-wasm` export exists; **Surfaced (builder
form)** — the host receives an unsigned transaction to submit through its own
wallet; **Surfaced (composed)** — covered by combining exported operations;
**Not surfaced** — no `cow-sdk-wasm` export, classified in the next section.

#### orderbook — `OrderbookApi` → `OrderBookClient`

| Native operation | WASM export | Coverage |
| --- | --- | --- |
| `quote` | `getQuote` | Surfaced |
| `send_order` | `sendOrder` / `sendOrderCreation` | Surfaced |
| `send_cancellations` | `cancelOrders` | Surfaced |
| `order` | `getOrder` | Surfaced |
| `orders` | `getOrders` / `getOrdersByOwner` | Surfaced |
| `trades` | `getTrades` | Surfaced |
| `native_price` | `getNativePrice` | Surfaced |
| `app_data` | `getAppData` | Surfaced |
| `upload_app_data` | `uploadAppData` | Surfaced |
| `version` | — | Not surfaced (Class 2) |
| `order_link` | — | Not surfaced (Class 2) |
| `order_multi_env` | — | Not surfaced (Class 2) |
| `tx_orders` | — | Not surfaced (Class 2) |
| `order_competition_status` | — | Not surfaced (Class 2) |
| `total_surplus` | — | Not surfaced (Class 2) |
| `solver_competition` | — | Not surfaced (Class 2) |
| `solver_competition_by_tx_hash` | — | Not surfaced (Class 2) |
| `latest_solver_competition` | — | Not surfaced (Class 2) |

#### trading — `Trading` → `TradingClient`

| Native operation | WASM export | Coverage |
| --- | --- | --- |
| `quote_only` | `getQuote` | Surfaced |
| `post_swap_order` | `postSwapOrder` | Surfaced |
| `post_swap_order_from_quote` | `postSwapOrderFromQuote` | Surfaced |
| `swap` (fluent `SwapBuilder` lifecycle) | `postSwapOrder` / `postSwapOrderFromQuote` / `getQuote` | Surfaced (native-only fluent shape over surfaced ops; see shape note) |
| `post_limit_order` | `postLimitOrder` | Surfaced |
| `cow_protocol_allowance` | `getCowProtocolAllowance` | Surfaced |
| `post_sell_native_currency_order` | `buildSellNativeCurrencyTx` | Surfaced (builder form) |
| `quote_results` | `getQuote` (owner supplied explicitly) | Surfaced (alternate shape) |
| `order` | `OrderBookClient.getOrder` | Surfaced (via orderbook client) |
| `pre_sign_transaction` | `buildPresignTx` | Surfaced (builder form) |
| `onchain_cancel_order` | `buildCancelOrderTx` | Surfaced (builder form) |
| `offchain_cancel_order` | `signCancellation*` + `cancelOrders` | Surfaced (composed) |
| `approval_transaction` / `approve_cow_protocol` | — | Not surfaced (recorded observation) |
| `poll_for_receipt` / `submit_and_wait_for_receipt` | — | Not surfaced (Class 1) |

#### signing — `signing` crate

| Native operation | WASM export | Coverage |
| --- | --- | --- |
| `sign_order` / `sign_order_with_scheme` | `signOrderWithTypedDataSigner`, `signOrderEthSignDigest`, `signOrderWithEip1193` | Surfaced |
| `generate_order_id` | `computeOrderUid` | Surfaced |
| `order_typed_data` / `order_typed_data_payload` / `domain_separator` | `orderTypedData`, `domainSeparator` | Surfaced |
| `eip1271_signature_payload` | `eip1271SignaturePayload`, `signOrderWithEip1271`, `signOrderWithCustomEip1271` | Surfaced |
| `sign_order_cancellation` / `sign_order_cancellations` (+ scheme variants) | `signCancellationWithTypedDataSigner`, `signCancellationWithEip1193`, `signCancellationEthSignDigest` | Surfaced |
| `verify_eip1271_signature` / `verify_eip1271_signature_cached` and the verification caches | — | Not surfaced (Class 2) |

#### app-data, subgraph, contracts

| Crate | Coverage |
| --- | --- |
| `app-data` | Surfaced: document generation, info/hash/CID derivation, validation, CID and hex conversion, and IPFS fetch by CID and by app-data hash. Typed metadata builders (hooks, flashloan, partner fee) are reachable through the app-data document `metadata` field rather than as individual typed exports. |
| `subgraph` | Surfaced: totals, recent daily and hourly volume, and arbitrary GraphQL query execution. The native builder-level routing override (`SubgraphApi::with_config_override`) is a construction-time concern rather than a separate surfaced operation. |
| `contracts` | Surfaced for the consumer-relevant surface: settlement and eth-flow event-log decoders, deployment-address lookup, and the eth-flow and settlement calldata used by the transaction builders. The low-level encoding and verification surface (raw order hashing and UID packing, signature codecs and on-chain EIP-1271 verification, wrapped-native wrap/unwrap interactions, and interaction normalization) is internal building-block code and is not a consumer API on any target. |

### Non-surfaced capability classification

Every native capability without a `cow-sdk-wasm` export falls into one of five
classes.

**Class 1 — Runtime-model boundary.** `cow-sdk-wasm` is a callback leaf. The
JavaScript host owns the wallet, the event loop, and the RPC provider, so the
crate exposes unsigned transaction builders rather than managed broadcast or
receipt-polling flows, and the native Alloy adapter crates
(`cow-sdk-alloy`, `cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`) are
native-only and cannot compile for `wasm32`. ADR 0039 holds the wasm32
dependency tree free of those adapters, browser-wallet, reqwest, and hyper.
Members: the managed-flow counterparts of pre-sign, on-chain and off-chain
cancellation, and approval; and `poll_for_receipt` /
`submit_and_wait_for_receipt`.

**Class 2 — Outside the defined workflow scope.** These are public read
operations the native crates carry because the orderbook and signing crates
mirror the full upstream service and protocol surface. They are omitted from
`cow-sdk-wasm` because the crate is scoped to the workflow set in
`docs/parity.md`, which routes full-feature consumers to the upstream
TypeScript SDK (ADR 0039). They use the same transport, DTO, and envelope
machinery as the surfaced reads, so each is an additive item rather than a
runtime-model boundary. Members:

- Orderbook reads, eight of which are public methods on the upstream
  `cowprotocol/cow-sdk` `OrderBookApi` (`packages/order-book/src/api.ts`):
  `version`, `order_link`, `order_multi_env`, `tx_orders`,
  `order_competition_status`, `total_surplus`,
  `solver_competition`, and
  `solver_competition_by_tx_hash`. `latest_solver_competition` is a
  native convenience read with no direct upstream `OrderBookApi` method.
- On-chain EIP-1271 signature verification (`verify_eip1271_signature` /
  `verify_eip1271_signature_cached`) and its verification caches, which require
  a chain `Provider` read rather than a service call.

**Class 3 — Internal contract-binding surface.** The low-level `contracts`
encoding and verification surface is building-block code shared by native
tooling; it is not exposed as a consumer API on any target and has no upstream
consumer analogue.

**Class 4 — Deferred capability families.** The composable conditional-order
framework is a deferred capability recorded only by
[ADR 0048](../adr/0048-composable-conditional-order-framework.md); no
composable crate ships in the workspace, and its deployment addresses remain
resolvable through the typed `Registry`. Its absence from `cow-sdk-wasm` is a
deferred-capability boundary on every target rather than a WASM-specific one.
`cow-sdk-contracts` has shipped its helper body and compiles for both native and
`wasm32`, so it is the most direct candidate for a future deterministic-helper
addition to the `cow-sdk-wasm` JavaScript surface; its absence from
`cow-sdk-wasm` today is a binding-surface choice, not a target limitation.
Bridging, the flashloan helper surface, and hook-trampoline chaining are
likewise deferred on every target.

**Class 5 — Separate WASM leaf.** `cow-sdk-browser-wallet` compiles for
`wasm32` and carries its own wallet integration, but it is a distinct leaf and
is intentionally not part of the `cow-sdk-wasm` dependency tree (ADR 0039). The
`cow-sdk-wasm` consumer model for wallets is the typed JavaScript callback
boundary rather than a Rust-side wallet.

### Recorded observations

- **Approval-transaction builder.** `cow-sdk-wasm` exposes transaction builders
  for pre-sign (`buildPresignTx`), cancellation (`buildCancelOrderTx`), and
  native-currency sell (`buildSellNativeCurrencyTx`), and exposes an allowance
  read (`getCowProtocolAllowance`). It does not currently expose an
  approval-transaction builder. The native `trading` crate provides a pure,
  signer-free `approval_transaction` (`crates/trading/src/allowance.rs`) that
  returns a `TransactionRequest`. A WASM builder mirroring the existing
  transaction-builder shape is an additive candidate with no runtime-model
  barrier; it completes the read-allowance-then-approve path for hosts that own
  submission. This item is recorded for tracking and is not a conformance
  defect against the current defined scope.
- **Consumer-relevant Class 2 reads.** Among the Class 2 reads,
  `order_competition_status` and `total_surplus` are the operations a
  host building order-status or surplus presentation would most likely require.
  They remain out of the defined scope; any future decision to support such a
  host on `cow-sdk-wasm` should treat them as the first additions.

## Shape Correspondence

A surfaced capability does not carry the native Rust shape unchanged. The WASM
surface re-shapes every operation through a fixed transform set, so the
divergence is uniform and predictable. The public consumer surface is the
committed TypeScript facade snapshot under `crates/wasm/snapshots/facade/`,
which re-exports the `tsify`-generated DTO types from the raw snapshot under
`crates/wasm/snapshots/raw/` (the `tsify` DTOs under
`crates/wasm/src/exports/dto/` are that snapshot's source). Per ADR 0039 the
raw wasm-bindgen output is a package-internal artifact; the shapes below are the
facade's.

### Systematic transforms

| Concern | Native Rust shape | WASM / TypeScript shape |
| --- | --- | --- |
| Client construction | Typestate builder (`OrderbookApi::builder().chain(..).env(..).transport(..).build()`) | Single typed config object (`new OrderBookClient({ chainId, env?, apiKey?, transport, transportPolicy?, timeoutMs? })`) |
| Capability injection | Generic over the `Signer` / `Provider` / `HttpTransport` traits | JS callbacks (`TypedDataSignerCallback`, `DigestSignerCallback`, `Eip1193RequestCallback`, `CustomEip1271Callback`, `ContractReadCallback`); transport via `HttpTransportConfig = { kind: "fetch" } \| { kind: "callback"; callback: CowFetchCallback }` |
| Operation inputs | Typed structs built with constructors and `with_*` (`TradeParams::new(..).with_slippage_bps(..)`) | Plain input DTO objects with `camelCase` fields (`SwapParametersInput`, `LimitTradeParametersInput`, `OrderQuoteRequestInput`, …) |
| Operation outputs | Typed value `T` carried by `Result<T, E>` | `WasmEnvelope<T> = { schemaVersion: "v1" \| "__unknown"; value: T }` (exceptions below) |
| Atomic amounts | `Amount` (`#[repr(transparent)]` over `U256`, decimal-string serde) | `string` |
| Addresses, UIDs, hashes | `Address`, `OrderUid`, `Hash32`, `AppDataHash`, `HexData` newtypes | `string` (lowercase `0x`-canonical) |
| Free-form JSON | `serde_json::Value` | `Value = unknown` |
| Chain id | `SupportedChainId` / `u32` | `number` (raw EVM chain id) |
| Quote id | `i64` | `number` (validated to the JS safe-integer range) |
| Enumerations | Rust enums | string-literal unions (`OrderKindDto = "sell" \| "buy"`; `SigningSchemeDto = "eip712" \| "ethsign" \| "eip1271" \| "presign"`; …) |
| Per-chain maps | `BTreeMap<_, Address>` (`AddressPerChain`) | `Record<string, string>` |
| Cancellation and timeout | `Option<&ProtocolOptions>` plus a `CancellationToken` | `options?: SdkClientOptions = { signal?: AbortSignal; timeoutMs?: number }`; signing adds `SigningOptions extends SdkClientOptions { walletConfig?: { timeoutMs?: number } }` |
| Errors | Typed `Result<T, OrderbookError \| TradingError \| SigningError \| …>` | Rejected `Promise` carrying `WasmError` (aliased `CowError`): a `kind`-tagged discriminated union, each variant carrying `schemaVersion`, with redacted, lower-cardinality fields |
| Async | `async fn(..) -> Result<T, E>` | `(..) => Promise<WasmEnvelope<T>>`; a native sync helper becomes a sync `(..) => WasmEnvelope<T>` |
| Instance lifetime | released by Rust ownership | the raw class carries `free()`; the public facade class exposes `dispose()`; the host must release it |

### Output-envelope exceptions

Three deterministic helpers return a bare value rather than a `WasmEnvelope`:
`domainSeparator(chainId): string`, `supportedChainIds(): Uint32Array`, and
`wasmVersion(): string`. Every other export follows the `WasmEnvelope<T>` rule,
including the four `SubgraphClient` methods, which the facade types as
`Promise<WasmEnvelope<unknown>>` — the envelope is present, but the payload is
`unknown` (see the divergence note below). The raw generated declaration types
those four as `Promise<any>`; the facade narrows them to the enveloped
`unknown` form.

### DTO-to-native-type correspondence

Each payload that crosses the ABI is a `tsify`-generated DTO that mirrors a
native type under the uniform transforms above. The principal correspondences:

| WASM / TS DTO | Native type |
| --- | --- |
| `OrderInput` | `cow_sdk_core::OrderData` (unsigned order shape) |
| `OrderDto` | `cow_sdk_orderbook::Order` |
| `OrderQuoteRequestInput` / `OrderQuoteResponseDto` / `QuoteDataDto` | `OrderQuoteRequest` / `OrderQuoteResponse` / `QuoteData` |
| `TradeDto` | `cow_sdk_orderbook::Trade` |
| `SwapParametersInput` / `LimitTradeParametersInput` | `cow_sdk_trading::TradeParams` / `LimitTradeParams` |
| `QuoteResultsDto` | `cow_sdk_trading::QuoteResults` |
| `OrderPostingResultDto` | `cow_sdk_trading::OrderPostingResult` |
| `TypedDataEnvelopeDto` | `cow_sdk_core::TypedDataPayload` |
| `TransactionRequestDto` | `cow_sdk_core::TransactionRequest` |
| `SettlementEventDto` / `EthFlowEventDto` | `cow_sdk_contracts::SettlementEvent` / `EthFlowEvent` |
| `DeploymentAddressesDto` | `cow_sdk_wasm::helpers::dto::DeploymentAddresses` |
| `AppDataInfoDto` / `ValidationResultDto` / `AppDataDocInput` | `cow-sdk-app-data` info, validation, and document inputs |

The exhaustive field-level inventory of the generated DTOs is the committed
declaration snapshot, owned by the
[WASM Type Generation Audit](wasm-type-generation-audit.md); this audit records
the correspondence, not every field.

### Representative signature deltas

| Operation | Native Rust | WASM / TypeScript |
| --- | --- | --- |
| Orderbook quote | `OrderbookApi::quote(&OrderQuoteRequest) -> Result<OrderQuoteResponse, OrderbookError>` | `OrderBookClient.getQuote(request: OrderQuoteRequestInput, options?): Promise<WasmEnvelope<OrderQuoteResponseDto>>` |
| Owner orders | `orders(&OrdersQuery) -> Result<Vec<Order>, _>` (request struct carries owner and pagination) | `getOrders(owner: string, pagination?: PaginationOptions, options?): Promise<WasmEnvelope<OrderDto[]>>` (decomposed arguments) |
| Submit order | `send_order(&OrderCreation) -> Result<OrderUid, _>` | `sendOrder(signed: SignedOrderDto, options?): Promise<WasmEnvelope<string>>` (UID as `string`) |
| Sign order | `sign_order(&OrderData, chain, &S: TypedDataSigner, opts) -> Result<SigningResult, _>` | `signOrderWithTypedDataSigner(input: OrderInput, chainId: number, owner: string, typedDataSigner: TypedDataSignerCallback, options?): Promise<WasmEnvelope<SignedOrderDto>>` |
| Managed swap | `Trading::post_swap_order(TradeParams, &S: Signer, opts) -> Result<OrderPostingResult, _>` | `TradingClient.postSwapOrder(params: SwapParametersInput, owner: string, signerCallback: TypedDataSignerCallback, options?): Promise<WasmEnvelope<OrderPostingResultDto>>` |
| Cancellation success | `send_cancellations(&OrderCancellations) -> Result<(), _>` | `cancelOrders(signed: SignedCancellationsInput, options?): Promise<WasmEnvelope<{ cancelled: true }>>` |

### Shape divergences to track

Beyond the uniform transforms, these specific differences are worth tracking:

- **Subgraph response payloads are untyped.** On the facade the four
  `SubgraphClient` methods return `Promise<WasmEnvelope<unknown>>` — enveloped
  like every other client call, but their payload is `unknown`: the native
  `Total` and volume shapes are the only client responses without a typed DTO on
  the TS surface. (The raw generated declaration types them as `Promise<any>`;
  the facade narrows them.)
- **Bare-value helpers.** `domainSeparator`, `supportedChainIds`, and
  `wasmVersion` return values directly, outside the `WasmEnvelope` rule.
- **Decomposed inputs.** `getOrders` and `getOrdersByOwner` split the native
  `OrdersQuery` into `(owner, pagination?)`; `getTrades` accepts the
  combined `TradesQueryInput`, whose exactly-one-of `owner` / `orderUid`
  constraint is a runtime check rather than a type.
- **Owner is an explicit parameter.** Signing and managed-post exports take
  `owner: string` positionally because no Rust `Signer` is present to resolve
  it; the native signer-resolved `quote_results` path has no TS counterpart.
- **Error cardinality is reduced.** Native error enums collapse into the
  `WasmError` union's `kind` set with redacted fields, so a consumer matching
  native variants does not get a one-to-one TS counterpart.
- **`feeAmount` is structurally present but constrained.** Order DTOs surface
  `feeAmount` for EIP-712 struct-hash compatibility, while services accepts only
  `"0"`.
- **Client instances require explicit release.** `free()` / `dispose()` has no
  native analogue.
- **The native fluent swap builder has no TypeScript counterpart.**
  `Trading::swap()` returns a typestate `SwapBuilder` whose `execute` / `submit`
  / `quote` terminals compose the already-surfaced quote-sign-post flow. It is a
  native-only ergonomic wrapper: its `Set` / `Unset` typestate cannot cross the
  wasm-bindgen ABI, and the sell/buy transposition safety it retrofits onto the
  positional `TradeParams::new` constructor is already provided by the
  named-field `SwapParametersInput` DTO. The wasm surface covers the same
  capability through `postSwapOrder`, `postSwapOrderFromQuote`, and `getQuote`,
  so the builder's absence is a shape choice, not a capability gap.

## Evidence

Primary implementation points:

- `crates/wasm/src/exports/`
- `crates/wasm/src/exports/dto/`
- `crates/wasm/snapshots/facade/`
- `crates/wasm/snapshots/raw/`
- `crates/wasm/src/helpers/`
- `crates/orderbook/src/api.rs`
- `crates/trading/src/`
- `crates/signing/src/`

Primary regression coverage:

- `crates/wasm/tests/wasm_surface_contract.rs`
- `crates/wasm/tests/wasm_workflow_coverage_contract.rs`
- `crates/wasm/tests/host_pure_helpers.rs`
- `tests/wasm_dependency_invariant.rs`

Validation surface:

```text
cargo test -p cow-sdk-wasm --test host_pure_helpers
cargo test -p cow-rs-workspace-tests --test wasm_dependency_invariant
wasm-pack test crates/wasm --headless --chrome
node crates/wasm/npm/scripts/verify-exports.mjs
```
