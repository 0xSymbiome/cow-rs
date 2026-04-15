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

| Surface | Purpose |
| --- | --- |
| [`sdk-verification-console`](../examples/wasm/sdk-verification-console/README.md) | Deterministic SDK verification and browser inspection for WASM-compatible surfaces |
| [`browser-wallet-console`](../examples/wasm/browser-wallet-console/README.md) | Mock-wallet proof plus explicit injected-wallet flows for browser-runtime support |

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
