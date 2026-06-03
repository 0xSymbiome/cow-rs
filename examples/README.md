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
`ethflow_transaction_simulation`, `onchain_order_actions_simulation`, and
`transaction_lifecycle`.

See [Native examples](native/README.md) for the full scenario list.

Run every deterministic non-live binary with:

```text
cargo run-deterministic-examples
```

## WASM

`examples/wasm/cow-trader-dioxus/` is a runnable browser-wallet trade example
(Dioxus, wasm): it discovers an injected wallet (EIP-6963), connects, signs, and
swaps a CoW order end to end using only `cow-sdk` public types.

See [the example README](wasm/cow-trader-dioxus/README.md) to build and run it.

## TypeScript WASM Package

The TypeScript-callable WASM package examples cover the main JavaScript host
patterns:

- `wasm-typescript-node-viem/` signs through viem's EIP-1193 request path.
- `wasm-typescript-browser-mm/` signs through a MetaMask-style injected wallet.
- `wasm-typescript-cloudflare-proxy/` initializes the Cloudflare flavor and
  proxies orderbook requests from a Worker.

Each example has its own README and `pnpm test` check.
