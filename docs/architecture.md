# Architecture

`cow-rs` is a small family of focused crates. The facade crate exists for
ergonomics; the leaf crates own behavior.

```mermaid
flowchart TD
  sdk["cow-sdk"];
  core["cow-sdk-core"];
  contracts["cow-sdk-contracts"];
  signing["cow-sdk-signing"];
  appdata["cow-sdk-app-data"];
  orderbook["cow-sdk-orderbook"];
  transport_policy["cow-sdk-transport-policy"];
  trading["cow-sdk-trading"];
  subgraph_crate["cow-sdk-subgraph"];
  wallet["cow-sdk-browser-wallet"];
  transport_wasm["cow-sdk-transport-wasm"];
  wasm["cow-sdk-wasm"];
  alloy_provider["cow-sdk-alloy-provider"];
  alloy_signer["cow-sdk-alloy-signer"];
  alloy["cow-sdk-alloy"];

  sdk --> core;
  sdk --> contracts;
  sdk --> signing;
  sdk --> appdata;
  sdk --> orderbook;
  sdk --> transport_policy;
  sdk --> trading;
  sdk -.->|feature: browser-wallet| wallet;
  sdk -.->|feature: alloy-provider| alloy_provider;
  sdk -.->|feature: alloy-signer| alloy_signer;
  sdk -.->|feature: alloy| alloy;
  contracts --> core;
  signing --> core;
  signing --> contracts;
  appdata --> core;
  orderbook --> core;
  orderbook --> transport_policy;
  trading --> core;
  trading --> contracts;
  trading --> signing;
  trading --> appdata;
  trading --> orderbook;
  subgraph_crate --> core;
  subgraph_crate --> transport_policy;
  wallet --> core;
  transport_wasm --> core;
  wasm --> core;
  wasm --> contracts;
  wasm --> signing;
  wasm --> appdata;
  wasm --> orderbook;
  wasm --> trading;
  wasm --> subgraph_crate;
  wasm --> transport_wasm;
  alloy_provider --> core;
  alloy_signer --> core;
  alloy_signer --> contracts;
  alloy --> core;
  alloy --> contracts;
  alloy --> alloy_provider;
  alloy --> alloy_signer;
```

## Crate Roles

| Crate | Role | Use when |
| --- | --- | --- |
| `cow-sdk` | Thin public facade | You want the main Rust SDK entrypoint. |
| `cow-sdk-core` | Shared domain types, config, validation, runtime traits, and the `HttpTransport` seam with its native `ReqwestTransport` default | You need the common typed contracts. |
| `cow-sdk-transport-policy` | Shared HTTP retry driver (`run_with_retry`), rate-limit, jitter, `Retry-After`, target-neutral wall clock, and transport classification policy | You need consistent transport behavior across typed clients. |
| `cow-sdk-contracts` | `alloy::sol!`-generated typed bindings, the typed `Registry` deployment authority, fail-closed `CoWSwapOnchainOrders` event decoding, and deterministic hashing and verification helpers | You need ABI-level, address-authority, or settlement-level primitives. |
| `cow-sdk-signing` | Typed-data, signing, cancellation, UID helpers, and the `Eip1271VerificationCache` seam (the always-available `NoopEip1271VerificationCache` plus the feature-gated `InMemoryEip1271VerificationCache`) | You need signing without the full trading layer. |
| `cow-sdk-app-data` | App-data encoding, schema handling, and CID behavior | You need app-data generation or validation. |
| `cow-sdk-orderbook` | Typed orderbook transport over the `HttpTransport` seam, with the `OrderbookApiBuilder` typestate | You need explicit request and response control. |
| `cow-sdk-trading` | Quote-to-order workflows plus the quote, submit, cancel, and approve orchestration surface | You need the main trading orchestration layer. |
| `cow-sdk-subgraph` | Read-only subgraph access over the `HttpTransport` seam, with the `SubgraphApiBuilder` typestate | You need GraphQL reads or custom subgraph queries. |
| `cow-sdk-transport-wasm` | Browser-target `HttpTransport` implementation (`FetchTransport`) | You build for `wasm32-unknown-unknown` and need the shipped browser default. |
| `cow-sdk-wasm` | TypeScript-callable wasm-bindgen bindings over deterministic SDK helpers, typed callbacks, orderbook/subgraph/IPFS clients, and trading flows | JavaScript or TypeScript should call the Rust SDK through wasm exports. |
| `cow-sdk-browser-wallet` | Browser-runtime wallet integration | You need EIP-1193 wallet flows in WASM. |
| `cow-sdk-alloy-provider` | Native Alloy-backed `Provider` adapter | You need read-only chain RPC through Alloy without a signer dependency. |
| `cow-sdk-alloy-signer` | Native Alloy-backed local private-key `Signer` adapter | You need local message or EIP-712 signing without provider-backed transaction submission. |
| `cow-sdk-alloy` | Composed native Alloy provider plus signer adapter | You need one native client for `Provider`, `LogProvider`, `SigningProvider`, and `Signer` helper flows. |
| `cow-sdk-composable` | Reserved manifest for composable-order helpers, with current readiness evidence owned by contracts, signing, docs, and parity fixtures | You need to track the planned composable leaf without pulling an unfinished helper API. |
| `cow-sdk-cow-shed` | COW Shed account-abstraction proxy derivation, EIP-712 hook signing, and `executeHooks` calldata encoding for EOA and EIP-1271 owners, with the `CowShedHooks` orchestrator | You need the COW Shed account-abstraction surface. |

