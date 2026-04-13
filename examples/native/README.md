# Native Examples

This crate provides standalone Rust examples for the public `cow-sdk` surface.
The scenarios are deterministic by default and avoid browser, extension, and
live order-submission requirements.

## Scenarios

| Scenario | Coverage |
| --- | --- |
| `sdk_surface_report` | Public facade inventory for the root crate. |
| `app_data_roundtrip` | App-data document generation, hashing, and schema-facing output. |
| `signing_roundtrip` | Order signing, cancellation signing, and EIP-1271 payload generation. |
| `quote_only_simulation` | Quote-only trading flow without order submission. |
| `limit_order_simulation` | Limit-order construction and simulated submission. |
| `order_lifecycle_simulation` | Order lookup plus off-chain cancellation. |
| `trading_sdk_simulation` | High-level `TradingSdk` quote, allowance, and approval flow. |
| `ethflow_transaction_simulation` | native-sell / EthFlow transaction construction and simulated submission through `get_eth_flow_transaction` and `post_sell_native_currency_order`. |
| `onchain_order_actions_simulation` | pre-sign transaction generation plus regular-order and EthFlow on-chain cancellation routing. |
| `orderbook_transport_roundtrip` | Mocked orderbook request and response flow. |
| `orderbook_live_probe` | Optional live orderbook version probe through the typed SDK client. |
| `subgraph_query_roundtrip` | Canonical query helper flow through `cow-sdk-subgraph`. |
| `subgraph_custom_query_roundtrip` | Explicit `SubgraphQueryRequest` flow for custom GraphQL documents. |
| `subgraph_live_query` | Optional live subgraph query with explicit environment configuration. |

Subgraph scenarios use `cow-sdk-subgraph` directly instead of the root facade.

## Validation Commands

```text
cargo check --manifest-path examples/native/Cargo.toml --examples
cargo test --manifest-path examples/native/Cargo.toml
```

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

Before running `orderbook_live_probe`, optionally set:

- `COW_SMOKE_ORDERBOOK_ENV`
- `COW_SMOKE_ORDERBOOK_CHAIN_ID`
- `COW_SMOKE_ORDERBOOK_BASE_URL`
- `COW_SMOKE_ORDERBOOK_API_KEY`

The example defaults to the production mainnet orderbook endpoint when these variables are not supplied.

Before running `subgraph_live_query`, set:

- `THE_GRAPH_API_KEY`
- optionally `COW_SUBGRAPH_CHAIN_ID`; the default is mainnet

For an operator-oriented smoke pass that classifies unavailable services separately from regressions, use:

```text
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- orderbook-live
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- subgraph-live
```
