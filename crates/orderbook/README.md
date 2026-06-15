# cow-sdk-orderbook

Typed [CoW Protocol](https://cow.fi) orderbook client with chain and
environment-aware endpoint resolution, explicit request policy, and
deterministic response decoding.

> ⚠️ **Alpha — `0.1.0-alpha`.** Pre-release and not security-audited; the public
> API may change before `0.1.0`. It is published as a pre-release, so Cargo
> selects it only when you opt in (`cow-sdk-orderbook = "0.1.0-alpha.1"`). Review
> it yourself before relying on it with real funds.

This crate owns the canonical request builders, typed wire DTOs, response
transforms, and retry policy for the CoW Protocol orderbook REST API. It is used
internally by the [`cow-sdk-trading`](https://crates.io/crates/cow-sdk-trading)
orchestration surface and is exposed directly when you only need the typed
transport layer without the higher-level trading flow. Because it transports
already-signed orders, it depends on no signing crate: you get the typed quote,
post, and query surface without compiling the ECDSA signing stack. Transport
configuration is policy-visible: HTTP timeout, retry rules, and user-agent
defaults are explicit.

## What it provides

- **Quoting with fail-closed echo verification** — `quote()` validates the
  request, POSTs `/api/v1/quote`, and rejects any response that altered a
  request-determined field before it can be signed.
- **Order submission and cancellation** — `send_order()` and
  `send_cancellations()` over typed `OrderCreation` / `OrderCancellations`.
- **Order and trade reads with EthFlow normalization** — `order()`,
  `order_multi_env()` (404-fallback across environments), `orders()`,
  `tx_orders()`, and `trades()`.
- **Status, pricing, and surplus** — `order_competition_status()`,
  `native_price()`, `total_surplus()`, and `version()`.
- **Content-addressed app-data** — `app_data()` and `upload_app_data()`, the
  latter with two-stage keccak256 hash verification (client precheck + server echo).
- **Solver competition** — `solver_competition()` and
  `solver_competition_by_tx_hash()`.
- **A typed rejection taxonomy and retry verdict** — `OrderbookRejection` maps
  every server `errorType` to a typed variant (with a forward-compatible
  `Unknown` fallback) and an `OrderbookRejectionCategory` action partition;
  `OrderbookError::is_retryable()` mirrors the SDK's own transport retry decision
  and `backoff_hint()` surfaces the server's `Retry-After`.
- **Hardened transport** — an instance-scoped rate limiter shared across clones,
  SSRF host validation on base-URL overrides, a response-size cap, and credential
  and PII redaction so error output never leaks upstream bytes (ADR 0025).

## Install

```toml
[dependencies]
cow-sdk-orderbook = "0.1.0-alpha.1"
```

## Minimal example

Build the client with the typestate builder, then request a sell-side quote.
`build()` is zero-config on every target: native uses the default `reqwest`
transport, `wasm32` uses the browser `fetch` transport. Inject your own with
`.transport(...)` on either.

```rust,no_run
use cow_sdk_orderbook::{
    Address, Amount, CowEnv, OrderQuoteRequest, OrderQuoteSide, OrderbookApi,
    SupportedChainId,
};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let orderbook = OrderbookApi::builder()
    .chain(SupportedChainId::Mainnet)
    .env(CowEnv::Prod)
    .build()?;

// Sell-side quote for 1 WETH -> USDC.
let weth = Address::new("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")?;
let usdc = Address::new("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")?;
let from = Address::new("0x76b0340e50BD9883D8B2CA5fd9f52439a9e7Cf58")?;
let request = OrderQuoteRequest::new(
    weth,
    usdc,
    from,
    OrderQuoteSide::sell(Amount::from_units(1, 18)?),
);

let quote = orderbook.quote(&request).await?;
println!("quoted buy amount: {}", quote.quote.buy_amount);
# Ok(())
# }
```

## Feature flags

| Feature | Default | Enables |
| --- | --- | --- |
| `tracing` | off | Wraps every client method in a `tracing` span carrying chain, environment, endpoint, and method fields, and enables `cow-sdk-core`'s tracing. |

## Where this fits

This crate is the typed REST transport layer. It does not sign or hold keys (it
transports already-signed payloads — signing lives in
[`cow-sdk-signing`](https://crates.io/crates/cow-sdk-signing) and the alloy signer
adapters), it does not write canonical app-data JSON (that is
[`cow-sdk-app-data`](https://crates.io/crates/cow-sdk-app-data); `upload_app_data`
hashes the bytes you give it), and it does not orchestrate swaps or send on-chain
transactions (that is [`cow-sdk-trading`](https://crates.io/crates/cow-sdk-trading)).
Most consumers reach it through the [`cow-sdk`](https://crates.io/crates/cow-sdk)
facade as `cow_sdk::orderbook`.

## Where to next

- [Getting Started](https://github.com/0xSymbiome/cow-rs/blob/main/docs/getting-started.md)
- [Workspace README](https://github.com/0xSymbiome/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-or-later. See the workspace
[LICENSE](https://github.com/0xSymbiome/cow-rs/blob/main/LICENSE)
file for the full text.
