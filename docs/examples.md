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
| Work with app-data and signing | `app_data_roundtrip`, `signing_roundtrip` |
| Quote, build, cancel, and simulate trading flows | `quote_only_simulation`, `cancellation_combinator`, `limit_order_simulation`, `trading_sdk_simulation` |
| Inspect order lifecycle and on-chain actions | `order_lifecycle_simulation`, `ethflow_transaction_simulation`, `onchain_order_actions_simulation` |
| Inspect typed orderbook transport | `orderbook_transport_roundtrip` |
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

Use the WASM examples when you need browser-facing verification surfaces.

| Surface | Package | Purpose |
| --- | --- | --- |
| [`sdk-verification-console`](../examples/wasm/sdk-verification-console/README.md) | `cow-sdk-verification-console` | Deterministic SDK verification and browser inspection for WASM-compatible surfaces |
| [`browser-wallet-console`](../examples/wasm/browser-wallet-console/README.md) | `cow-sdk-browser-wallet-console` | Mock-wallet proof plus explicit injected-wallet flows for browser-runtime support |

For the two-tier browser-runtime proof posture these consoles follow, see
[Browser-runtime proof posture](browser-runtime-proof-posture.md).

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

## Adding A WASM Console

WASM consoles under `examples/wasm/` are verification dashboards, not
pedagogical playgrounds. New consoles extend the existing surface without
diluting that genre. The rules below govern naming, shape, and scope.

### Naming

- Folder: `examples/wasm/<capability>-console/` in kebab-case, suffix
  `-console`.
- Cargo package name: `cow-sdk-<capability>-console`. Drop the inner `sdk-`
  only when the literal substitution would repeat, so the folder
  `sdk-verification-console/` maps to the package `cow-sdk-verification-console`
  rather than `cow-sdk-sdk-verification-console`.
- Playwright lane folder: `e2e/<capability>/`.
- Hosted Pages path: `<capability>-console/` under the repository Pages host.

### Shape

Every console ships with:

- A one-sentence user-outcome subheading immediately under the H1 in both the
  README and the HTML landing page
- A primary walkthrough entry that drives a deterministic flow end-to-end so
  the first reviewer click exercises a signed result
- A persistent mode indicator exposing env, chain, wallet, and last action
  while the reader scrolls
- A visible hosted-build link when the page is not already served from the
  hosted Pages host
- A README on the fixed template shape: H1, user-outcome subheading,
  What this shows, Modes, Build, Serve, Validation, Hosted build, Related
- Deterministic host-side Rust tests plus an in-browser `wasm-bindgen-test`
  lane and a route-mocked Playwright lane

### Hybrid Extensibility

- A capability that introduces a new user workflow in the browser lands as a
  new `examples/wasm/<capability>-console/` crate with its own Playwright lane
  and hosted Pages path.
- A capability that is a deterministic SDK addition without a new user
  workflow extends the existing sdk-verification console as one or more new
  panels inside `cow-sdk-verification-console` rather than forking a new
  console crate.
- When in doubt, default to a panel. Lifting a panel to its own console later
  is cheaper than splitting an over-broad console after the fact.

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
- Use the SDK verification console when you need browser-hosted WASM proof.
- Use the browser wallet console when you need explicit wallet authorization
  flows in the browser.
- The browser-facing consoles enable static browser-live CoW orderbook actions
  on `staging`; production requires a proxy-enabled deployment instead of the
  shipped static page.