The `cow-sdk-composable` reserved manifest is not a workspace member yet and
does not expose a crate body. It keeps package identity, MSRV, and evidence
paths stable while the shared registry, ABI, signature, and documentation
surfaces remain reviewable.

## Layering

| Layer | Crates | Responsibility |
| --- | --- | --- |
| Foundation | `cow-sdk-core` | Shared domain model, runtime seams, and the `HttpTransport` trait |
| Deterministic protocol transforms | `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data` | Typed bindings, registry authority, hashing, signing, app-data, and compatibility logic |
| Client policy | `cow-sdk-transport-policy` | Shared retry, cooldown, rate-limit, and classification behavior above the raw transport seam |
| Client | `cow-sdk-orderbook`, `cow-sdk-subgraph` | Typed HTTP and GraphQL access through the `HttpTransport` seam |
| Workflow | `cow-sdk-trading` | Quote, submit, cancel, approve, and related flows |
| Runtime adapter | `cow-sdk-browser-wallet`, `cow-sdk-transport-wasm`, `cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, `cow-sdk-alloy` | Browser-wallet session integration, browser-target HTTP transport, and opt-in native Alloy provider/signer adapters |
| TypeScript WASM leaf | `cow-sdk-wasm` | Typed wasm-bindgen exports and JavaScript callbacks over the same protocol helpers and HTTP seams |
| Facade | `cow-sdk` | Curated public entrypoint |

## TypeScript-Callable WASM Surface

`cow-sdk-wasm` is a peer leaf, not a replacement for the native facade and
not a replacement for the upstream `@cowprotocol/cow-sdk` TypeScript SDK.
For most browser dapps and standard TypeScript applications, the upstream
TypeScript SDK is the recommended choice because of its smaller bundle size
at equivalent feature subsets. `cow-sdk-wasm` is appropriate for specialized
cases — deterministic Rust signing parity, single-source-of-truth Rust +
TypeScript embedding, and Cloudflare Workers (size-compatible at the time of
measurement; full Workers support pending release-bundle and startup
validation).

Its surface has four layers: pure helpers for deterministic protocol output,
wallet and signer callback exports, orderbook plus subgraph plus IPFS clients,
and trading clients. The crate reuses the same Rust helpers that native
consumers call, then crosses into JavaScript only at typed wasm-bindgen
exports and callbacks.

```mermaid
flowchart TD
  pure["Layer 1: pure helpers"];
  callbacks["Layer 2: wallet and signer callbacks"];
  clients["Layer 3: orderbook, subgraph, and IPFS clients"];
  trading_wasm["Layer 4: trading clients"];
  js_host["JavaScript runtime"];

  pure --> callbacks;
  callbacks --> clients;
  clients --> trading_wasm;
  js_host -.-> callbacks;
  js_host -.-> clients;
