# cow-sdk-subgraph

Typed [CoW Protocol](https://cow.fi) subgraph query primitives with
saved query documents, explicit raw-GraphQL request contracts, and a
typed error boundary.

This is a read-only analytics crate. It is kept deliberately separate
from the default [`cow-sdk`](https://crates.io/crates/cow-sdk) facade so
trading-first consumers do not pay a GraphQL transport dependency they
do not use. Depend on this crate directly when building analytics,
reporting, or dashboards over CoW Protocol subgraph data.

## Install

```toml
[dependencies]
cow-sdk-subgraph = "0.1"
```

## Minimal example

```rust
use cow_sdk_core::SupportedChainId;
use cow_sdk_subgraph::SubgraphApi;

let _subgraph = SubgraphApi::builder()
    .chain(SupportedChainId::Mainnet)
    .api_key("your-subgraph-api-key")
    .build()
    .expect("subgraph client builds with canonical defaults");
```

## Where to next

- [Architecture](https://github.com/cowdao-grants/cow-rs/blob/main/docs/architecture.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
