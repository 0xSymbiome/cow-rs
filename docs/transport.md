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
  cancellation). Consumers can bring their own provider through the
  `docs/providers/` adapter guide or use the native Alloy provider adapter.
- **`AsyncSigningProvider`** in `cow-sdk-core` extends `AsyncProvider` for
  async providers that can create signers. Read-only provider adapters do not
  implement this extension.

Native Alloy runtime dependencies are explicit opt-ins. `alloy-provider` is
allowed only in `cow-sdk-alloy-provider` and `cow-sdk-alloy`, and
`alloy-signer-local` is allowed only in `cow-sdk-alloy-signer` and
`cow-sdk-alloy`. CI normalises these allow-list checks through
`cargo check-alloy-provider-invariant` and
`cargo check-alloy-signer-invariant`, so the default facade stays
provider-neutral.

## Native Alloy Adapters

`cow-sdk-alloy-provider` implements `AsyncProvider` for read-only Alloy RPC
access. `cow-sdk-alloy-signer` implements `AsyncSigner` for local private-key
message and typed-data signing. `cow-sdk-alloy` composes both through an Alloy
wallet-filler provider so allowance, approval, pre-sign, and on-chain
cancellation helpers can use one native client.

The Alloy adapter crates are native-only. On `wasm32-unknown-unknown`, use
`cow-sdk-browser-wallet` for signing and inject browser RPC access through the
browser runtime instead.

## The `HttpTransport` Trait

```rust,ignore
#[async_trait(?Send)]
pub trait HttpTransport: std::fmt::Debug {
    async fn get(
        &self,
        path: &str,
        headers: &[(String, String)],
        timeout: Option<std::time::Duration>,
    ) -> Result<String, TransportError>;
    async fn post(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        timeout: Option<std::time::Duration>,
    ) -> Result<String, TransportError>;
    async fn put(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        timeout: Option<std::time::Duration>,
    ) -> Result<String, TransportError>;
    async fn delete(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        timeout: Option<std::time::Duration>,
    ) -> Result<String, TransportError>;
}
```

Implementations return the raw response body as a `String` on success
or a typed `TransportError` on failure. The trait is dyn-compatible
through `async-trait`, so injected clients can share a transport handle
across native and browser callers. Native futures are `Send`; browser
futures drop that bound so the `FetchTransport` implementation remains
viable. Callers that install a transport on the orderbook or subgraph
builders wrap it in an `Arc<dyn HttpTransport + Send + Sync>`.
The default seam is request/response only; it does not expose Server-Sent
Events or streaming subscriptions.

## The Native Default: `ReqwestTransport`

On native targets, `cow-sdk-core::ReqwestTransport` is the ready-to-use
default. `OrderBookApi::builder()` and `SubgraphApi::builder()` install
it automatically when the caller does not supply `.transport(...)`.

```rust,ignore
use cow_sdk::{CowEnv, OrderBookApi, SupportedChainId};

let orderbook = OrderBookApi::builder()
    .chain(SupportedChainId::Mainnet)
    .environment(CowEnv::Prod)
    .build()?;
```

For explicit control, build a `ReqwestTransport` from a configuration:

```rust,ignore
use std::sync::Arc;
use cow_sdk::{HttpTransport, ReqwestTransport, ReqwestTransportConfig};

let config = ReqwestTransportConfig::new("https://api.cow.fi")
    .with_user_agent("my-bot/1.0");
let transport: Arc<dyn HttpTransport + Send + Sync> =
    Arc::new(ReqwestTransport::new(config)?);
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
use cow_sdk::{CowEnv, HttpTransport, OrderBookApi, SupportedChainId};
use cow_sdk_transport_wasm::{FetchTransport, FetchTransportConfig};

let transport: Arc<dyn HttpTransport + Send + Sync> = Arc::new(FetchTransport::new(
    &FetchTransportConfig::new("https://api.cow.fi"),
));
let orderbook = OrderBookApi::builder()
    .chain(SupportedChainId::Mainnet)
    .environment(CowEnv::Prod)
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

```rust,no_run
use async_trait::async_trait;
use cow_sdk::{HttpTransport, TransportError};
use std::{collections::HashMap, time::Duration};

#[derive(Debug, Clone, Default)]
pub struct FixtureTransport {
    responses: HashMap<String, String>,
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl HttpTransport for FixtureTransport {
    async fn get(
        &self,
        path: &str,
        _headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        self.responses
            .get(path)
            .cloned()
            .ok_or_else(|| TransportError::Configuration {
                message: format!("no fixture for GET {path}").into(),
            })
    }

    async fn post(
        &self,
        path: &str,
        _body: &str,
        _headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        self.responses
            .get(path)
            .cloned()
            .ok_or_else(|| TransportError::Configuration {
                message: format!("no fixture for POST {path}").into(),
            })
    }

    async fn put(
        &self,
        path: &str,
        _body: &str,
        _headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        self.responses
            .get(path)
            .cloned()
            .ok_or_else(|| TransportError::Configuration {
                message: format!("no fixture for PUT {path}").into(),
            })
    }

    async fn delete(
        &self,
        path: &str,
        _body: &str,
        _headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        self.responses
            .get(path)
            .cloned()
            .ok_or_else(|| TransportError::Configuration {
                message: format!("no fixture for DELETE {path}").into(),
            })
    }
}
```

Install it through the builder's `.transport(...)` setter:

```rust,ignore
use std::sync::Arc;
use cow_sdk::{OrderBookApi, SupportedChainId};

let transport: Arc<dyn cow_sdk::HttpTransport + Send + Sync> = Arc::new(fixture_transport);
let orderbook = OrderBookApi::builder()
    .chain(SupportedChainId::Mainnet)
    .environment(/* prod | staging */)
    .transport(transport)
    .build()?;
```

The same pattern works for bridging deployments, custom retry layers,
authenticated proxies, or in-process mock servers.

## Transport Policy

The transport-policy layer (retry, rate-limit, user-agent, cooldowns, and
classification) sits above the trait and is unchanged by the transport choice.
`cow-sdk-transport-policy::TransportPolicy` is consumed by both the orderbook
and subgraph builders through `.transport_policy(...)`, while `cow-sdk` exposes
the same types under `cow_sdk::http`. The default orderbook and subgraph
policies preserve the reviewed retryable status set (`408`, `425`, `429`,
`500`, `502`, `503`, `504`), honor `Retry-After` on `429` and `503`, and keep
rate-limit state instance-scoped.

## Related Docs

- [Architecture](architecture.md) — how `HttpTransport` fits into the
  workspace's published family
- [Integrations](integrations.md) — broader runtime-adapter guide
  covering `Signer`, `AsyncSigner`, `Provider`, `AsyncProvider`,
  `AsyncSigningProvider`, and `HttpTransport`
- [Performance](performance.md) — shared-client pooling recipes and
  default-transport policy
- [Observability](observability.md) — tracing boundary and the
  transport-layer span lattice
- [ADR 0013](adr/0013-http-transport-injection-and-typestate-builders.md)
  — the architectural rule behind the seam
- [ADR 0041](adr/0041-transport-policy-l3-layering.md)
  — the shared retry and rate-limit policy layer