```

<!-- runtime-routing:start -->
## Choose the crate or package by runtime

| You're building... | Use | Why |
| --- | --- | --- |
| Native Rust services, bots, solvers, analytics | `cow-sdk` | Native HTTP transport, signing, trading, orderbook, and subgraph surfaces. |
| Native Rust apps using Alloy | `cow-sdk` plus `cow-sdk-alloy-*` | Opt-in Alloy provider and signer adapters without widening the default facade. |
| Rust apps that compile to browser WASM | `cow-sdk-browser-wallet` plus `cow-sdk-transport-wasm` | Rust-on-wasm wallet and fetch plumbing; not the JavaScript-callable package. |
| Standard browser dapp or CowSwap-style UI in TypeScript | Upstream [`@cowprotocol/cow-sdk`](https://www.npmjs.com/package/@cowprotocol/cow-sdk) | Substantially smaller bundle at equivalent feature subsets; mature web ecosystem fit. |
| TypeScript apps that need byte-for-byte Rust signing parity (viem, ethers, wagmi, or EIP-1193 wallets) | `<published-cow-sdk-wasm-package>` | Wallet stack-agnostic callbacks and the full facade surface. |
| Browser dapps with a smaller bundle target | `<published-cow-sdk-wasm-package>/orderbook` | Orderbook and signing subset with a smaller raw wasm budget. |
| Signer services or HSM proxies | `<published-cow-sdk-wasm-package>/signing` | Signing, UID, EIP-1271, and deployment helpers without HTTP clients. |
| Node.js 22 or 24 LTS backends | `<published-cow-sdk-wasm-package>` | Node target works without browser polyfills when transport is configured. |
| Cloudflare Workers | `<published-cow-sdk-wasm-package>/cloudflare` plus `<published-cow-sdk-wasm-package>/cloudflare/wasm` | Worker-compatible web target with explicit module initialization. Size-compatible with current Workers Free compressed-size limit at the time of measurement; full Workers support pending release-bundle and startup validation. |
| Deno | `<published-cow-sdk-wasm-package>` | Experimental build-only support; validate in your own runtime before production use. |
| Non-JS wasm consumers, WASI, WebAssembly components, TinyGo, Blazor, AssemblyScript guests, or no_std | Out of scope for 0.1.0 | Use native Rust crates where possible; the npm package targets JavaScript hosts. |
<!-- runtime-routing:end -->

## TypeScript facade architecture

After publication, the npm package wraps raw wasm-bindgen output behind a
TypeScript facade. Public consumers import from the resolved package name or a
flavor subpath, while generated wasm-bindgen modules remain package-internal.

The facade exposes camelCase TypeScript APIs, named callback types, explicit
`dispose()` methods on client classes, and normalized `SdkError` values. It
also adapts `transport: { kind: "fetch" }` into the callback transport ABI so
browser, Node.js, and Worker consumers can use the same public constructor
shape.

## Facade And Adapter FAQ

### Why `cow-sdk-subgraph` is not part of the default facade

`cow-sdk` stays narrow on purpose. The default facade is the trading-first SDK
entrypoint, while `cow-sdk-subgraph` remains an explicit read-only analytics
crate. Keeping subgraph access separate avoids widening the default dependency
graph for consumers that only need order creation, signing, quoting, and
submission. This matches [ADR 0001](adr/0001-multi-crate-sdk-family-with-thin-facade.md)
and [ADR 0003](adr/0003-separate-read-only-subgraph-crate.md): the facade
optimizes for the main transactional path, and analytics stay opt-in.

### Provider And Signer Adapter Seams

Native runtime integrations plug in through the stable traits owned by
`cow-sdk-core`:

```rust
use cow_sdk_core::{Provider, Signer, SigningProvider};
```

The same seam also owns the transaction lifecycle boundary. Signers return
`TransactionBroadcast`, a hash-only broadcast acknowledgement, while provider
receipt lookups return `TransactionReceipt` with optional mined-state fields
such as status, block, gas, sender, and recipient. Adapter implementations must
not turn submission into implicit receipt polling; mined observation stays an
explicit provider call.

The SDK declares its provider, signer, and signing-provider contracts in
`cow-sdk-core` rather than binding trading helpers directly to a concrete
Ethereum runtime library. This lets one trading call site drive native Alloy on
x86 / ARM, the browser-wallet leaf on `wasm32`, or any custom adapter that
implements the same traits. If `cow-sdk-trading` depended on a concrete
provider library directly, the wasm path would have to pull native-only
dependencies or fork trading helpers per runtime.

For custom integrations, implement `Provider` for the RPC backend and `Signer`
for the signer backend; the narrower [`Owner`], [`TypedDataSigner`], and
[`DigestSigner`] capability traits per
[ADR 0045](adr/0045-async-signer-trait-narrowing.md) remain available for
callback-shaped adapters that expose only one signing operation.
Wallet-capable adapters implement `SigningProvider`, which extends `Provider`
with signer creation. Adapters that can fetch event logs additionally implement
`LogProvider`, the opt-in log-fetch capability supertrait
([ADR 0057](adr/0057-log-provider-capability-trait.md)); its `get_logs` performs
a single bounded query — over an address set and the four EVM topic slots, so
indexed arguments such as an event's `owner` filter server-side — that feeds the
fail-closed event decoders.
Native Alloy support is already shipped as
`cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, and `cow-sdk-alloy`;
browser-wallet support implements the same traits directly without widening
the native facade.

