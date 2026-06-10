# Native Examples

These examples demonstrate the public Rust SDK surface without requiring
browser runtimes or live order placement.

If you are starting from scratch, begin with
[Getting Started](../../docs/getting-started.md). This page is the native
scenario catalog that extends that canonical onboarding path.

All consumer-facing examples live in this `cow-sdk-examples-native` cookbook and
consume the `cow-sdk` facade (`cow_sdk::...`) — the recommended single-dependency
path. The SDK ships no consumer examples that depend on the individual leaf
crates; the facade is the one entry point.

## Recommended First Sequence

Use this order when you want the shortest deterministic path:

1. `swap_quickstart`
2. `sign_order`
3. `limit_order`
4. `trading_full_cycle`

`swap_quickstart` uses the recommended fluent `Trading::swap()` builder — named
sell/buy/amount setters that cannot be transposed, then `execute` to quote,
sign, and post in one call (or `quote` to inspect before `submit`). It is the
shortest path from the facade to a posted order.

After that, branch by goal through the full scenario table below.

## Scenarios

| Scenario | Purpose |
| --- | --- |
| `facade_surface` | Report facade construction and the resolved on-chain deployment |
| `app_data` | Generate and inspect app-data output |
| `sign_order` | Sign orders and cancellations and inspect typed payloads |
| `quote` | Build a quote flow without submission |
| `slippage_suggester` | Quote with a consumer-supplied `SlippageSuggester` |
| `cancel_in_flight` | Cancel an in-flight quote with `Cancellable::cancel_with(&token)` |
| `limit_order` | Build and simulate signed limit-order submission |
| `eip1271_signer` | Post a limit order signed by a custom `Eip1271Signer` (smart account) |
| `order_lifecycle` | Inspect order lookup and off-chain cancellation |
| `receipt_lifecycle` | Drive `submit_and_wait_for_receipt` through the testing doubles for mined, reverted, and timeout outcomes |
| `swap_quickstart` | Make your first swap end to end (quote, sign, post) against a mock |
| `trading_full_cycle` | Inspect high-level `Trading` quote, allowance, and approval flow |
| `error_classification` | Classify failures with `CowError::class()` and decide retry versus abort |
| `ethflow` | Build native-sell / EthFlow transaction data |
| `ethflow_checker` | Avoid EthFlow order-id collisions with a custom `EthFlowOrderExistsChecker` |
| `onchain_actions` | Build pre-sign and on-chain cancellation transactions |
| `orderbook_transport` | Inspect typed orderbook transport behavior |
| `order_history` | List an account's orders and trade history through `OrderbookApi` |
| `orderbook_live` | Run an opt-in live orderbook version probe |
| `subgraph_query` | Inspect canonical subgraph helpers and the explicit `SubgraphQueryRequest` escape hatch |
| `subgraph_live` | Run an opt-in live subgraph query |
| `alloy_quickstart` | Build the composed native Alloy client against a mock RPC |
| `alloy_provider` | Use the read-only Alloy provider leaf against a mock RPC |
| `alloy_signer` | Sign a real CoW order typed-data payload with the Alloy signer leaf |
| `transaction_lifecycle` | Compare helper-based receipt waiting with broadcast-only submission through the composed Alloy signer |
| `alloy_custom_traits` | Compose an Alloy leaf with a consumer-supplied trait impl in both directions (SDK provider + your signer, SDK signer + your provider) |
| `alloy_trading_full_flow` | Invoke allowance, approval receipt waiting, native-currency wrapping (`wrap_interaction`), and pre-sign Trading async boundaries through `Client` |

Subgraph scenarios reach the subgraph surface through the `cow-sdk` `subgraph`
feature, the same way the other scenarios use the root facade.

### Test Doubles

Scenarios use one of two deterministic doubles, by intent:

- **`cow_sdk::testing` mocks** (`MockOrderbook`, `MockSigner`, `MockProvider`)
  for flow scenarios where the point is the SDK call sequence, not the wire.
