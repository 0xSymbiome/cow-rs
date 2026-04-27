# Transport

This page explains how `cow-rs` dispatches HTTP requests to the CoW
Protocol orderbook and the subgraph, how to choose a transport on
native and browser targets, and how to plug in a custom transport
implementation for tests, bridging, or bespoke deployments.

## Two Runtime Seams, Not One

`cow-rs` exposes two orthogonal runtime seams. They never share a
concrete backend.

- **`HttpTransport`** in `cow-sdk-core` is the production HTTPS seam
  used by `cow-sdk-orderbook` and `cow-sdk-subgraph`. It dispatches
  REST and GraphQL traffic. The native default is `ReqwestTransport`
  in `cow-sdk-core`; the browser default is `FetchTransport` in the
  dedicated `cow-sdk-transport-wasm` crate.
- **`AsyncProvider`** in `cow-sdk-core` is the read-only chain-RPC seam used by
  on-chain helpers (allowance reads, EIP-1271 verification, on-chain
  cancellation). Consumers bring their own provider through the
  `docs/providers/` adapter guide.
- **`AsyncSigningProvider`** in `cow-sdk-core` extends `AsyncProvider` for
  async providers that can create signers. Read-only provider adapters do not
  implement this extension.

`alloy-provider` is intentionally not pulled by any shipped leaf
crate. `cargo tree --invert alloy-provider` returns empty for
`cow-sdk`, `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`,
`cow-sdk-app-data`, `cow-sdk-orderbook`, `cow-sdk-trading`,
`cow-sdk-subgraph`, and `cow-sdk-browser-wallet`, so consumers keep
full control of their chain-RPC runtime.

## The `HttpTransport` Trait

```rust,ignore
#[async_trait(?Send)]
pub trait HttpTransport: std::fmt::Debug {
    async fn get(&self, path: &str) -> Result<String, TransportError>;
    async fn post(&self, path: &str, body: &str) -> Result<String, TransportError>;
    async fn delete(&self, path: &str, body: &str) -> Result<String, TransportError>;
}
```

Implementations return the raw response body as a `String` on success
or a typed `TransportError` on failure. The trait is dyn-compatible
through `async-trait`, so `Arc<dyn HttpTransport>` composes cleanly
across native and browser callers. The returned futures are `!Send`
to keep the browser implementation viable; native consumers that
need `Send` wrap the transport in an `Arc<dyn HttpTransport + Send + Sync>`
newtype when required.

## The Native Default: `ReqwestTransport`

On native targets, `cow-sdk-core::ReqwestTransport` is the ready-to-use
default. `OrderBookApi::builder()` and `SubgraphApi::builder()` install
it automatically when the caller does not supply `.transport(...)`.

```rust,ignore
use cow_sdk::{OrderBookApi, SupportedChainId};

let orderbook = OrderBookApi::builder()
    .chain(SupportedChainId::Mainnet)
    .environment(/* prod | staging */)
    .build()?;
```

For explicit control, build a `ReqwestTransport` from a configuration:

```rust,ignore
use std::sync::Arc;
use cow_sdk::{HttpTransport, ReqwestTransport, ReqwestTransportConfig};

let config = ReqwestTransportConfig::new("https://api.cow.fi")
    .with_user_agent("my-bot/1.0");
let transport: Arc<dyn HttpTransport> = Arc::new(ReqwestTransport::new(config)?);
```

Multi-chain consumers reuse a single `reqwest::Client` across every
`OrderBookApi` and `SubgraphApi` instance through the builder's
convenience `.client(reqwest_client)` setter. This is a shortcut over
installing a shared `ReqwestTransport`; the connection-pool reuse is
the same.

## The Browser Default: `FetchTransport`

On `wasm32-unknown-unknown`, `cow-sdk-transport-wasm::FetchTransport`
bridges the same async signature through `web-sys::fetch` and
`wasm-bindgen-futures`. The builder's native default-transport
convenience is gated on `#[cfg(not(target_arch = "wasm32"))]`, so
browser consumers supply the transport explicitly:

```rust,ignore
use std::sync::Arc;
use cow_sdk::{HttpTransport, OrderBookApi, SupportedChainId};
use cow_sdk_transport_wasm::FetchTransport;

let transport: Arc<dyn HttpTransport> = Arc::new(FetchTransport::default());
let orderbook = OrderBookApi::builder()
    .chain(SupportedChainId::Mainnet)
    .environment(/* prod | staging */)
    .transport(transport)
    .build()?;
```

