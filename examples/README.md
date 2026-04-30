# Examples

The repository includes native and WASM example galleries for the public
`cow-rs` surface.

## Native

`examples/native/` contains deterministic command-line scenarios for:

- app-data generation
- signing and cancellation payloads
- orderbook transport
- quote, limit-order, and lifecycle workflows
- native-sell / EthFlow transaction construction
- pre-sign and on-chain cancellation flows
- read-only subgraph access

The native gallery includes dedicated scenarios for
`ethflow_transaction_simulation` and `onchain_order_actions_simulation`.

See [Native examples](native/README.md) for the full scenario list.

Run every deterministic non-live binary with:

```text
cargo run-deterministic-examples
```

## WASM

`examples/wasm/` contains browser-facing verification surfaces for the SDK
facade and browser-wallet support.

See [WASM examples](wasm/README.md) for the WASM example index.
