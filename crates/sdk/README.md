# cow-sdk

Primary Rust SDK facade for [CoW Protocol](https://cow.fi).

`cow-sdk` is the curated first-touch entry point of the `cow-rs` crate
family. It re-exports the core types, signing helpers, contract helpers,
orderbook client, app-data helpers, and the high-level trading
orchestration surface from one place. Browser-wallet support is optional
and feature-gated behind `browser-wallet`.

The cow-named identity and numeric primitive types (`Address`, `Hash32`,
`AppDataHash`, `HexData`, `OrderUid`, `Amount`)
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
use cow_sdk::core::SupportedChainId;
use cow_sdk::trading::Trading;

let _trading = Trading::builder()
    .chain_id(SupportedChainId::Sepolia)
    .app_code("your-app-code")
    .build()
    .unwrap();
```

Once constructed, a single call quotes, signs, and posts a swap. The order
owner defaults to the signer's address:

```rust,no_run
# use std::error::Error;
use cow_sdk::core::{Address, Amount, OrderKind, SupportedChainId};
use cow_sdk::trading::{TradeParameters, Trading};
#
# async fn run<S>(signer: &S) -> Result<(), Box<dyn Error>>
# where
#     S: cow_sdk::core::Signer,
#     S::Error: std::fmt::Display + cow_sdk::core::SignerError,
# {
let trading = Trading::builder()
    .chain_id(SupportedChainId::Sepolia)
    .app_code("your-app-code")
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
let posted = trading.post_swap_order(params, signer, None).await?;
println!("posted order: {}", posted.order_id.to_hex_string());
# Ok(())
# }
```

For allowance, approval, pre-sign, or on-chain cancellation that does not need
an app code, call the crate's free functions directly —
`cow_protocol_allowance`, `approval_transaction`, `pre_sign_transaction`,
and `cancel_order_onchain` — without constructing a trading client.

## Handling errors

Every fallible call returns a typed error. The facade aggregates the per-crate
errors into `CowError`, and every error type — facade or leaf — exposes a coarse
`ErrorClass` (`Validation`, `Transport`, `Remote`, `RateLimited`, `Signing`,
`Cancelled`, `Internal`) for telemetry. Orderbook failures add a status-precise
retry verdict: `is_retryable()` returns the same decision the SDK's own transport
retry loop reaches, and `backoff_hint()` surfaces the server's `Retry-After`
cooldown when present.

```rust
use std::time::Duration;
use cow_sdk::{CowError, ErrorClass};

/// Decide whether a failed SDK call should be retried, and how long to wait.
fn retry_delay(error: &CowError) -> Option<Duration> {
    // `class()` is the coarse telemetry bucket; `is_retryable()` is the
    // status-precise retry decision — a retryable `503` and a non-retryable
    // `400` are both `ErrorClass::Remote`, so class alone cannot tell them apart.
    let _telemetry_bucket: ErrorClass = error.class();
    error
        .is_retryable()
        .then(|| error.backoff_hint().unwrap_or(Duration::from_millis(500)))
}
```

`CowError` is the convenience aggregate for consumers that `?`-propagate every
SDK call into one type. A consumer with its own error type — or that needs
rejection-specific handling — matches the **leaf** error directly instead: each
leaf carries the same `class()` and `is_retryable()`, plus the finer-grained
`OrderbookRejection::category()` that names the action a rejection calls for. The
native `error_classification` example walks every `ErrorClass` bucket and the
`category()` refinement end to end.

On-chain submission has its own verdict. The receipt-wait helpers return
`WaitError`, which is generic over the caller's signer and provider error types,
so it stays out of `CowError`; use `WaitError::reverted()` to tell a real
on-chain revert from a transient broadcast, lookup, timeout, or cancellation.

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)
- [Architecture](https://github.com/cowdao-grants/cow-rs/blob/main/docs/architecture.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
