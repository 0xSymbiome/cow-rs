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
| Quote, build, and simulate trading flows | `quote_only_simulation`, `limit_order_simulation`, `trading_sdk_simulation` |
| Inspect order lifecycle and on-chain actions | `order_lifecycle_simulation`, `ethflow_transaction_simulation`, `onchain_order_actions_simulation` |
| Inspect typed orderbook transport | `orderbook_transport_roundtrip` |
| Work with read-only subgraph access | `subgraph_query_roundtrip`, `subgraph_custom_query_roundtrip` |
| Run an opt-in live service check | `orderbook_live_probe`, `subgraph_live_query` |

See [Native examples](../examples/native/README.md) for commands and
environment notes.

## WASM

Use the WASM examples when you need browser-facing verification surfaces.

| Surface | Package | Purpose |
| --- | --- | --- |
| [`sdk-verification-console`](../examples/wasm/sdk-verification-console/README.md) | `cow-sdk-verification-console` | Deterministic SDK verification and browser inspection for WASM-compatible surfaces |
| [`browser-wallet-console`](../examples/wasm/browser-wallet-console/README.md) | `cow-sdk-browser-wallet-console-wasm` | Mock-wallet proof plus explicit injected-wallet flows for browser-runtime support |

For the two-tier browser-runtime proof posture these consoles follow, see
[Browser-runtime proof posture](browser-runtime-proof-posture.md).

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
- The native examples intentionally stay provider-agnostic. They use
  deterministic mocks or explicit transport surfaces rather than coupling the
  examples to one provider-specific adapter.
- Native runtime integrations plug into
  `cow-sdk-core::{Signer, AsyncSigner, Provider, AsyncProvider}`. That keeps
  provider-specific choices outside the default facade while preserving one
  stable seam for downstream adapters. See [Integrations](integrations.md) when
  you are ready to wire a custom runtime.

## Choosing A Starting Point

- Start with [Getting Started](getting-started.md) for the shortest path from
  the facade crate to deterministic signed-order output.
- Continue with native examples for trading, signing, app-data, and transport
  workflows.
- Use `cow-sdk-subgraph` examples when you need read-only subgraph access.
- Use the SDK verification console when you need browser-hosted WASM proof.
- Use the browser wallet console when you need explicit wallet authorization
  flows in the browser.
- The browser-facing consoles enable static browser-live CoW orderbook actions
  on `staging`; production requires a proxy-enabled deployment instead of the
  shipped static page.
