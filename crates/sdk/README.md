# cow-sdk

Primary Rust SDK facade for [CoW Protocol](https://cow.fi).

> ⚠️ **Alpha — `0.1.0-alpha`.** Pre-release and not security-audited; the public
> API may change before `0.1.0`. It is published as a pre-release, so Cargo
> selects it only when you opt in (`cow-sdk = "0.1.0-alpha.4"`). Review it
> yourself before relying on it with real funds.

`cow-sdk` is the curated first-touch entry point of the `cow-rs` crate
family. It re-exports the core types, signing helpers, contract helpers,
orderbook client, app-data helpers, and the high-level trading
orchestration surface from one place.

The cow-named identity and numeric primitive types (`Address`, `Hash32`,
`AppDataHash`, `HexData`, `OrderUid`, `Amount`)
re-export through the facade as cow-owned
`#[repr(transparent)]` newtypes over `alloy_primitives` per
[ADR 0052](https://github.com/0xSymbiome/cow-rs/blob/main/docs/adr/0052-alloy-primitives-canonical-primitive-layer.md).

## Install

```toml
[dependencies]
cow-sdk = "0.1.0-alpha.4"
```

## Feature flags

Every optional surface is off by default (`default = []`); enable only what you
use. The HTTP retry, rate-limit, and `Retry-After` transport policy is always on.

| Feature | Enables |
| --- | --- |
| `subgraph` | Read-only subgraph analytics as `cow_sdk::subgraph`; lifts `SubgraphError` into `CowError`. |
| `cow-shed` | COW Shed account-abstraction hooks as `cow_sdk::cow_shed` (proxy derivation, EIP-712 hook signing, factory calldata). |
| `alloy-provider` | Native Alloy read-only `Provider` adapter as `cow_sdk::alloy_provider`. |
| `alloy-signer` | Native Alloy local-key `Signer` adapter as `cow_sdk::alloy_signer`. |
| `alloy` | The composed native Alloy client as `cow_sdk::alloy`; implies `alloy-provider` and `alloy-signer`. |
| `in-memory-cache` | The `InMemoryEip1271Cache` implementation. The cache trait and `NoopEip1271Cache` are always available; this adds the in-memory store. |
| `testing` | In-memory test doubles (`OrderbookClient`, `Signer`, `Provider`) as `cow_sdk::testing` for downstream integration tests. Dev-dependency only. |
| `tracing` | `tracing` spans and structured events across the SDK; see the [Observability](https://github.com/0xSymbiome/cow-rs/blob/main/docs/observability.md) guide. |

```toml
[dependencies]
cow-sdk = { version = "0.1.0-alpha.4", features = ["subgraph", "cow-shed"] }
```

## Native default example

The shortest ready-state path uses the native default orderbook transport.
Browser targets use the same trading API but must inject a browser transport;
see the workspace
[Getting Started](https://github.com/0xSymbiome/cow-rs/blob/main/docs/getting-started.md)
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
use cow_sdk::core::{address, Address, Amount, SupportedChainId};
use cow_sdk::trading::Trading;

// Tokens are compile-time validated `Address` literals, not raw strings. The
// literal is the lowercase wire form; a mixed-case literal rejects at build time.
const WETH: Address = address!("0xfff9976782d46cc05630d1f6ebab18b2324d6b14");
const COW: Address = address!("0x0625afb445c3b6b7b929342a04a22599fd5dbb59");
#
# async fn run<S>(signer: &S) -> Result<(), Box<dyn Error>>
# where
#     S: cow_sdk::core::Signer,
#     S::Error: std::fmt::Display + cow_sdk::core::UserRejection,
# {
let trading = Trading::builder()
    .chain_id(SupportedChainId::Sepolia)
    .app_code("your-app-code")
    .build()?;

// The named setters keep the sell and buy legs from being transposed, and
// `execute` becomes callable only once both tokens and an amount are set. The
// owner defaults to the signer address and slippage uses the quote-aware
// tolerance unless either is set.
let posted = trading
    .swap()
    .sell_token(WETH)
    .buy_token(COW)
    .sell_amount(Amount::parse_units("0.1", 18)?)
    .execute(signer)
    .await?;

println!("https://explorer.cow.fi/sepolia/orders/{}", posted.order_id);
# Ok(())
# }
```

For allowance, approval, pre-sign, or on-chain cancellation that does not need
an app code, call the crate's free functions directly —
`cow_protocol_allowance`, `approval_transaction`, `pre_sign_transaction`,
and `onchain_cancel_order` — without constructing a trading client.

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

## Examples

The workspace ships runnable, deterministic scenarios for every facade
workflow — quoting, posting, signing, app-data, transport, subgraph access,
and the Alloy adapters — cataloged by goal in
[Examples](https://github.com/0xSymbiome/cow-rs/blob/main/docs/examples.md).
[Getting Started](https://github.com/0xSymbiome/cow-rs/blob/main/docs/getting-started.md)
walks the recommended first session.

## Where to next

- [Getting Started](https://github.com/0xSymbiome/cow-rs/blob/main/docs/getting-started.md)
- [Workspace README](https://github.com/0xSymbiome/cow-rs/blob/main/README.md)
- [Architecture](https://github.com/0xSymbiome/cow-rs/blob/main/docs/architecture.md)

## License

Licensed under GPL-3.0-or-later. See the workspace
[LICENSE](https://github.com/0xSymbiome/cow-rs/blob/main/LICENSE)
file for the full text.
