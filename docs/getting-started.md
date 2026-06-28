# Getting Started

`cow-rs` is a trading-first Rust SDK for CoW Protocol.

This guide is the canonical first-touch path for SDK consumers who want to:

1. understand the public crate surface
2. build a ready-state `Trading`
3. verify the deterministic signing contract
4. verify a simulated order-submission flow without live services
5. branch into the maintained native and WASM example families

The sequence below stays deterministic on purpose.

It does not depend on:

- a live orderbook session
- a wallet extension
- browser-hosted pages

If you need provider or signer integration guidance while you read this page,
use [Integrations](integrations.md).

## Install

`cow-sdk` is published on crates.io. It is an alpha, so pin the pre-release:

```toml
[dependencies]
cow-sdk = "0.1.0-alpha.8"
```

`cargo add cow-sdk@0.1.0-alpha.8` does the same. To run this guide's
deterministic checks against a local checkout instead, depend on the crate by
path and run the commands from the repository root:

```toml
[dependencies]
cow-sdk = { path = "/path/to/cow-rs/crates/sdk" }
```

## What The Root Crate Exposes

`cow-sdk` is the thin facade crate.

It re-exports the main public surface for:

- shared core and config types
- signing helpers
- contracts helpers
- app-data helpers
- typed orderbook client types
- trading orchestration

The root facade re-exports `cow-sdk-subgraph` only behind the off-by-default
`subgraph` feature (`cow-sdk = { features = ["subgraph"] }`, surfaced as
`cow_sdk::subgraph`).

Read-only subgraph access otherwise stays in the separate `cow-sdk-subgraph`
crate.

Browser and wallet integration is served to JavaScript and TypeScript
consumers by the `cow-sdk-js` package together with the host app's own wallet
stack (viem, wagmi, or any EIP-1193 provider). The host wraps its EIP-1193
provider into the SDK's typed-data signer callback to supply the wallet connection.

Native Alloy support is additive as well. Use `cow-sdk-alloy-provider` for
read-only RPC, `cow-sdk-alloy-signer` for local private-key signing, and
`cow-sdk-alloy` when one native client should satisfy both provider and signer
helper paths. The facade exposes those surfaces behind the `alloy-provider`,
`alloy-signer`, and `alloy` features on native targets.

Shared validation and configuration failures surface under the canonical
`CoreError` type from `cow-sdk-core`.

That split matters when you choose where to start:

- use `cow-sdk` for the main trading-first path
- use `cow-sdk-subgraph` (directly, or through the `cow-sdk` `subgraph` feature) when you need explicit GraphQL reads
- use `cow-sdk-js` when TypeScript or JavaScript code should call the Rust
  SDK through wasm-bindgen exports, including browser-wallet flows driven by the
  host's own EIP-1193 provider

On `wasm32-unknown-unknown`, the orderbook and subgraph builders auto-select
the shipped browser-target HTTP transport. When you need an explicit instance,
`cow-sdk-core` exports `FetchTransport` (the browser sibling of
`ReqwestTransport`) from its `transport::fetch` module; install it on the
orderbook and subgraph builders through `.transport(...)` as
`Arc<dyn HttpTransport + Send + Sync>`.

For the rest of this guide, stay on the default `cow-sdk` facade on a
native target.

## Using cow-sdk-js From TypeScript

