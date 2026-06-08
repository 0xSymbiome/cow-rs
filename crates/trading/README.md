# cow-sdk-trading

High-level [CoW Protocol](https://cow.fi) trading orchestration surface
covering quoting, signing, posting, allowance management, and on-chain
order actions.

This is the orchestration layer that turns configured signers,
providers, and orderbook clients into a single ready-state trading
facade. The primary entry point is `Trading`. Most end-user code
reaches this crate through [`cow-sdk`](https://crates.io/crates/cow-sdk);
depend on it directly when you want the trading entry points without
the browser-wallet optional dependency.

## Install

```toml
[dependencies]
cow-sdk-trading = "0.1"
```

## Minimal example

The `TradingBuilder::ready` one-call shortcut accepts a complete
`TraderParameters` plus an options bundle and returns a ready-state
`Trading`:

```rust
use cow_sdk_core::SupportedChainId;
use cow_sdk_trading::{TraderParameters, TradingBuilder, TradingOptions};

let _trading = TradingBuilder::ready(
    TraderParameters::new(SupportedChainId::Sepolia, "your-app-code")
        .expect("app code validates"),
    TradingOptions::default(),
)
.expect("ready-state construction");
```

For fluent control over env, settlement-contract overrides, or transport
injection, use the full builder:

```rust
use cow_sdk_core::{CowEnv, SupportedChainId};
use cow_sdk_trading::{Trading, TradingOptions};

let _trading = Trading::builder()
    .chain_id(SupportedChainId::Sepolia)
    .app_code("your-app-code")
    .env(CowEnv::Prod)
    .options(TradingOptions::new())
    .build()
    .expect("ready-state construction");
```

Allowance reads, approval submission, pre-sign transaction construction, and
on-chain cancellation need chain authority but no app code, so they are the
crate's free functions — `cow_protocol_allowance`, `approval_transaction`,
`pre_sign_transaction`, and `cancel_order_onchain` — and need no trading
client. Quote, post, order lookup, and off-chain cancellation flows use the
ready `Trading` client.

Owner attribution lives on the per-trade `TradeParameters` (or
`LimitTradeParameters`); the `Trading` client does not store a default owner. For
signer-backed flows the signer's address fills the slot when
`TradeParameters.owner` is `None`.

## Quoting a swap

Quoting is the lowest-friction action and needs no signer — the owner comes
from `TradeParameters`:

```rust,no_run
use cow_sdk_core::{Address, Amount, OrderKind, SupportedChainId};
use cow_sdk_trading::{TradeParameters, TradingBuilder, TradingOptions};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let trading = TradingBuilder::ready(
    cow_sdk_trading::TraderParameters::new(SupportedChainId::Mainnet, "your-app-code")?,
    TradingOptions::default(),
)?;

// Sell 1 WETH for USDC.
let weth = Address::new("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")?;
let usdc = Address::new("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")?;
let params = TradeParameters::new(
    OrderKind::Sell,
    weth,
    usdc,
    Amount::from_units(1, 18)?,
)
.with_owner(Address::new("0x76b0340e50BD9883D8B2CA5fd9f52439a9e7Cf58")?);

let quote = trading.quote_only(params, None).await?;
println!("suggested slippage (bps): {}", quote.suggested_slippage_bps);
# Ok(())
# }
```

## Waiting for mined receipts

For workflows that need to observe mined success or revert status before
continuing, use `submit_and_wait_for_receipt`. This is the common shape for
approve-then-settle flows:

```rust,no_run
# use std::error::Error;
# use cow_sdk_core::{
#     Provider, Signer, TransactionRequest, TransactionStatus,
# };
use cow_sdk_trading::{WaitError, WaitOptions, submit_and_wait_for_receipt};
#
# async fn approve_flow<S, P>(
#     signer: &S,
#     provider: &P,
#     approve_tx: &TransactionRequest,
# ) -> Result<(), Box<dyn Error>>
# where
#     S: Signer,
#     S::Error: Error + 'static,
#     P: Provider,
#     P::Error: Error + 'static,
# {

let receipt = match submit_and_wait_for_receipt(
    signer,
    provider,
    approve_tx,
    WaitOptions::approve_default(),
)
.await
{
    Ok(receipt) => receipt,
    Err(WaitError::Reverted { receipt }) => {
        return Err(format!(
            "approval reverted: gas_used={:?}",
            receipt.gas_used
        )
        .into());
    }
    Err(WaitError::Timeout {
        transaction_hash,
        elapsed,
    }) => {
        return Err(format!(
            "approval receipt {} was not observed after {:?}",
            transaction_hash.to_hex_string(),
            elapsed
        )
        .into());
    }
    Err(other) => return Err(Box::new(other)),
};

assert_eq!(receipt.status, Some(TransactionStatus::Success));
# Ok(())
# }
```

The companion `poll_for_receipt` helper is available when a workflow already
has a transaction hash from a separate broadcast path. Both helpers are generic
over `Signer` and `Provider`, so they work with the native Alloy
client, separate Alloy provider and signer adapters, browser-wallet adapters,
and custom integrations.

When only the revert verdict matters, `WaitError::reverted()` returns the
reverted receipt for a mined on-chain failure and `None` for the transient
broadcast, lookup, timeout, and cancellation variants — a coarse alternative to
matching each variant.

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [Integrations Guide](https://github.com/cowdao-grants/cow-rs/blob/main/docs/integrations.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
