# Native Examples

These examples demonstrate the public Rust SDK surface without requiring
browser runtimes or live order placement.

## Scenarios

| Scenario | Purpose |
| --- | --- |
| `sdk_surface_report` | Report the root facade surface |
| `app_data_roundtrip` | Generate and inspect app-data output |
| `signing_roundtrip` | Sign orders and cancellations and inspect typed payloads |
| `quote_only_simulation` | Build a quote flow without submission |
| `limit_order_simulation` | Build and simulate signed limit-order submission |
| `order_lifecycle_simulation` | Inspect order lookup and off-chain cancellation |
| `trading_sdk_simulation` | Inspect high-level `TradingSdk` quote, allowance, and approval flow |
| `ethflow_transaction_simulation` | Build native-sell / EthFlow transaction data |
| `onchain_order_actions_simulation` | Build pre-sign and on-chain cancellation transactions |
| `orderbook_transport_roundtrip` | Inspect typed orderbook transport behavior |
| `orderbook_live_probe` | Run an opt-in live orderbook version probe |
| `subgraph_query_roundtrip` | Inspect canonical subgraph helper usage |
| `subgraph_custom_query_roundtrip` | Inspect explicit `SubgraphQueryRequest` usage |
| `subgraph_live_query` | Run an opt-in live subgraph query |

Subgraph scenarios use `cow-sdk-subgraph` directly rather than the root
facade.

## Validation

```text
cargo check --manifest-path examples/native/Cargo.toml --examples
cargo test --manifest-path examples/native/Cargo.toml
```

## Running Examples

```text
cargo run --manifest-path examples/native/Cargo.toml --example sdk_surface_report
cargo run --manifest-path examples/native/Cargo.toml --example app_data_roundtrip
cargo run --manifest-path examples/native/Cargo.toml --example signing_roundtrip
cargo run --manifest-path examples/native/Cargo.toml --example quote_only_simulation
cargo run --manifest-path examples/native/Cargo.toml --example limit_order_simulation
cargo run --manifest-path examples/native/Cargo.toml --example order_lifecycle_simulation
cargo run --manifest-path examples/native/Cargo.toml --example trading_sdk_simulation
cargo run --manifest-path examples/native/Cargo.toml --example ethflow_transaction_simulation
cargo run --manifest-path examples/native/Cargo.toml --example onchain_order_actions_simulation
cargo run --manifest-path examples/native/Cargo.toml --example orderbook_transport_roundtrip
cargo run --manifest-path examples/native/Cargo.toml --example orderbook_live_probe
cargo run --manifest-path examples/native/Cargo.toml --example subgraph_query_roundtrip
cargo run --manifest-path examples/native/Cargo.toml --example subgraph_custom_query_roundtrip
cargo run --manifest-path examples/native/Cargo.toml --example subgraph_live_query
```

## Optional Environment Variables

Before running `orderbook_live_probe`, you can set:

- `COW_SMOKE_ORDERBOOK_ENV`
- `COW_SMOKE_ORDERBOOK_CHAIN_ID`
- `COW_SMOKE_ORDERBOOK_BASE_URL`
- `COW_SMOKE_ORDERBOOK_API_KEY`

Before running `subgraph_live_query`, set:

- `THE_GRAPH_API_KEY`
- optionally `COW_SUBGRAPH_CHAIN_ID`