For most browser dapps, web apps, and CowSwap-style UIs, the upstream
[`@cowprotocol/cow-sdk`](https://www.npmjs.com/package/@cowprotocol/cow-sdk)
TypeScript SDK is the recommended choice. It is substantially smaller at
equivalent feature subsets.

`cow-sdk-js` is appropriate for specialized cases — TypeScript services
that need byte-for-byte parity with the Rust SDK's signing path,
single-source-of-truth Rust + TypeScript embedding, and Cloudflare Workers
(size-compatible at the time of measurement; the `trading` flavour's edge build is built
and tested end-to-end in CI). It is published to npm as
`@symbiome-forge/cow-sdk-wasm`, exposing the wasm-bindgen surface through a
TypeScript facade, typed callbacks, per-flavor package exports, and
runtime-specific wasm artifacts.

```text
npm install @symbiome-forge/cow-sdk-wasm@0.1.0-alpha.8
```

## Choose the crate or package by runtime

The canonical runtime-to-package routing table lives in the root README:
[When to use cow-rs](../README.md#when-to-use-cow-rs).

The WASM package keeps wallet libraries outside the Rust crate. Supply typed
JavaScript callbacks for typed-data signing, digest signing,
custom EIP-1271 signatures, and HTTP fetch dispatch.

## Step 1: Build A Ready-State `Trading`

The ready-state builder contract is intentionally small.

`Trading::builder().build()` is only reachable after:

- a default `chainId`
- a stable `appCode`

A default `owner` is optional at build time.

You can set it on the builder when it is convenient, or you can provide the
owner or signer later at the workflow call site.

Use `appCode` as the stable identifier for the application or integration
surface that will own the orders.

Upstream CoW SDK docs and examples follow the same rule: `appCode` is the
application identifier used for order tracking, so it should read like a
durable app name and stay stable across orders from that surface.

Good examples include values such as `acme-trader-web`,
`acme-trader-backend`, or `ops-rebalancer`.

Minimal ready-state builder:

```rust
use cow_sdk::core::SupportedChainId;
use cow_sdk::trading::Trading;

fn build_trading() -> Result<Trading, Box<dyn std::error::Error>> {
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("your-app-code")
        .build()?;

    Ok(trading)
}
```

The owner field belongs on the per-trade `TradeParams` or
`LimitTradeParams`, not on the `Trading` client. The `Trading` client
does not store a default owner. For signer-backed flows (`post_swap_order`, `post_limit_order`,
`quote_results`), the signer's address fills the slot when
`TradeParams.owner` is `None`. For quote-only flows
(`quote_only`), the owner must come from `TradeParams.owner` or
from `TradeAdvancedSettings::quote_request.from`.

```rust
use cow_sdk::core::{address, Address, Amount, OrderKind};
use cow_sdk::trading::TradeParams;

// Compile-time validated address literals — the lowercase wire form, no runtime
// parse and no unwrap.
const USDC: Address = address!("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
const DAI: Address = address!("0x6b175474e89094c44da98b954eedeac495271d0f");

fn quote_request(owner: Address) -> Result<TradeParams, Box<dyn std::error::Error>> {
    let params = TradeParams::new(
        OrderKind::Sell,
        // USDC (6 decimals) sold for DAI.
        USDC,
        DAI,
        Amount::from_units(100, 6)?,
    )
    .with_owner(owner)
    .with_slippage_bps(50);

    Ok(params)
}
```

### Fluent Swap Lifecycle

For the common swap path, `Trading::swap()` opens a typed builder with named
token setters, so the sell and buy tokens cannot be transposed. Supply the sell
token, buy token, and an amount, then either `execute(&signer)` to quote, sign,
and post in one call, or `quote(&signer)` to inspect the quote before
`submit(&signer)`. The owner defaults to the signer's address. The same chain
drives any signer — a local key, a remote signer, a browser wallet, or a smart
account.

```rust,ignore
use cow_sdk::core::{Address, Amount, Signer, UserRejection};
use cow_sdk::trading::Trading;

async fn place_swap<S>(
    trading: &Trading,
    signer: &S,
    sell: Address,
    buy: Address,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: Signer,
    S::Error: std::fmt::Display + UserRejection,
{
    // One call quotes, signs, and posts.
    let posted = trading
        .swap()
        .sell_token(sell)
        .buy_token(buy)
        .sell_amount(Amount::from_units(100, 6)?)
        .slippage_bps(50)
        .execute(signer)
        .await?;
    println!("posted {}", posted.order_id);

    // Or inspect the quote first, then submit the exact quoted order.
    let quoted = trading
        .swap()
        .sell_token(sell)
        .buy_token(buy)
        .sell_amount(Amount::from_units(100, 6)?)
        .quote(signer)
        .await?;
    let _costs = quoted.results().amounts_and_costs.clone();
    let _posted = quoted.submit(signer).await?;

    Ok(())
}
```

The flat `post_swap_order`, `quote_only`, and `post_swap_order_from_quote`
entries remain available for callers that prefer to assemble `TradeParams`
directly.

### Fluent Limit Lifecycle

A limit order sets an explicit price — both the sell and buy amounts — so no quote is
fetched. `Trading::limit()` opens the same kind of typed builder as `swap()`: named
`sell_token`/`buy_token` and `sell_amount`/`buy_amount` setters that cannot be transposed
(a transposed amount would silently invert the price), then `post(&signer)` to sign and
post, or `post_presign()` for the smart-account path that needs no signer — there an
explicit `owner` identifies the account.

```rust,ignore
use cow_sdk::core::{Address, Amount, Signer, UserRejection};
use cow_sdk::trading::Trading;

async fn place_limit<S>(
    trading: &Trading,
    signer: &S,
    sell: Address,
    buy: Address,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: Signer,
    S::Error: std::fmt::Display + UserRejection,
{
    // Sell exactly 100 USDC, want at least 99 DAI.
    let posted = trading
        .limit()
        .sell_token(sell)
        .buy_token(buy)
        .sell_amount(Amount::from_units(100, 6)?)
        .buy_amount(Amount::from_units(99, 18)?)
        .post(signer)
        .await?;
    println!("posted {}", posted.order_id);

    Ok(())
}
```

The flat `post_limit_order` and `post_limit_order_presign` entries remain available for
callers that assemble `LimitTradeParams` directly. Native-currency (EthFlow) sells go
through `swap()` instead — set the native-currency sell token and the swap path
auto-routes to the on-chain EthFlow transaction; `limit()` is for ERC-20 limit orders and
rejects a native-currency sell token.

### Browser Ready-State Wiring

On `wasm32-unknown-unknown`, the ready-state trading API is the same, but
the browser cannot use the native default HTTP transport. The orderbook and
subgraph builders auto-select the browser transport on this target, so the
default path needs no wiring. To inject an explicit instance, build an orderbook
client with `cow_sdk_core::FetchTransport` and pass it once through the builder's
`orderbook` setter:

```rust,ignore
use std::sync::Arc;

use cow_sdk::core::{CowEnv, SupportedChainId};
use cow_sdk::orderbook::OrderbookApi;
use cow_sdk::trading::Trading;
use cow_sdk::http::HttpTransport;
use cow_sdk_core::{FetchTransport, FetchTransportConfig};

fn build_browser_ready_trading() -> Result<Trading, Box<dyn std::error::Error>> {
    let transport: Arc<dyn HttpTransport + Send + Sync> = Arc::new(FetchTransport::new(
        &FetchTransportConfig::new("https://api.cow.fi"),
    ));
    let orderbook = OrderbookApi::builder()
        .chain(SupportedChainId::Sepolia)
        .env(CowEnv::Prod)
        .transport(transport)
        .build()?;

    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("your-browser-app-code")
        .orderbook(orderbook)
        .build()?;

    Ok(trading)
}
```

### Chain-Bound Helpers Without an App Code

Allowance reads, approval submission, pre-sign transaction construction, and
on-chain cancellation need chain authority but no app code. Call the crate's
free functions directly — `cow_protocol_allowance`, `approval_transaction`,
`pre_sign_transaction`, and `onchain_cancel_order` — so an integration such
as an allowance/approval screen or a pre-sign tool needs no trading client at
all. Quote, post, order lookup, and off-chain cancellation flows use the ready
`Trading` client built with `build()`.

### What This Step Proves

This builder step proves the top-level SDK contract:

- the facade entrypoint is `Trading::builder()`
- `SupportedChainId` is the public chain selector type
- `appCode` is a required ready-state default and a stable integration
  identifier
- `build()` returns the ready `Trading` client; chain-bound helpers that
  need no app code are the crate's free functions
- `Address::new(...)` is the public validated address constructor
- `CoreError` is the canonical shared validation and configuration error type

This step does **not** yet prove signing, quoting, or transport behavior.

Those proofs come from the maintained scenarios below.

## EthFlow orders need a quote ID

Native-sell / EthFlow posting requires the quote identifier returned by the
orderbook. The `swap_params_to_limit_order_params` bridge produces a
`LimitTradeParamsFromQuote` value that guarantees the quote identifier
is present by construction, and the EthFlow native-currency submission
helper and transaction helper accept only that newtype on their public
entries. In the snippet below, `trading`, `orderbook`, `trader`, and `signer` are
the values built in the ready-state steps above, and `params` describes the
native-sell trade. The `ethflow` scenario runs this
flow end to end:

```rust,ignore
use cow_sdk::trading::{
    eth_flow_transaction, post_sell_native_currency_order,
    swap_params_to_limit_order_params, PostTradeAdditionalParams,
};

let quote = trading.quote_results(params.clone(), signer, None).await?;
let limit_from_quote = swap_params_to_limit_order_params(
    &quote.trade_parameters,
    &quote.quote_response,
)?;
let order = post_sell_native_currency_order(
    orderbook,
    &quote.app_data_info,
    &limit_from_quote,
    &PostTradeAdditionalParams::default(),
    trader,
    signer,
    None,
)
.await?;
```

If the orderbook quote response does not carry an identifier,
`swap_params_to_limit_order_params` fails with
`TradingError::MissingQuoteId("EthFlow order posting")` before the
native-currency transaction is built. The typed boundary lifts the
previous runtime check to a compile error when a consumer attempts to
pass a `LimitTradeParams` value missing a quote id directly to the
EthFlow entries.

## Step 2: Run The Deterministic Signing Scenario

Run the maintained signing scenario:

```text
cargo run -p cow-sdk-examples-native --example sign_order
```

This scenario is the shortest deterministic proof that the SDK can:

- build typed order-signing payloads
- derive the canonical order digest
- derive the canonical order id
- produce typed cancellation-signature output

It does not require:

- a live orderbook
- a browser wallet
- a deployed console page
- custom RPC infrastructure

### Current Example Output

A successful run from the committed example currently prints:

```json
{
  "surface": "cow_sdk::signing",
  "mode": "deterministic",
  "order": {
    "primaryType": "Order",
    "digest": "0x413343185c9b2b0acd540f33480c81881e1dc5f7ee98c93953383481eb1a5a01",
    "orderId": "0x413343185c9b2b0acd540f33480c81881e1dc5f7ee98c93953383481eb1a5a017e5f4552091a69125d5dfcb7b8c2659029395bdf6553f100",
    "signature": "0xdb77185465fa462491714e0a724bbdcef6bda36a1f8fec772f773377ecf79052729213692c1a99c36626dc46dd30fac941a79c0ee242ca24e5bad48967cd78ea1b",
    "scheme": "Eip712",
    "eip1271PayloadPrefix": "0x0000000000000000"
  },
  "cancellation": {
    "orderUid": "0xd64389693b6cf89ad6c140a113b10df08073e5ef3063d05a02f3f42e1a42f0ad0b7795e18767259cc253a2af471dbc4c72b49516ffffffff",
    "signature": "0x31c037bec6455a1caffc876eb4a46b2e818eeec3bec12f9463b1ed4ce9364b2d4e433652f3f77a797ec2878b8e1c687fb0f732e5bffa89259ca6a863598240961b",
    "scheme": "Eip712"
  }
}
```

### How To Read The Output

`surface` and `mode` confirm this is the deterministic signing lane.
`order.digest` is the EIP-712 struct hash; `order.orderId` is the 56-byte order
UID (digest, owner, and validity combined); `order.signature` is the
signer-facing string rather than a crate-private binary shape; `order.scheme` is
the explicit signing scheme; and `order.eip1271PayloadPrefix` is the
deterministic EIP-1271 helper prefix. The `cancellation` block proves off-chain
cancellation signs over the same typed order-UID surface.

### Why This Scenario Comes First

If signing is not clear, later transport output is harder to trust.

This scenario gives you one stable checkpoint before you introduce:

- quote construction
- orderbook posting
- runtime adapters
- browser-runtime concerns

It is the fastest way to answer:

"Can this checkout already produce the typed signing artifacts I expect?"

## Step 3: Run The Deterministic Limit-Order Submission Simulation

Run the maintained simulated submission scenario:

```text
cargo run -p cow-sdk-examples-native --example limit_order
```

This scenario uses:

- a mock signer
- a mock orderbook client
- a committed Sepolia-shaped trading flow

It proves order construction and order submission shape without pretending that
mock transport is the same thing as a live orderbook session.

### Current Example Output

A successful run from the committed example currently prints:

```json
{
  "surface": "cow_sdk::trading::Trading::limit",
  "mode": "simulated-transport",
  "result": {
    "orderId": "0x1111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111",
    "signatureLength": 132,
    "signingScheme": "eip712"
  },
  "presign": {
    "orderId": "0x1111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111",
    "signingScheme": "presign",
    "signatureLength": 42
  },
  "submission": {
    "quoteId": null,
    "sellAmount": "98646335338956442",
    "buyAmount": "30000000000000000000",
    "uploadedAppDataCount": 2
  }
}
```

### How To Read The Output

`result` carries the posted order id, the full signature length, and the
lowercase wire-form `eip712` scheme. `presign` is the same builder's signer-free
`post_presign` terminal — posted under the `presign` scheme, fillable only once
the owner flips the on-chain pre-signature flag (the smart-account placement
path). `submission.quoteId` is `null` because a limit order takes the explicit
price you set rather than a fetched quote; the sell and buy amounts and
`uploadedAppDataCount` are the values that actually crossed the simulated
submission seam.

### What This Scenario Adds Beyond Signing

The signing scenario proves the typed signing contract in isolation.

This second scenario proves the broader trading shape:

- `Trading` can carry ready-state defaults into a trade flow
- submission data stays typed across the seam
- app-data handling is part of the same high-level path
- the SDK returns stable user-facing submission output

This is the shortest deterministic answer to:

"Can this checkout already build and simulate a signed limit-order post?"

## Step 4: Understand The Deterministic Boundary

At this point you have verified two different proof classes:

1. a pure signing proof
2. a simulated transport proof

That boundary is intentional.

The examples above still do **not** claim:

- live orderbook reachability
- host-wallet integration through the typed-data signer callback
- deployed WASM page correctness
- runtime-specific provider integration

Those surfaces remain documented and maintained, but they are separate from the
deterministic onboarding path.

## Handling Errors

Every fallible call returns a typed error rather than a string. The `cow-sdk`
facade aggregates the per-crate errors into `CowError`, and every error type —
facade or leaf — exposes a coarse `ErrorClass` (`Validation`, `Transport`,
`Remote`, `RateLimited`, `Signing`, `Cancelled`, `Internal`) for telemetry and
retry decisions.

Orderbook failures additionally carry a status-precise retry verdict:
`is_retryable()` returns the same decision the SDK's own transport retry loop
reaches, and `backoff_hint()` surfaces the server's `Retry-After` cooldown when
present. A consumer can drive a bounded retry loop without re-deriving the
retryable-status set:

```rust,ignore
use std::time::Duration;

let mut attempts = 0;
let posted = loop {
    match trading.post_swap_order(params.clone(), signer, None).await {
        Ok(posted) => break posted,
        Err(error) if attempts < 3 && error.is_retryable() => {
            // Wait the server-suggested `Retry-After`, else your own backoff.
            let wait = error.backoff_hint().unwrap_or(Duration::from_millis(500));
            tokio::time::sleep(wait).await;
            attempts += 1;
        }
        // `error.class()` is the coarse telemetry bucket; the typed variant — for
        // example `OrderbookRejection::category()` — names the consumer action.
        Err(error) => return Err(error.into()),
    }
};
```

Run the maintained walkthrough for a full tour of every class and the
action-oriented rejection categories:

```text
cargo run -p cow-sdk-examples-native --example error_classification
```

`CowError` is the convenience aggregate for `?`-propagating consumers; a consumer
with its own error type matches the leaf directly, since each leaf carries the
same `class()` / `is_retryable()` plus the finer `OrderbookRejection::category()`.
On-chain submission has its own verdict: the receipt-wait helpers return
`WaitError` — generic over the caller's signer and provider error types, so it
stays out of `CowError` — and `WaitError::reverted()` distinguishes a real
on-chain revert from a transient broadcast, lookup, timeout, or cancellation.

## Step 5: Branch By Goal

After the two deterministic checkpoints above, branch into the maintained
example families by user goal.

### Native Follow-Ons

Use these examples when you want local or transport-mocked Rust flows:

`facade_surface`

- reports facade construction and the resolved on-chain deployment for a quick
  crate-orientation pass

`app_data`

- shows how app-data generation and validation surfaces behave

`quote`

- builds a quote flow without posting

`trading_full_cycle`

- exercises a broader quote, allowance, approval, and submission shape

`order_lifecycle`

- shows order lookup and off-chain cancellation

`ethflow`

- builds native-sell / EthFlow transaction data

`onchain_actions`

- builds pre-sign and on-chain cancellation transactions

`orderbook_transport`

- focuses on typed orderbook transport behavior rather than high-level trading

`alloy_quickstart`

- shows the composed native Alloy client setup

`alloy_provider`

- shows read-only Alloy RPC through `Provider`

`alloy_signer`

- signs a real CoW order typed-data payload through the Alloy signer leaf

`transaction_lifecycle`

- shows that native Alloy transaction submission returns a broadcast hash
  without receipt polling

`alloy_trading_full_flow`

- exercises allowance, approval, and pre-sign helper paths through the composed
  Alloy client

The `alloy_*` scenarios and `transaction_lifecycle` need a native Alloy
feature: run them with `--features alloy` (or the narrower
`--features alloy-provider` / `--features alloy-signer` for the provider-only or
signer-only scenarios).

### Read-Only Follow-Ons

When your goal is read-only analytics instead of trading orchestration, switch
to the explicit subgraph crate path:

- `subgraph_query`

These scenarios use `cow-sdk-subgraph` directly; the same surface is also
available through the `cow-sdk` `subgraph` feature.

### WASM Follow-Ons

When your goal is calling the Rust SDK from JavaScript or TypeScript, use the
`cow-sdk-js` package examples in the dedicated
[`0xSymbiome/cow-sdk-examples`](https://github.com/0xSymbiome/cow-sdk-examples)
repository:

- [`cow-signer-node`](https://github.com/0xSymbiome/cow-sdk-examples/tree/main/examples/wasm/cow-signer-node)
- [`cow-gateway-cloudflare`](https://github.com/0xSymbiome/cow-sdk-examples/tree/main/examples/wasm/cow-gateway-cloudflare)

For browser-wallet flows, integrate `cow-sdk-js` with your app's own wallet
stack (viem, wagmi, or any EIP-1193 provider): the host wraps its EIP-1193
provider into the SDK's typed-data signer callback to supply the wallet connection.

### Environment-Sensitive Follow-Ons

These are opt-in and no longer deterministic:

- `orderbook_live`
- `subgraph_live`

Use them only when you specifically need live service confirmation.

## A Good First Session

If you want one recommended first session from a fresh checkout, use:

```text
cargo check -p cow-sdk-examples-native --examples
cargo run -p cow-sdk-examples-native --example swap_quickstart
cargo run -p cow-sdk-examples-native --example sign_order
cargo run -p cow-sdk-examples-native --example limit_order
cargo run -p cow-sdk-examples-native --example trading_full_cycle
```

That sequence — the native catalog's recommended first sequence — proves:

- the facade compiles in the current checkout
- the recommended `Trading::swap()` path quotes, signs, and posts end to end
- the signing lane is stable
- the high-level post, allowance, and approval paths are stable under
  deterministic simulation

## Common Questions

**Do I need a provider adapter, a browser wallet, or live credentials to finish
this guide?** No. The deterministic first-touch path is provider-agnostic and
runs without a wallet or live-service credentials; they matter only when you
move into the environment-sensitive follow-ons. Wire native Alloy through
[Adapting Alloy](providers/adapting-alloy.md) and custom runtimes through
[Integrations](integrations.md) when you are ready.

**Why does the guide start with examples instead of a full application?** The
maintained examples are the canonical public proof surfaces for the current
repository state: versioned with the SDK, and already demonstrating the
supported public contracts.

## Troubleshooting

If `cargo run -p cow-sdk-examples-native --example sign_order`
fails:

- verify that the workspace builds on your local Rust toolchain
- rerun `cargo fmt --all --check` if you are working from a modified checkout
- confirm that you are running from the repository root

If `cargo run -p cow-sdk-examples-native --example limit_order`
fails:

- confirm the native example package still resolves from the checkout
- confirm the mock example was not run from a partially built target tree left
  over by a different workspace layout

If you need custom signer or provider wiring rather than the maintained example
scenarios:

- switch to [Integrations](integrations.md)

If you need crate-boundary guidance rather than runnable onboarding:

- switch to [Architecture](architecture.md)

If you need full publication and validation posture:

- switch to [Release Checklist](release-checklist.md)

## Next Reads

- [Documentation Index](README.md)
- [Integrations](integrations.md)
- [Examples](examples.md)
- [Architecture](architecture.md)
- [Verification Guide](verification.md)
