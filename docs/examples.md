# Examples

The examples are organized by user goal rather than by crate internals.

If you are new to `cow-rs`, start with [Getting Started](getting-started.md)
first. This page is the branch point after the deterministic onboarding flow,
not a second landing page.

## Native Rust

Use the native examples when you want deterministic, transport-mocked flows for
the main SDK surfaces.

| Goal | Example surface |
| --- | --- |
| Learn the facade shape | `sdk_surface_report` |
| Make your first swap end to end | `simplest_swap_quickstart` |
| Classify and handle SDK errors | `error_classification_simulation` |
| Work with app-data and signing | `app_data_roundtrip`, `signing_roundtrip` |
| Quote, build, cancel, and simulate trading flows | `quote_only_simulation`, `cancellation_combinator`, `limit_order_simulation`, `trading_sdk_simulation` |
| Inspect order lifecycle and on-chain actions | `order_lifecycle_simulation`, `ethflow_transaction_simulation`, `onchain_order_actions_simulation` |
| Inspect typed orderbook transport | `orderbook_transport_roundtrip`, `order_list_history_simulation` |
| Work with read-only subgraph access | `subgraph_query_roundtrip`, `subgraph_custom_query_roundtrip` |
| Work with native Alloy adapters | `alloy_quickstart`, `alloy_provider_only`, `alloy_signer_only`, `transaction_lifecycle`, `alloy_provider_with_custom_signer`, `alloy_signer_with_custom_provider`, `alloy_trading_full_flow` |
| Run an opt-in live service check | `orderbook_live_probe`, `subgraph_live_query`, `live_order_sepolia` |

See [Native examples](../examples/native/README.md) for commands and
environment notes.

The deterministic non-live native and per-crate example binaries share one
smoke command:

```text
cargo run-deterministic-examples
```

## WASM

Use the WASM example when you want a runnable browser-wallet flow in Rust.

| Surface | Crate features | Purpose |
| --- | --- | --- |
| [`cow-trader-dioxus`](../examples/wasm/cow-trader-dioxus/README.md) | `cow-sdk` (`browser-wallet`) + `cow-sdk-transport-wasm` | Discover an injected wallet (EIP-6963), connect, sign, and swap a CoW order end to end in the browser — written entirely in Rust with Dioxus, using only SDK public types |

The example is a consumer demonstration that talks to the live orderbook. The
deterministic browser-runtime proof for the underlying contract lives in the
crate test lanes (`cow-sdk-browser-wallet`, `cow-sdk-transport-wasm`), described
in [Browser-runtime proof posture](browser-runtime-proof-posture.md).

## TypeScript WASM Package Examples

These are specialized examples. For most browser dapps, web apps, and
CowSwap-style UIs, the upstream
[`@cowprotocol/cow-sdk`](https://www.npmjs.com/package/@cowprotocol/cow-sdk)
TypeScript SDK is the recommended choice; it is substantially smaller at
equivalent feature subsets. Use the examples below when you are integrating
the TypeScript-callable WASM package from JavaScript or TypeScript runtimes
for specialized cases — deterministic Rust signing parity, single-source-of-
truth Rust + TypeScript embedding, or Cloudflare Workers (size-compatible
with the current Workers Free compressed-size limit at the time of
measurement; full Workers support pending release-bundle and startup
validation).

The examples use a repository-local package alias before publication;
application code should replace it with the final
`<published-cow-sdk-wasm-package>` package name.

| Runtime | Example | Purpose |
| --- | --- | --- |
| Node.js 22 or 24 with viem | [`wasm-typescript-node-viem`](../examples/wasm-typescript-node-viem/README.md) | Sign an order through viem's EIP-1193 request path |
| Browser with MetaMask injection | [`wasm-typescript-browser-mm`](../examples/wasm-typescript-browser-mm/README.md) | Sign an order with `window.ethereum` and `eth_signTypedData_v4` |
| Cloudflare Workers | [`wasm-typescript-cloudflare-proxy`](../examples/wasm-typescript-cloudflare-proxy/README.md) | Initialize the Cloudflare flavor and proxy orderbook requests |

## Integration Notes

- The default `cow-sdk` facade stays trading-first. If you need read-only
  analytics or custom GraphQL access, add `cow-sdk-subgraph` directly instead
  of expecting it from the facade.
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

## Choosing A Starting Point

- Start with [Getting Started](getting-started.md) for the shortest path from
  the facade crate to deterministic signed-order output.
- Continue with native examples for trading, signing, app-data, and transport
  workflows.
- Use the TypeScript WASM package examples for Node.js, browser-wallet, or
  Cloudflare Worker integration.
- Use `cow-sdk-subgraph` examples when you need read-only subgraph access.
- Use the `cow-trader-dioxus` WASM example when you want a runnable
  browser-wallet trade flow in Rust.
