# Examples

The examples are organized by user goal rather than by crate internals.

If you are new to `cow-rs`, start with [Getting Started](getting-started.md)
first. This page is the branch point after the deterministic onboarding flow,
not a second landing page. This page branches by goal; the scenario catalog in
reference order lives in [Native examples](../examples/native/README.md).

## Native Rust

All consumer-facing examples are deterministic, transport-mocked recipes in the
`cow-sdk-examples-native` cookbook, and they consume the **`cow-sdk` facade**
(`cow_sdk::...`) — the recommended single-dependency path. Browse them by goal
below. The SDK ships no consumer examples that depend on the individual leaf
crates; the facade is the one entry point.

### Recommended: place a swap with `Trading::swap()`

The recommended way to place a swap is the fluent `Trading::swap()` builder:
named token setters that cannot be transposed, then `execute` to quote, sign,
and post in one call — or `quote` to inspect the result before `submit`. The
`swap_quickstart` scenario runs it end to end.

### Scenarios by goal

| Goal | Example surface |
| --- | --- |
| Make your first swap end to end (recommended `Trading::swap()`) | `swap_quickstart` |
| Learn the facade shape | `facade_surface` |
| Quote, then build, cancel, or simulate trading flows | `quote`, `slippage_suggester`, `cancel_in_flight`, `limit_order`, `trading_full_cycle` |
| Inspect order lifecycle and on-chain actions | `order_lifecycle`, `receipt_lifecycle`, `ethflow`, `ethflow_checker`, `onchain_actions` |
| Classify and handle SDK errors | `error_classification` |
| Work with app-data and signing | `app_data`, `sign_order`, `eip1271_signer` |
| Inspect typed orderbook transport | `orderbook_transport`, `order_history` |
| Work with read-only subgraph access | `subgraph_query` |
| Work with native Alloy adapters | `alloy_quickstart`, `alloy_provider`, `alloy_signer`, `transaction_lifecycle`, `alloy_custom_traits`, `alloy_trading_full_flow` |
| Run an opt-in live service check | `orderbook_live`, `subgraph_live` |

See [Native examples](../examples/native/README.md) for the full scenario
catalog, commands, and environment notes.

The deterministic non-live native example binaries share one smoke command:

```text
cargo run-deterministic-examples
```

## TypeScript WASM Package Examples

These are specialized examples. For most browser dapps, web apps, and
CowSwap-style UIs, the upstream
[`@cowprotocol/cow-sdk`](https://www.npmjs.com/package/@cowprotocol/cow-sdk)
TypeScript SDK is the recommended choice; it is substantially smaller at
equivalent feature subsets. Use the examples below when you are integrating
the TypeScript-callable WASM package from JavaScript or TypeScript runtimes
for specialized cases — deterministic Rust signing parity, single-source-of-
truth Rust + TypeScript embedding, or Cloudflare Workers (size-compatible
with the current Workers Paid/Bundled (~3 MB) compressed-size limit at the time of
measurement; the `trading` flavour's edge build is built and tested end-to-end
in CI (Workers Vitest), within the Workers compressed-size budget).

The package is published to npm as `@symbiome-forge/cow-sdk-wasm`. The
runnable WASM package examples live in the dedicated
[`0xSymbiome/cow-sdk-examples`](https://github.com/0xSymbiome/cow-sdk-examples)
repository; each carries its own README and `pnpm test` check.

| Runtime | Example | Purpose |
| --- | --- | --- |
| Node.js 22 or 24 | [`cow-signer-node`](https://github.com/0xSymbiome/cow-sdk-examples/tree/main/examples/wasm/cow-signer-node) | Sign an order offline with EIP-712 and EIP-1271 using the `signing` flavor |
| Cloudflare Workers | [`cow-gateway-cloudflare`](https://github.com/0xSymbiome/cow-sdk-examples/tree/main/examples/wasm/cow-gateway-cloudflare) | Run an orderbook quote gateway on the `trading` flavour's edge build |

## Integration Notes

- The default `cow-sdk` facade stays trading-first. Read-only analytics and
  custom GraphQL access are opt-in: enable the `cow-sdk` `subgraph` feature
  (surfaced as `cow_sdk::subgraph`) or depend on the standalone
  `cow-sdk-subgraph` crate.
- The native examples include both provider-agnostic deterministic flows and
  explicit Alloy adapter flows. Alloy scenarios run with `--features
  alloy-provider`, `--features alloy-signer`, or `--features alloy` depending
  on the smallest adapter surface they exercise. The `transaction_lifecycle`
  scenario shows the broadcast-hash result without receipt polling.
- Native runtime integrations plug into
  `cow-sdk-core::{Signer, Provider, SigningProvider}`. That keeps
  provider-specific choices outside the default facade while preserving one
  stable seam for downstream adapters. See [Integrations](integrations.md) when
  you are ready to wire a custom runtime.
- Quickstart surfaces may differ by audience (signer concretion, transport) but
  not by dialect (address idiom, naming case, order-id printing, amount value
  for the same asset).

## Choosing A Starting Point

- Start with [Getting Started](getting-started.md) for the shortest path from
  the facade crate to deterministic signed-order output.
- Continue with native examples for trading, signing, app-data, and transport
  workflows.
- Use the TypeScript WASM package examples for Node.js signing or Cloudflare
  Worker integration.
- Use `cow-sdk-subgraph` examples when you need read-only subgraph access.
- For browser-wallet trade flows, integrate the `cow-sdk-wasm` package with your
  app's own wallet stack (viem, wagmi, or any EIP-1193 provider); the SDK exposes
  the EIP-1193 request callback and the host supplies the wallet connection.
