# cow-sdk-trading

High-level [CoW Protocol](https://cow.fi) trading orchestration surface
covering quoting, signing, posting, allowance management, and on-chain
order actions.

> ⚠️ **Alpha — `0.1.0-alpha`.** Pre-release and not security-audited; the public
> API may change before `0.1.0`. It is published as a pre-release, so Cargo
> selects it only when you opt in (`cow-sdk-trading = "0.1.0-alpha.1"`). Review
> it yourself before relying on it with real funds.

This is the orchestration layer that turns configured signers,
providers, and orderbook clients into a single ready-state trading
facade. The primary entry point is `Trading`. Most end-user code
reaches this crate through [`cow-sdk`](https://crates.io/crates/cow-sdk);
depend on it directly when you want the trading entry points without
the rest of the facade surface.

## Install

```toml
[dependencies]
cow-sdk-trading = "0.1.0-alpha.1"
```

## Minimal example

The `TradingBuilder::ready` one-call shortcut accepts a complete
total `TraderParams` and returns a ready-state `Trading` with the default
per-chain orderbook client:

```rust
use cow_sdk_core::SupportedChainId;
use cow_sdk_trading::{TraderParams, TradingBuilder};

let _trading = TradingBuilder::ready(
    TraderParams::new(SupportedChainId::Sepolia, "your-app-code")
        .expect("app code validates"),
);
```

For fluent control over env, settlement-contract overrides, or orderbook
injection, use the full builder:

```rust
use cow_sdk_core::{CowEnv, SupportedChainId};
use cow_sdk_trading::Trading;

let _trading = Trading::builder()
    .chain_id(SupportedChainId::Sepolia)
    .app_code("your-app-code")
    .env(CowEnv::Prod)
    .build()
    .expect("ready-state construction");
```

Allowance reads, approval submission, pre-sign transaction construction, and
on-chain cancellation need chain authority but no app code, so they are the
crate's free functions — `cow_protocol_allowance`, `approval_transaction`,
`pre_sign_transaction`, and `onchain_cancel_order` — and need no trading
client. Quote, post, order lookup, and off-chain cancellation flows use the
ready `Trading` client.

Owner attribution lives on the per-trade `TradeParams` (or
`LimitTradeParams`); the `Trading` client does not store a default owner. For
signer-backed flows the signer's address fills the slot when
`TradeParams.owner` is `None`.

## Swap in one call

`Trading::swap()` opens a typed builder with named token setters, so the sell
and buy tokens cannot be transposed. `execute` quotes, signs, and posts in one
call; `quote` returns a result you can inspect before `submit`. The same chain
works with any signer — a local key, a remote signer, a host-supplied EIP-1193
wallet, or a smart account:

```rust,no_run
use cow_sdk_core::{Amount, Signer, UserRejection, address};
use cow_sdk_trading::Trading;

# async fn run<S>(trading: Trading, signer: &S) -> Result<(), Box<dyn std::error::Error>>
# where S: Signer, S::Error: std::fmt::Display + UserRejection {
let weth = address!("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
let usdc = address!("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");

// Quote, sign, and post in one call.
let posted = trading
    .swap()
    .sell_token(weth)
    .buy_token(usdc)
    .sell_amount(Amount::from_units(1, 18)?)
    .slippage_bps(50)
    .execute(signer)
    .await?;
println!("posted order {}", posted.order_id.to_hex_string());

// Or inspect the quote first, then submit the exact quoted order.
let quoted = trading
    .swap()
    .sell_token(weth)
    .buy_token(usdc)
    .sell_amount(Amount::from_units(1, 18)?)
    .quote(signer)
    .await?;
println!("suggested slippage (bps): {}", quoted.results().suggested_slippage_bps);
let _posted = quoted.submit(signer).await?;
# Ok(())
# }
```

## Limit order in one call

A limit order sets an explicit price — both amounts — so no quote is fetched.
`Trading::limit()` opens the same kind of typed builder as `swap()`: named setters that
cannot be transposed, then `post` to sign and post, or `post_presign` for the
smart-account path that needs no signer:

```rust,no_run
use cow_sdk_core::{Amount, Signer, UserRejection, address};
use cow_sdk_trading::Trading;

# async fn run<S>(trading: Trading, signer: &S) -> Result<(), Box<dyn std::error::Error>>
# where S: Signer, S::Error: std::fmt::Display + UserRejection {
let weth = address!("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
let usdc = address!("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");

// Sell exactly 1 WETH, want at least 3000 USDC.
let posted = trading
    .limit()
    .sell_token(weth)
    .buy_token(usdc)
    .sell_amount(Amount::from_units(1, 18)?)
    .buy_amount(Amount::from_units(3000, 6)?)
    .post(signer)
    .await?;
println!("posted order {}", posted.order_id.to_hex_string());
# Ok(())
# }
```

## Quoting a swap

Quoting is the lowest-friction action and needs no signer — the owner comes
from `TradeParams`:

```rust,no_run
use cow_sdk_core::{Address, Amount, OrderKind, SupportedChainId, address};
use cow_sdk_trading::{TradeParams, TradingBuilder};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let trading = TradingBuilder::ready(
    cow_sdk_trading::TraderParams::new(SupportedChainId::Mainnet, "your-app-code")?,
);

// Sell 1 WETH for USDC.
let weth = address!("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
let usdc = address!("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
let params = TradeParams::new(
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
client, separate Alloy provider and signer adapters, host-supplied EIP-1193
wallets, and custom integrations.

When only the revert verdict matters, `WaitError::reverted()` returns the
reverted receipt for a mined on-chain failure and `None` for the transient
broadcast, lookup, timeout, and cancellation variants — a coarse alternative to
matching each variant.

## Feature flags

| Feature | Default | Enables |
| --- | --- | --- |
| `tracing` | off | `tracing` spans on every `Trading` method and the broadcast/receipt path, and enables tracing across the core, contracts, signing, orderbook, and app-data crates. |

## Where this fits

`Trading` orchestrates; it carries no transport or signing crypto of its own.
Orderbook I/O is delegated to an injected or default
[`cow-sdk-orderbook`](https://crates.io/crates/cow-sdk-orderbook) client (which
owns retry and rate-limit policy), and signing goes through a caller-supplied
`cow_sdk_core::Signer`. `OrderBoundsValidator` enforces only operator-independent
invariants client-side; the services backend remains authoritative for
deny-lists, balances, exact validity windows, and price checks. No `alloy_*` type
appears in the public API. Most consumers reach this crate through the
[`cow-sdk`](https://crates.io/crates/cow-sdk) facade as `cow_sdk::trading`.

## Examples

The workspace ships runnable, deterministic scenarios for the trading
workflows — quoting, posting, EthFlow, receipt waiting, and the advanced
seam traits — cataloged by goal in
[Examples](https://github.com/0xSymbiome/cow-rs/blob/main/docs/examples.md).
[Getting Started](https://github.com/0xSymbiome/cow-rs/blob/main/docs/getting-started.md)
walks the recommended first session.

## Where to next

- [Getting Started](https://github.com/0xSymbiome/cow-rs/blob/main/docs/getting-started.md)
- [Integrations Guide](https://github.com/0xSymbiome/cow-rs/blob/main/docs/integrations.md)
- [Workspace README](https://github.com/0xSymbiome/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-or-later. See the workspace
[LICENSE](https://github.com/0xSymbiome/cow-rs/blob/main/LICENSE)
file for the full text.
