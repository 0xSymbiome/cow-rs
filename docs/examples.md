# Examples

The examples demonstrate supported usage patterns for `cow-rs`.

## Native

`examples/native/scenarios/` covers deterministic and mocked Rust flows for app-data, signing, orderbook, trading, subgraph, and facade usage.

Subgraph usage is covered through three dedicated native scenarios:

- `subgraph_query_roundtrip.rs` for canonical helper usage
- `subgraph_custom_query_roundtrip.rs` for explicit custom GraphQL requests
- `subgraph_live_query.rs` for opt-in live execution with explicit environment configuration

## WASM

`examples/wasm/sdk-verification-console/` covers deterministic SDK verification and browser-facing inspection.

`examples/wasm/browser-wallet-console/` covers mock-wallet and injected-wallet browser flows through the public SDK.

## Usage

Use the example README files for exact commands and environment requirements.
