# Native Examples

These examples demonstrate the public Rust SDK surface without requiring
browser runtimes or live order placement.

If you are starting from scratch, begin with
[Getting Started](../../docs/getting-started.md). This page is the native
scenario catalog that extends that canonical onboarding path.

Two complementary example lanes live in this repository:

- The scenario catalog below runs from the aggregate package
  `cow-sdk-examples-native` and shows cross-crate flows that combine the
  public facade with the lower-level crates.
- Per-crate examples under each individual crate show the shortest public
  surface for a single crate against recorded fixtures or local mock
  transports, so a reviewer can read one file and see how that crate is
  consumed in isolation. See [Per-Crate Examples](#per-crate-examples)
  below.

## Recommended First Sequence

Use this order when you want the shortest deterministic path:

1. `signing_roundtrip`
2. `limit_order_simulation`
3. `trading_sdk_simulation`

After that, branch by goal through the full scenario table below.

## Scenarios

| Scenario | Purpose |
| --- | --- |
| `sdk_surface_report` | Report the root facade surface |
| `app_data_roundtrip` | Generate and inspect app-data output |
| `signing_roundtrip` | Sign orders and cancellations and inspect typed payloads |
| `quote_only_simulation` | Build a quote flow without submission |
| `cancellation_combinator` | Cancel an in-flight quote with `Cancellable::cancel_with(&token)` |
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
| `alloy_quickstart` | Build the composed native Alloy client against a mock RPC |
| `alloy_provider_only` | Use the read-only Alloy provider leaf against a mock RPC |
| `alloy_signer_only` | Sign a real CoW order typed-data payload with the Alloy signer leaf |
| `alloy_provider_with_custom_signer` | Pair the Alloy provider leaf with a consumer-supplied async signer |
| `alloy_signer_with_custom_provider` | Pair the Alloy signer leaf with a consumer-supplied async provider |
| `alloy_trading_full_flow` | Invoke allowance, approval, and pre-sign TradingSdk async boundaries through `AlloyClient` |

Subgraph scenarios use `cow-sdk-subgraph` directly rather than the root
facade.

## Validation

```text
cargo check --manifest-path examples/native/Cargo.toml --examples
cargo test --manifest-path examples/native/Cargo.toml
cargo run-deterministic-examples
```

## Running Examples

```text
cargo run --manifest-path examples/native/Cargo.toml --example sdk_surface_report
cargo run --manifest-path examples/native/Cargo.toml --example app_data_roundtrip
cargo run --manifest-path examples/native/Cargo.toml --example signing_roundtrip
cargo run --manifest-path examples/native/Cargo.toml --example quote_only_simulation
cargo run --manifest-path examples/native/Cargo.toml --example cancellation_combinator
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
cargo run --manifest-path examples/native/Cargo.toml --example alloy_quickstart --features alloy
cargo run --manifest-path examples/native/Cargo.toml --example alloy_provider_only --features alloy-provider
cargo run --manifest-path examples/native/Cargo.toml --example alloy_signer_only --features alloy-signer
cargo run --manifest-path examples/native/Cargo.toml --example alloy_provider_with_custom_signer --features alloy-provider
cargo run --manifest-path examples/native/Cargo.toml --example alloy_signer_with_custom_provider --features alloy-signer
cargo run --manifest-path examples/native/Cargo.toml --example alloy_trading_full_flow --features alloy
```

The live probes are opt-in follow-ons. They extend the deterministic onboarding
path; they do not replace it.

## Optional Environment Variables

Before running `orderbook_live_probe`, you can set:

- `COW_SMOKE_ORDERBOOK_ENV`
- `COW_SMOKE_ORDERBOOK_CHAIN_ID`
- `COW_SMOKE_ORDERBOOK_BASE_URL`
- `COW_SMOKE_ORDERBOOK_API_KEY`

Before running `subgraph_live_query`, set:

- `THE_GRAPH_API_KEY`
- optionally `COW_SUBGRAPH_CHAIN_ID`

## Per-Crate Examples

Each leaf crate that owns a durable public surface carries a small,
self-contained example that demonstrates the crate's primary user
journey against a recorded fixture or a local mock transport. These
examples compile under the pinned MSRV and require no RPC credentials.

| Crate | Example | Primary user journey |
| --- | --- | --- |
| `cow-sdk-trading` | `signed_order_end_to_end` | full quote → sign → post flow through `TradingSdk::builder()` against an injected in-process orderbook and signer |
| `cow-sdk-trading` | `typestate_builder_example` | ready versus helper-only builder terminals and their fail-closed mode boundary |
| `cow-sdk-orderbook` | `paginated_orders_fetch` | paginated `GetOrdersRequest` loop through `OrderBookApi::builder_from_context(...).base_url(...).build()` against a `wiremock::MockServer` |
| `cow-sdk-subgraph` | `typed_query_with_escape_hatch` | canonical `TOTALS_QUERY` typed path plus the explicit `run_query` raw-document escape hatch, both against a `wiremock::MockServer` |

Run them with:

```text
cargo run -p cow-sdk-trading --example signed_order_end_to_end
cargo run -p cow-sdk-trading --example typestate_builder_example
cargo run -p cow-sdk-orderbook --example paginated_orders_fetch
cargo run -p cow-sdk-subgraph --example typed_query_with_escape_hatch
```

### Recorded-Fixture And Mock-Transport Patterns

All three per-crate examples use one of two lightweight patterns so the
example stays runnable without network access:

- **Recorded fixture + in-process trait impl** (`cow-sdk-trading`): a
  fixed `OrderQuoteResponse` JSON fixture plus an inline struct that
  implements the public `OrderbookClient` trait, backed by an inline
  signer that implements the public `Signer` trait. The SDK sees the
  same trait surface a real deployment would use.
- **HTTP mock transport** (`cow-sdk-orderbook`, `cow-sdk-subgraph`):
  a local `wiremock::MockServer` serves the HTTP or GraphQL shape the
  client expects, and the crate's `*::new_with_base_url` /
  `with_config` path points the client at the mock URL. This mirrors
  the upstream test pattern for each crate.

When a consumer wants to adapt a per-crate example to a real service,
replacing the mock server with the production URL (or replacing the
inline `OrderbookClient` with the live `OrderBookApi`) is the only
change required.
