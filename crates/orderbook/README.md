# cow-sdk-orderbook

Typed [CoW Protocol](https://cow.fi) orderbook client with chain and
environment-aware endpoint resolution, explicit request policy, and
deterministic response decoding.

This crate owns the canonical request builders, typed wire DTOs,
response transforms, and retry policy for the CoW Protocol orderbook
REST API. It is used internally by the
[`cow-sdk-trading`](https://crates.io/crates/cow-sdk-trading)
orchestration surface and is exposed directly when you only need the
typed transport layer without the higher-level trading flow. Transport
configuration is policy-visible: HTTP timeout, retry rules, and
user-agent defaults are explicit.

## Install

```toml
[dependencies]
cow-sdk-orderbook = "0.1"
```

## Minimal example

Build the client with the typestate builder, then request a sell-side quote.
On native targets `build()` uses the default `reqwest` transport; on `wasm32`
inject a browser transport with `.transport(...)` before `.build()`.

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

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
