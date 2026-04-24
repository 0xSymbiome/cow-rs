# cow-sdk

Primary Rust SDK facade for [CoW Protocol](https://cow.fi).

`cow-sdk` is the curated first-touch entry point of the `cow-rs` crate
family. It re-exports the core types, signing helpers, contract helpers,
orderbook client, app-data helpers, and the high-level trading
orchestration surface from one place. Browser-wallet support is optional
and feature-gated behind `browser-wallet`.

## Install

```toml
[dependencies]
cow-sdk = "0.1"
```

## Minimal example

```rust
use cow_sdk::prelude::{SupportedChainId, TradingSdk};

let _sdk = TradingSdk::builder()
    .with_chain_id(SupportedChainId::Sepolia)
    .with_app_code("your-app-code")
    .build()
    .unwrap();
```

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)
- [Architecture](https://github.com/cowdao-grants/cow-rs/blob/main/docs/architecture.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
