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

```rust
use cow_sdk_trading::{SupportedChainId, TradingSdk};

let _sdk = TradingSdk::builder()
    .with_chain_id(SupportedChainId::Sepolia)
    .with_app_code("your-app-code")
    .build()
    .unwrap();
```

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [Integrations Guide](https://github.com/cowdao-grants/cow-rs/blob/main/docs/integrations.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