`FetchTransport` uses the default fetch redirect policy (auto-follow),
so the `TransportErrorClass::Redirect` variant is unreachable from the
browser side. Cross-adapter parity tests exercise every other
classification arm against both adapters.

## Typed Failures: `TransportError` And `TransportErrorClass`

Every transport adapter funnels failures into the same typed enum:

```rust,ignore
pub enum TransportError {
    Transport { class: TransportErrorClass, detail: String },
    Configuration { message: String },
}
```

`TransportErrorClass` is an exhaustive partition:

| Variant | Meaning |
| --- | --- |
| `Timeout` | Client-side request timeout |
| `Connect` | TCP or TLS handshake failure |
| `Redirect` | Upstream returned a redirect the adapter refused to follow |
| `Decode` | Response body decode failure |
| `Body` | Failure reading or writing the request or response body |
| `Builder` | Client-builder misconfiguration |
| `Request` | Request-level upstream failure |
| `Status` | Non-success HTTP status class |
| `Other` | Fallthrough for adapter-specific failure modes |

Both defaults strip the URL through `reqwest::Error::without_url` (on
native) and by explicit omission (on the browser) before wrapping, so
credential-bearing query strings never leak through `Debug` or
`Display`. Downstream error aggregates (`OrderbookError::Transport`,
`SubgraphError::Transport`, `AppDataError::Transport`) carry the same
classification partition.

## Bringing Your Own Transport

Any type that implements `HttpTransport` works as an injected
transport. A common pattern is a test transport that replays fixtures
without touching the network:

```rust,ignore
use async_trait::async_trait;
use cow_sdk::{HttpTransport, TransportError};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct FixtureTransport {
    responses: HashMap<String, String>,
}

#[async_trait(?Send)]
impl HttpTransport for FixtureTransport {
    async fn get(&self, path: &str) -> Result<String, TransportError> {
        self.responses
            .get(path)
            .cloned()
            .ok_or_else(|| TransportError::Configuration {
                message: format!("no fixture for GET {path}"),
            })
    }

    async fn post(&self, path: &str, _body: &str) -> Result<String, TransportError> {
        self.responses
            .get(path)
            .cloned()
            .ok_or_else(|| TransportError::Configuration {
                message: format!("no fixture for POST {path}"),
            })
    }

    async fn delete(&self, path: &str, _body: &str) -> Result<String, TransportError> {
        self.responses
            .get(path)
            .cloned()
            .ok_or_else(|| TransportError::Configuration {
                message: format!("no fixture for DELETE {path}"),
            })
    }
}
```

Install it through the builder's `.transport(...)` setter:

```rust,ignore
use std::sync::Arc;
use cow_sdk::{OrderBookApi, SupportedChainId};

let transport: Arc<dyn cow_sdk::HttpTransport> = Arc::new(fixture_transport);
let orderbook = OrderBookApi::builder()
    .chain(SupportedChainId::Mainnet)
    .environment(/* prod | staging */)
    .transport(transport)
    .build()?;
```

The same pattern works for bridging deployments, custom retry layers,
authenticated proxies, or in-process mock servers.

## Transport Policy

The transport-policy layer (retry, rate-limit, user-agent, pinning)
sits above the trait and is unchanged by the transport choice.
`OrderBookApi::builder().policy(...)` and
`SubgraphApi::builder().policy(...)` accept a typed policy, and both
builders preserve the default policy byte-for-byte when the setter is
not called.

## Related Docs

- [Architecture](architecture.md) — how `HttpTransport` fits into the
  nine-crate workspace
- [Integrations](integrations.md) — broader runtime-adapter guide
  covering `Signer`, `AsyncSigner`, `Provider`, `AsyncProvider`,
  `AsyncSigningProvider`, and `HttpTransport`
- [Performance](performance.md) — shared-client pooling recipes and
  default-transport policy
- [Observability](observability.md) — tracing boundary and the
  transport-layer span lattice
- [ADR 0013](adr/0013-http-transport-injection-and-typestate-builders.md)
  — the architectural rule behind the seam
