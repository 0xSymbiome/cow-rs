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
- native Alloy transaction lifecycle timing

The native gallery includes dedicated scenarios for
`ethflow`, `onchain_actions`, and
`transaction_lifecycle`.

See [Native examples](native/README.md) for the full scenario list.

Run every deterministic non-live binary with:

```text
cargo run-deterministic-examples
```

## TypeScript WASM Package

The TypeScript-callable WASM package examples cover specialized JavaScript host
patterns:

- `wasm/cow-signer-node/` signs an order offline with EIP-712 and EIP-1271
  through the `signing` flavor.
- `wasm/cow-gateway-cloudflare/` runs an orderbook quote gateway on the
  `cloudflare` flavor inside a Worker.

Each example has its own README and `pnpm test` check.
