# Native Examples

Standalone Rust examples using `cow-sdk`.

## Scenarios

- `sdk_surface_report`
  - deterministic SDK report
- `app_data_roundtrip`
  - app-data conversion and schema report
- `signing_roundtrip`
  - order and cancellation signing report
- `quote_only_simulation`
  - quote-only flow
- `limit_order_simulation`
  - limit-order flow
- `order_lifecycle_simulation`
  - order lookup and cancellation flow
- `orderbook_transport_roundtrip`
  - mocked orderbook flow
- `trading_sdk_simulation`
  - mocked trading flow
- `subgraph_query_roundtrip`
  - mocked subgraph flow

## Commands

```text
cargo check --manifest-path examples/native/Cargo.toml --examples
```

```text
cargo run --manifest-path examples/native/Cargo.toml --example sdk_surface_report
cargo run --manifest-path examples/native/Cargo.toml --example app_data_roundtrip
cargo run --manifest-path examples/native/Cargo.toml --example signing_roundtrip
cargo run --manifest-path examples/native/Cargo.toml --example quote_only_simulation
cargo run --manifest-path examples/native/Cargo.toml --example limit_order_simulation
cargo run --manifest-path examples/native/Cargo.toml --example order_lifecycle_simulation
cargo run --manifest-path examples/native/Cargo.toml --example orderbook_transport_roundtrip
cargo run --manifest-path examples/native/Cargo.toml --example trading_sdk_simulation
cargo run --manifest-path examples/native/Cargo.toml --example subgraph_query_roundtrip
```

`subgraph_query_roundtrip` uses `cow-sdk-subgraph`.