The native Alloy adapter family ships as three crates so a consumer can pull
only the capabilities they exercise: `cow-sdk-alloy-provider` for read-only
RPC, `cow-sdk-alloy-signer` for local private-key signing, and `cow-sdk-alloy`
for the composed read, log-fetch, and sign flow that most trading applications
need. The composed client implements `LogProvider` over the same provider it
already holds, so a consumer fetches event logs without a second provider. The
split keeps the provider leaf free of signing-crypto features and lets the
signer leaf stay free of transport plumbing.

The stable public contract is the trait seam itself. Native signer and RPC
integrations remain additive leaf crates so the workspace does not freeze one
provider ecosystem into `core`, `trading`, or the default `cow-sdk` facade.
Use [Integrations](integrations.md) for a worked adapter example.

## Cross-Cutting Contracts

### Primitive Layer

`alloy_primitives` is the canonical EVM primitive layer and `alloy_sol_types`
is the canonical EIP-712 / Solidity-binding layer across the workspace per
[ADR 0052](adr/0052-alloy-primitives-canonical-primitive-layer.md). The
cow-named identity and numeric public types are cow-owned
`#[repr(transparent)]` newtypes around the corresponding `alloy_primitives`
type; `TypedDataDomain` is a cow-owned `#[non_exhaustive]` struct that emits
the canonical EIP-1193 wire shape through its own `Serialize` impl and
bridges to `alloy_sol_types::Eip712Domain` via an `into_alloy_domain()`
adapter at the EIP-712 hashing seam.

### Runtime Traits

`cow-sdk-core` owns the signer and provider seams used across the workspace.
The trait surface is async by construction, and typed-data payloads stay
structured rather than being reconstructed from ad hoc field lists.
Credential-bearing
config stays explicit as input, but the default diagnostic and serialized
surfaces owned by `cow-sdk-core`, `cow-sdk-orderbook`, and `cow-sdk-app-data`
redact secret material instead of treating it as routine log data.
Transaction broadcast and receipt observation stay separate typed results so
callers can reason about submission, inclusion, and execution without
provider-specific timing assumptions.

### Typed Amounts

`cow-sdk-core` keeps two distinct amount roles at the typed boundary.
`Amount` and `SignedAmount` are cow-owned `#[repr(transparent)]` newtypes
over `alloy_primitives::U256` and `alloy_primitives::I256` respectively per
[ADR 0052](adr/0052-alloy-primitives-canonical-primitive-layer.md),
preserving the decimal-string wire form. `Amount` carries the atomic
(base-unit) quantity that crosses the wire; human-readable token amounts
convert in and out through `Amount::from_units(whole, decimals)` for whole
amounts, `Amount::parse_units(value, decimals)` for fractional or textual
input, and `Amount::format_units(decimals)`, which scale by `10^decimals`
with integer arithmetic (never floating point) so the round trip stays exact. Both numeric
newtypes
expose a fallible-by-return arithmetic surface — `checked_*` (returning
`Option`) and explicit `saturating_*` clamps — and intentionally ship no
bare `Add` / `Sub` / `Mul` operators, so an overflow or underflow can
never silently wrap (or panic) on a typed amount; a caller that needs raw
wrapping reaches through `as_u256` / `into_u256`.
The cow-owned `Deserialize` impl is strict-decimal-only on the wire
boundary; the cow `Amount::new` and `SignedAmount::new` constructors stay
lenient to preserve the existing constructor contract.

