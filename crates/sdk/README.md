# cow-sdk

Primary Rust SDK facade for [CoW Protocol](https://cow.fi).

`cow-sdk` is the curated first-touch entry point of the `cow-rs` crate
family. It re-exports the core types, signing helpers, contract helpers,
orderbook client, app-data helpers, and the high-level trading
orchestration surface from one place. Browser-wallet support is optional
and feature-gated behind `browser-wallet`.

The cow-named identity and numeric primitive types (`Address`, `Hash32`,
`AppDataHash`, `HexData`, `OrderUid`, `Amount`, `SignedAmount`)
re-export through the facade as cow-owned
`#[repr(transparent)]` newtypes over `alloy_primitives` per
[ADR 0052](https://github.com/cowdao-grants/cow-rs/blob/main/docs/adr/0052-alloy-primitives-canonical-primitive-layer.md).

## Install

```toml
[dependencies]
cow-sdk = "0.1"
```

## Native default example

The shortest ready-state path uses the native default orderbook transport.
Browser targets use the same trading API but must inject a browser transport;
see the workspace
[Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
guide for that wiring.

```rust
use cow_sdk::prelude::{SupportedChainId, Trading};

let _sdk = Trading::builder()
    .with_chain_id(SupportedChainId::Sepolia)
    .with_app_code("your-app-code")
    .build()
    .unwrap();
```

Once constructed, a single call quotes, signs, and posts a swap. The order
owner defaults to the signer's address:

```rust,no_run
# use std::error::Error;
use cow_sdk::prelude::{Address, SupportedChainId, TradeParameters, Trading};
use cow_sdk::core::{Amount, OrderKind};
#
# async fn run<S>(signer: &S) -> Result<(), Box<dyn Error>>
# where
#     S: cow_sdk::core::Signer,
#     S::Error: std::fmt::Display + cow_sdk::core::SignerError,
# {
let sdk = Trading::builder()
    .with_chain_id(SupportedChainId::Sepolia)
    .with_app_code("your-app-code")
    .build()?;

// Sell 0.1 WETH for COW on Sepolia.
let weth = Address::new("0xfff9976782d46cc05630d1f6ebab18b2324d6b14")?;
let cow = Address::new("0x0625afb445c3b6b7b929342a04a22599fd5dbb59")?;
let params = TradeParameters::new(
    OrderKind::Sell,
    weth,
    cow,
    Amount::parse_units("0.1", 18)?,
);

// One call quotes, signs with `signer`, and posts to the orderbook.
let posted = sdk.post_swap_order(params, signer, None).await?;
println!("posted order: {}", posted.order_id.to_hex_string());
# Ok(())
# }
```

For allowance, approval, pre-sign, or on-chain cancellation that does not need
an app code, call the crate's free functions directly —
`get_cow_protocol_allowance`, `approval_transaction`, `get_pre_sign_transaction`,
and `cancel_order_onchain` — without constructing a trading client.

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)
- [Architecture](https://github.com/cowdao-grants/cow-rs/blob/main/docs/architecture.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
