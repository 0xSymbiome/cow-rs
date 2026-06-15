# cow-sdk-transport-wasm

WebAssembly `fetch`-based HTTP transport for the [CoW Protocol](https://cow.fi)
Rust SDK. Implements `cow_sdk_core::HttpTransport` through the realm's global
`fetch`.

> ⚠️ **Alpha — `0.1.0-alpha`.** Pre-release and not security-audited; the public
> API may change before `0.1.0`. It is published as a pre-release, so Cargo
> selects it only when you opt in (`cow-sdk-transport-wasm = "0.1.0-alpha.1"`).
> Review it yourself before relying on it with real funds.

This crate is the `wasm32` implementation of `cow_sdk_core::HttpTransport`,
backed by the JavaScript `fetch` API. It is the browser sibling of core's
native-only `ReqwestTransport`, and the zero-config default transport for the
orderbook, subgraph, and trading clients on `wasm32` — so most consumers never
name it directly. It owns HTTP dispatch only; wallet and signer surfaces live in
[`cow-sdk-browser-wallet`](https://crates.io/crates/cow-sdk-browser-wallet).

The crate root is gated with `#![cfg(target_arch = "wasm32")]`, so every
non-wasm32 target sees an empty compilation unit. The transport reads `fetch`
from the realm's global scope (via `js_sys::global`, not `window()`), so it runs
on any `wasm32` realm that exposes a global `fetch` — a browser main thread, a
web worker, Cloudflare Workers, or Node 18+ — not the main thread alone.

## What it provides

- **`FetchTransport`** — a `cow_sdk_core::HttpTransport` implementation over the
  realm's global `fetch`, with per-request timeout enforced through an
  `AbortController` and per-call request headers.
- **`FetchTransportConfig`** — the base URL plus an optional request `timeout`
  and a `max_response_bytes` cap on the decoded body.
- **Typed, redacted errors** — every `fetch` failure surfaces through the shared
  `cow_sdk_core::TransportError` / `TransportErrorClass` taxonomy
  (`Timeout`, `Connect`, `Redirect`, `Decode`, `Body`, `Status`), identical to
  the native adapter, with URLs omitted from error output.
- **`dyn`-injectable** — compose it as
  `Arc<dyn HttpTransport + Send + Sync>` so typed clients stay runtime-neutral
  and never link the native `reqwest` stack.

## Install

```toml
[dependencies]
cow-sdk-transport-wasm = "0.1.0-alpha.1"
```

This crate compiles only for `wasm32-unknown-unknown`; on every other target it
is an empty crate.

## Example

```rust
# // `cow-sdk-transport-wasm` is empty on non-wasm32 targets, so the body is
# // gated to keep this example compiling under a native `cargo test --doc`.
# #[cfg(target_arch = "wasm32")]
# mod wasm_only {
use std::time::Duration;

use cow_sdk_core::HttpTransport;
use cow_sdk_transport_wasm::{FetchTransport, FetchTransportConfig};

pub fn build_transport() -> FetchTransport {
    FetchTransport::new(
        &FetchTransportConfig::new("https://api.cow.fi")
            .with_timeout(Duration::from_secs(20)),
    )
}

pub async fn version(
    transport: &FetchTransport,
) -> Result<String, cow_sdk_core::TransportError> {
    let response = transport.get("/api/v1/version", &[], None).await?;
    Ok(response.into_body())
}
# }
```

## Feature flags

| Feature | Default | Enables |
| --- | --- | --- |
| `tracing` | off | Emits one transport-layer `tracing` span around each request, and enables `cow-sdk-core`'s tracing. The public surface is unchanged. |

## Where this fits

It is a separate crate, not a module in core, on purpose. Every crate depends on
`cow-sdk-core`, so putting browser globals (`web-sys` / `js-sys` / `wasm-bindgen`)
into core would force them onto every `wasm32` build and remove the opt-out that
non-`fetch` consumers — the npm `cow-sdk-wasm` crate, custom transports — rely
on. So the browser default is an opt-in leaf and core stays transport-neutral on
`wasm32`. (The native `ReqwestTransport` can live in core because native targets
are homogeneous; `wasm32` targets are not.)

- **vs the native default:** `ReqwestTransport`
  ([`cow-sdk-core`](https://crates.io/crates/cow-sdk-core), `cfg(not(wasm32))`) is
  the sibling for native builds.
- **vs the JS-callback transport:** `cow-sdk-wasm` ships a `JsCallbackHttpTransport`
  for consumers who want JavaScript to own the HTTP call (a custom client, bespoke
  auth or headers, or a TypeScript app wiring its own stack). Use `FetchTransport`
  when Rust should acquire and call `fetch` itself.
- **vs the wallet transport:** [`cow-sdk-browser-wallet`](https://crates.io/crates/cow-sdk-browser-wallet)
  owns the EIP-1193 `Eip1193Transport` seam — a different trait, for wallet RPC and
  signing, not REST HTTP.

## Where to next

- [Getting Started](https://github.com/0xSymbiome/cow-rs/blob/main/docs/getting-started.md)
- [Transport Guide](https://github.com/0xSymbiome/cow-rs/blob/main/docs/transport.md)
- [cow-sdk-wasm README](https://github.com/0xSymbiome/cow-rs/blob/main/crates/wasm/README.md)
- [Workspace README](https://github.com/0xSymbiome/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-or-later. See the workspace
[LICENSE](https://github.com/0xSymbiome/cow-rs/blob/main/LICENSE)
file for the full text.