### Transport Seams

`cow-sdk` exposes two orthogonal runtime seams that never share a concrete
backend. The `HttpTransport` trait in `cow-sdk-core` is the HTTPS seam used
by `cow-sdk-orderbook` and `cow-sdk-subgraph` for REST and GraphQL
dispatch; native consumers get `ReqwestTransport` from `cow-sdk-core`, and
browser consumers get `FetchTransport` from the dedicated
`cow-sdk-transport-wasm` leaf crate. The shared retry driver
(`run_with_retry`) plus retry, cooldown, rate-limit, target-neutral wall
clock, and transport-error classification policy lives in
`cow-sdk-transport-policy`, so the orderbook, subgraph, and IPFS clients run
every attempt through one retry loop and keep the same behavior on native
and browser targets without widening the raw `HttpTransport` trait. The `Provider` trait (also in `cow-sdk-core`)
is the read-only chain-RPC seam used by on-chain helpers such as allowance
reads, EIP-1271 verification, and on-chain cancellation. Signer creation for
wallet-capable providers lives in `SigningProvider`; no provider implementation
ships by default, so consumers bring their own through the
[Providers](providers/README.md) adapter guide.

The trait is dyn-compatible, so injected clients compose transports behind
`Arc<dyn HttpTransport + Send + Sync>`. Typed failures flow through a single
`TransportError` enum and its `TransportErrorClass` partition, both of
which strip URLs before wrapping so credential-bearing query strings never
surface through `Debug` or `Display`. The full transport story lives in
[Transport](transport.md).

### Transport Ownership

Retry behavior, rate limits, GraphQL request shape, and API-key handling
stay with the transport crates that own those behaviors.
For subgraph access, stable production metadata and typed request failures
expose only redacted or non-secret route identity while keeping explicit
override support.

Production deployments that issue requests across several chains can pool
a single `reqwest::Client` across every orderbook and subgraph instance
they build. On native targets, `OrderbookApi::builder()` and
`SubgraphApi::builder()` both expose a `.client(shared_client)` convenience
method over `ReqwestTransport` that preserves any custom keep-alive,
timeout, or TLS settings verbatim, so one warm connection cache backs
every chain the consumer routes through. Browser consumers install
`FetchTransport` through the builder's `.transport(...)` setter instead.
The [Performance](performance.md) page records the recommended HTTP/2
keep-alive recipe, shared-client usage pattern, and the knob summary that
accompanies each opt-in setting.

### Cancellation

Long-running public operations on `OrderbookApi`, `SubgraphApi`, and
`Trading` each expose one canonical async method, and callers compose
cooperative cancellation by wrapping the returned future through
`cow_sdk_core::Cancellable::cancel_with(&token)` at the call site. The
`cow_sdk_core::CancellationToken` is a re-export of
`tokio_util::sync::CancellationToken`, so every SDK surface routes
cancellation through the same typed import. The combinator polls the
borrowed token in a biased branch before each inner poll; when the token
fires, the wrapper drops the inner request future so the underlying
socket releases promptly, and the typed `Cancelled` variant on the
relevant error aggregate surfaces at the caller.

For example, the following quotes and posts a swap with a shared
cancellation token:

```rust,ignore
use cow_sdk_core::Cancellable;

let token = cow_sdk_core::CancellationToken::new();
let result = sdk
    .post_swap_order(params, &signer, None)
    .cancel_with(&token)
    .await?;
```

Cancellation is cooperative: the caller owns the token, and every SDK
instance that needs to propagate shutdown through a shared token simply
clones it. `From<Cancelled>` bridges on `CoreError`, `OrderbookError`,
`SubgraphError`, `TradingError`, `SigningError`, `BrowserWalletError`, the
native Alloy adapter errors, and the facade `SdkError` lift the marker through
`?` across every public error boundary.

### Workflow Ownership

