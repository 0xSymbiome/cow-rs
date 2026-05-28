# cow-sdk-trading

High-level [CoW Protocol](https://cow.fi) trading orchestration surface
covering quoting, signing, posting, allowance management, and on-chain
order actions.

This is the orchestration layer that turns configured signers,
providers, and orderbook clients into a single ready-state trading
facade. The primary entry point is `TradingSdk`. Most end-user code
reaches this crate through [`cow-sdk`](https://crates.io/crates/cow-sdk);
depend on it directly when you want the trading entry points without
the browser-wallet optional dependency.

## Install

```toml
[dependencies]
cow-sdk-trading = "0.1"
```

## Minimal example

The `TradingSdkBuilder::ready` one-call shortcut accepts a complete
`TraderParameters` plus an options bundle and returns a ready-state
`TradingSdk`:

```rust
use cow_sdk_core::SupportedChainId;
use cow_sdk_trading::{TraderParameters, TradingSdkBuilder, TradingSdkOptions};

let _sdk = TradingSdkBuilder::ready(
    TraderParameters::new(SupportedChainId::Sepolia, "your-app-code")
        .expect("app code validates"),
    TradingSdkOptions::default(),
)
.expect("ready-state construction");
```

For fluent control over env, settlement-contract overrides, or transport
injection, use the full builder:

```rust
use cow_sdk_core::{CowEnv, SupportedChainId};
use cow_sdk_trading::{TradingSdk, TradingSdkOptions};

let _sdk = TradingSdk::builder()
    .with_chain_id(SupportedChainId::Sepolia)
    .with_app_code("your-app-code")
    .with_env(CowEnv::Prod)
    .with_options(TradingSdkOptions::new())
    .build_ready()
    .expect("ready-state construction");
```

Use `TradingSdkBuilder::helper_only` (or `build_helper_only()` on the
full builder) for chain-bound helper workflows that do not need quote,
post, order lookup, or off-chain cancellation submission through the
SDK:

```rust
use cow_sdk_core::SupportedChainId;
use cow_sdk_trading::{TradingSdkBuilder, TradingSdkOptions};

let _sdk = TradingSdkBuilder::helper_only(
    SupportedChainId::Sepolia,
    TradingSdkOptions::default(),
)
.expect("helper-only construction");
```

Helper-only SDKs support allowance reads, approval submission, pre-sign
transaction construction, and on-chain cancellation. Quote, post, order lookup,
and off-chain cancellation methods are available only on `TradingSdk`.

Owner attribution lives on the per-trade `TradeParameters` (or
`LimitTradeParameters`); the SDK does not store a default owner. For
signer-backed flows the signer's address fills the slot when
`TradeParameters.owner` is `None`.

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

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [Integrations Guide](https://github.com/cowdao-grants/cow-rs/blob/main/docs/integrations.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
