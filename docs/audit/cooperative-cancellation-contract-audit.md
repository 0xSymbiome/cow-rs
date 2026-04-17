# Cooperative Cancellation Contract Audit

Status: Current
Last reviewed: 2026-04-17
Owning surface: Cross-cutting cooperative cancellation across `cow-sdk-core`, `cow-sdk-orderbook`, `cow-sdk-subgraph`, and `cow-sdk-trading`
Refresh trigger: Changes to the cancellation token re-export, to the `_with_cancellation` public entry points, to the typed `Cancelled` error variants, or to the biased `select!` implementation path
Related docs:
- [ADR 0010](../adr/0010-runtime-neutral-async-and-transport-posture.md)
- [Architecture](../architecture.md)
- [Observability](../observability.md)

## Scope

This audit covers:

- the shared `CancellationToken` re-export on `cow-sdk-core`
- representative `_with_cancellation` entry points on the orderbook,
  subgraph, and trading public surfaces
- typed `Cancelled` variants on `CoreError`, `OrderbookError`,
  `SubgraphError`, and `TradingError`
- the biased `tokio::select!` propagation pattern that drops in-flight
  request futures promptly when the caller cancels

It does not cover browser-wallet session cancellation, unrelated transport
policy, or future capability crates outside the published surface.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Shared token import | One typed cancellation token re-export across every public crate | Conforms |
| `_with_cancellation` entry points | Long-running operations accept a token reference and return `Cancelled` when it fires | Conforms |
| Typed `Cancelled` variants | Every affected error aggregate surfaces cancellation as a discrete typed variant | Conforms |
| Biased `select!` propagation | In-flight futures are dropped promptly so sockets release rather than waiting on deadlines | Conforms |

## Current Contract

### Shared Token Import

`cow-sdk-core` re-exports `tokio_util::sync::CancellationToken` as
`cow_sdk_core::CancellationToken`. Every downstream crate routes
cancellation through that one typed import so consumers do not mix
independent tokens across crate boundaries.

### `_with_cancellation` Entry Points

Representative long-running operations expose a `_with_cancellation`
variant that accepts `&CancellationToken`. The non-cancellation wrappers
construct a default token and delegate to the cancellation path, so
existing callers observe no behavioural change.

### Typed `Cancelled` Variants

`CoreError`, `OrderbookError`, `SubgraphError`, and `TradingError` each
expose a discrete `Cancelled` variant. Facade aggregation through
`SdkError` preserves the variant so downstream telemetry can distinguish
cancellation from transport or validation failure without pattern-matching
error sources.

### Biased `select!` Propagation

Internal implementation drives a biased `tokio::select!` against
`token.cancelled()`. When the token fires, the SDK drops the in-flight
request future so the underlying socket releases promptly rather than
waiting for the request deadline. Cancellation is cooperative: the caller
owns the token and can clone it to propagate shutdown across multiple
SDK instances.

## Evidence

Primary implementation points:

- `crates/core/src/lib.rs`
- `crates/core/src/errors.rs`
- `crates/orderbook/src/api.rs`
- `crates/orderbook/src/error.rs`
- `crates/subgraph/src/api.rs`
- `crates/subgraph/src/error.rs`
- `crates/trading/src/sdk.rs`
- `crates/trading/src/error.rs`

Primary regression coverage:

- `crates/orderbook/tests/api_contract.rs`
- `crates/subgraph/tests/api_contract.rs`
- `crates/trading/tests/sdk_contract.rs`

Validation surface:

```text
cargo fmt --all --check
cargo test -p cow-sdk-core
cargo test -p cow-sdk-orderbook
cargo test -p cow-sdk-subgraph
cargo test -p cow-sdk-trading
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --target wasm32-unknown-unknown -p cow-sdk
```