`cow-sdk-trading` owns quote-to-order orchestration. It composes lower-level
crates instead of spreading user-facing workflow logic across signing,
transport, and contract crates. When callers inject an orderbook client into
orderbook-bound trading helpers, that client becomes the canonical chain and
environment authority; conflicting explicit values are rejected instead of
being silently mixed through precedence fallbacks. When quote results are
reused for posting, the originating orderbook runtime binding remains part of
that contract, so quote-derived submission is rejected if the caller switches
to a different orderbook endpoint, chain, or environment. Reviewed
`sellTokenBalance` and `buyTokenBalance` semantics remain part of the same
workflow contract through quote, order, sign, and post seams. The typestate
builder and its total-input shortcuts share the same injected-orderbook
validation boundary. Ready-state `Trading` construction requires a validated
`AppCode` plus explicit or injected chain authority; chain-bound helpers that
need no app code — allowance, approval, pre-sign, and on-chain cancellation —
are the crate's free functions. Recoverable-signature posting
rejects explicit owner or signer mismatch before submission, and user-facing
partner-fee policy remains typed on trading request surfaces and only crosses
into raw metadata at the explicit app-data translation seam.

For browser-wallet-backed trading flows, chain coherence remains leaf-owned by
`cow-sdk-browser-wallet`. When the workflow already has an explicit chain
authority, `BrowserWallet::signer_for_chain` binds that expectation to the
wallet session so quote, address, signature, gas, and transaction operations
fail fast if the active wallet chain drifts.

Typed browser-wallet chain-management follows the same rule. Successful
`switch_chain` and `switch_or_add_chain` results are returned only after the
refreshed wallet session confirms the requested chain, so switch helpers do
not treat wallet RPC acknowledgement as sufficient authority on its own.

### Browser-Runtime Support

Browser wallet support is a leaf capability, not a hidden default. The root
facade exposes it through an explicit feature, while the full browser-runtime
contract remains owned by `cow-sdk-browser-wallet`. Chain-bound browser-wallet
signers keep live wallet flows aligned with the selected workflow chain without
widening `cow-sdk-trading` into a browser-specific crate, and typed
chain-management helpers confirm refreshed session state before they report
switch success.

## Public Boundary Rules

- `cow-sdk` stays thin.
- Pure transform crates do not perform hidden network I/O.
- `cow-sdk-subgraph` remains a separate read-only crate.
- Browser-wallet method growth stays leaf-owned and typed.
- Orderbook wire DTOs remain string-heavy only at the explicit HTTP boundary.
- Public configs, endpoint discovery, and typed request failures expose only
  redacted or non-secret route identity.
- Reviewed subgraph query constants may be public when they are deliberately
  stabilized, but saved GraphQL breadth beyond that reviewed set and test-only
  schema fixtures stay non-public.
- `OrderbookApi`, `SubgraphApi`, and `Trading` construct exclusively
  through their typestate builders; no free-function public constructors
  remain on any of the three.
- Native Alloy dependencies are confined to the reviewed opt-in adapter crates:
  `alloy-provider` is allowed only in `cow-sdk-alloy-provider` and
  `cow-sdk-alloy`, while `alloy-signer-local` is allowed only in
  `cow-sdk-alloy-signer` and `cow-sdk-alloy`. The default facade stays
  provider-neutral unless an Alloy feature is enabled.
- Every deployed-contract-address lookup routes through the typed
  `Registry` authority; hard-coded chain-scoped address constants are not
  allowed in shipped crates.
- Every ABI binding the SDK emits call-data against is generated through
  `alloy::sol!` from byte-identical upstream Solidity mirrors committed
  under `crates/contracts/abi/` and gated by
  `cargo parity-verify-sol-provenance` against SHA-256 rows in
  `parity/source-lock.yaml`.
- On-chain order event logs (`CoWSwapOnchainOrders` `OrderPlacement` /
  `OrderInvalidation`) are decoded through a fail-closed, provider-free decoder
  that validates every field and never panics on adversarial input.

## Related Docs

- [Principles](principles.md)
- [Transport](transport.md)
- [Deployments](deployments.md)
- [Verification Guide](verification-guide.md)
- [Parity Matrix](parity-matrix.md)
- [ADRs](adr/README.md)