- **`wiremock`** for transport and wire-shape scenarios
  (`orderbook_transport`, `order_history`,
  `error_classification`, `subgraph_query`) and for
  `cancel_in_flight`, where aborting an in-flight request cannot be
  shown against an instant in-memory double.

### Scenario Conventions

Every scenario opens with a `//!` module header — a one-line summary plus a
short body naming the key SDK symbols, the transport or double, and the one
design point worth knowing — kept in sync with its catalog row above. The body
then walks the `main` flow with step comments that mark each stage and call out
anything non-obvious, without narrating line by line.

## Validation

```text
cargo check --manifest-path examples/native/Cargo.toml --examples
cargo test --manifest-path examples/native/Cargo.toml
cargo run-deterministic-examples
```

## Running Examples

```text
cargo run --manifest-path examples/native/Cargo.toml --example facade_surface
cargo run --manifest-path examples/native/Cargo.toml --example app_data
cargo run --manifest-path examples/native/Cargo.toml --example sign_order
cargo run --manifest-path examples/native/Cargo.toml --example quote
cargo run --manifest-path examples/native/Cargo.toml --example cancel_in_flight
cargo run --manifest-path examples/native/Cargo.toml --example limit_order
cargo run --manifest-path examples/native/Cargo.toml --example order_lifecycle
cargo run --manifest-path examples/native/Cargo.toml --example receipt_lifecycle
cargo run --manifest-path examples/native/Cargo.toml --example swap_quickstart
cargo run --manifest-path examples/native/Cargo.toml --example trading_full_cycle
cargo run --manifest-path examples/native/Cargo.toml --example error_classification
cargo run --manifest-path examples/native/Cargo.toml --example ethflow
cargo run --manifest-path examples/native/Cargo.toml --example onchain_actions
cargo run --manifest-path examples/native/Cargo.toml --example orderbook_transport
cargo run --manifest-path examples/native/Cargo.toml --example order_history
cargo run --manifest-path examples/native/Cargo.toml --example slippage_suggester
cargo run --manifest-path examples/native/Cargo.toml --example eip1271_signer
cargo run --manifest-path examples/native/Cargo.toml --example ethflow_checker
cargo run --manifest-path examples/native/Cargo.toml --example orderbook_live
cargo run --manifest-path examples/native/Cargo.toml --example subgraph_query
cargo run --manifest-path examples/native/Cargo.toml --example subgraph_live
cargo run --manifest-path examples/native/Cargo.toml --example alloy_quickstart --features alloy
cargo run --manifest-path examples/native/Cargo.toml --example alloy_provider --features alloy-provider
cargo run --manifest-path examples/native/Cargo.toml --example alloy_signer --features alloy-signer
cargo run --manifest-path examples/native/Cargo.toml --example transaction_lifecycle --features alloy
cargo run --manifest-path examples/native/Cargo.toml --example alloy_custom_traits --features alloy
cargo run --manifest-path examples/native/Cargo.toml --example alloy_trading_full_flow --features alloy
```

The live probes are opt-in follow-ons. They extend the deterministic onboarding
path; they do not replace it.

## Optional Environment Variables

Before running `orderbook_live`, you can set:

- `COW_SMOKE_ORDERBOOK_ENV`
- `COW_SMOKE_ORDERBOOK_CHAIN_ID`
- `COW_SMOKE_ORDERBOOK_BASE_URL`
- `COW_SMOKE_ORDERBOOK_API_KEY`

Before running `subgraph_live`, set:

- `THE_GRAPH_API_KEY`
- optionally `COW_SUBGRAPH_CHAIN_ID`

## Example Placement Rule

Consumer-facing examples live in this `examples/native/` cookbook and import the
`cow-sdk` facade (`cow_sdk::...`) — the recommended single-dependency path. The
SDK does not ship consumer examples under individual crates' `examples/`
directories: depending on a leaf crate directly is not the recommended
consumption model, so an example that imported one would teach the wrong shape.
New examples are added as facade scenarios here.
