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

```rust
use cow_sdk_orderbook::{CowEnv, OrderbookApi, SupportedChainId};

let _api = OrderbookApi::builder()
    .chain(SupportedChainId::Sepolia)
    .environment(CowEnv::Prod)
    .build()
    .expect("orderbook client builds with canonical defaults");
```

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
