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
- **`Provider`** in `cow-sdk-core` is the read-only chain-RPC seam used by
  on-chain helpers (allowance reads, EIP-1271 verification, on-chain
  cancellation). Consumers can bring their own provider through the
  `docs/providers/` adapter guide or use the native Alloy provider adapter.
- **`SigningProvider`** in `cow-sdk-core` extends `Provider` for
  providers that can create signers. Read-only provider adapters do not
  implement this extension.

Native Alloy runtime dependencies are explicit opt-ins. `alloy-provider` is
allowed only in `cow-sdk-alloy-provider` and `cow-sdk-alloy`, and
`alloy-signer-local` is allowed only in `cow-sdk-alloy-signer` and
`cow-sdk-alloy`. CI normalises these allow-list checks through
`cargo check-alloy-provider-invariant` and
`cargo check-alloy-signer-invariant`, so the default facade stays
provider-neutral.

## Native Alloy Adapters

`cow-sdk-alloy-provider` implements `Provider` for read-only Alloy RPC
access. `cow-sdk-alloy-signer` implements `Signer` for local private-key
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
    ) -> Result<TransportResponse, TransportError>;
    async fn post(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        timeout: Option<std::time::Duration>,
    ) -> Result<TransportResponse, TransportError>;
    async fn put(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        timeout: Option<std::time::Duration>,
    ) -> Result<TransportResponse, TransportError>;
    async fn delete(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        timeout: Option<std::time::Duration>,
    ) -> Result<TransportResponse, TransportError>;
}
```

Implementations return a `TransportResponse` on success or a typed
`TransportError` on failure. `TransportResponse` carries the 2xx status
code, the response headers, and the body, with accessors that mirror the
`http` crate (`status()`, `headers()`, `header(name)`, `body()`,
`into_body()`); header values are held in the `Redacted<T>` newtype so
they never surface through `Debug`, and the body renders as a byte length
rather than its contents. Non-2xx responses stay on the typed error
channel through `TransportError::HttpStatus`, which carries the same
status, headers, and body shape, so success and failure share one
representation. The trait is dyn-compatible through `async-trait`, so
injected clients can share a transport handle across native and browser
callers. Native futures are `Send`; browser futures drop that bound so the
`FetchTransport` implementation remains viable. Callers that install a
transport on the orderbook or subgraph builders wrap it in an
`Arc<dyn HttpTransport + Send + Sync>`. The default seam is
request/response only; it does not expose Server-Sent Events or streaming
subscriptions.

## The Native Default: `ReqwestTransport`

On native targets, `cow-sdk-core::ReqwestTransport` is the ready-to-use
default. `OrderbookApi::builder()` and `SubgraphApi::builder()` install
it automatically when the caller does not supply `.transport(...)`.

```rust,ignore
use cow_sdk::core::{CowEnv, SupportedChainId};
use cow_sdk::orderbook::OrderbookApi;

let orderbook = OrderbookApi::builder()
    .chain(SupportedChainId::Mainnet)
    .env(CowEnv::Prod)
    .build()?;
```

For explicit control, build a `ReqwestTransport` from a configuration:

```rust,ignore
use std::sync::Arc;
use cow_sdk::http::{HttpTransport, ReqwestTransport, ReqwestTransportConfig};

let config = ReqwestTransportConfig::new("https://api.cow.fi")
    .with_user_agent("my-bot/1.0");
let transport: Arc<dyn HttpTransport + Send + Sync> =
    Arc::new(ReqwestTransport::new(config)?);
```

Multi-chain consumers reuse a single `reqwest::Client` across every
`OrderbookApi` and `SubgraphApi` instance through the builder's
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
use cow_sdk::core::{CowEnv, SupportedChainId};
use cow_sdk::orderbook::OrderbookApi;
use cow_sdk::http::HttpTransport;
use cow_sdk_transport_wasm::{FetchTransport, FetchTransportConfig};

let transport: Arc<dyn HttpTransport + Send + Sync> = Arc::new(FetchTransport::new(
    &FetchTransportConfig::new("https://api.cow.fi"),
));
let orderbook = OrderbookApi::builder()
    .chain(SupportedChainId::Mainnet)
    .env(CowEnv::Prod)
    .transport(transport)
    .build()?;
```

`FetchTransport` uses the default fetch redirect policy (auto-follow),
so the `TransportErrorClass::Redirect` variant is unreachable from the
browser side. Cross-adapter parity tests exercise every other
classification arm against both adapters.

## JavaScript Callback Transport

`cow-sdk-wasm` also exposes `JsCallbackHttpTransport` for JavaScript runtimes
that do not have a browser `Window` or that need to own HTTP dispatch. The
transport implements the same `cow_sdk_core::HttpTransport` trait, but calls a
host-provided `CowFetchCallback` with a typed request object.

The request object carries method, URL, headers, body, timeout, and a live
`AbortSignal`. The SDK assembles that object with JavaScript property writes so
the signal is not serialized. Timeout remains SDK-owned through
`globalThis.AbortController`; `TimerGuard` clears the opaque timeout handle and
drops its closure on success, throw, rejection, malformed response, or abort.

Use this path for Node.js 22 or 24 LTS, Cloudflare Workers, Deno, custom service
workers, and tests that need precise control over HTTP responses. Cloudflare
Workers consume the web-target package through `./cloudflare` and
`./cloudflare/wasm`, not the bundler target.

## TypeScript TransportConfig

After publication, the TypeScript facade exposes one transport option on every
client constructor:

```ts
import { OrderBookClient } from "<published-cow-sdk-wasm-package>";

const client = new OrderBookClient({
  chainId: 1,
  env: "prod",
  transport: { kind: "fetch" }
});
```

`transport: { kind: "fetch" }` uses `globalThis.fetch`. It is the shortest
path for browser, Node.js, and Worker hosts that already expose a standards
compatible fetch implementation. Pass `fetch` explicitly when the host uses a
wrapped, instrumented, or test-owned implementation:

```ts
const client = new OrderBookClient({
  chainId: 1,
  env: "prod",
  transport: { kind: "fetch", fetch: instrumentedFetch }
});
```

`transport: { kind: "callback", callback }` gives the host full ownership of
HTTP dispatch. The callback receives a `CowFetchRequest` with method, URL,
headers, optional body, optional timeout, and a live `AbortSignal`; it returns
a `CowFetchResponse` with status, optional headers, and optional body:

```ts
const client = new OrderBookClient({
  chainId: 1,
  env: "prod",
  transport: {
    kind: "callback",
    callback: async (request) => {
      const response = await fetch(request.url, {
        method: request.method,
        headers: request.headers,
        body: request.body,
        signal: request.signal
      });

      return {
        status: response.status,
        statusText: response.statusText,
        headers: Object.fromEntries(response.headers.entries()),
        body: await response.text()
      };
    }
  }
});
```

Every client also accepts `transportPolicy: TransportPolicyConfig`:

```ts
const client = new OrderBookClient({
  chainId: 1,
  env: "prod",
  transport: { kind: "fetch" },
  transportPolicy: {
    retryPolicy: { maxAttempts: 3, baseDelayMs: 200, maxDelayMs: 2_000 },
    requestRateLimiter: { tokensPerInterval: 5, intervalMs: 1_000, scope: "perHost" },
    jitterStrategy: "full",
    tracingEnabled: true,
    userAgent: "my-app/1.0"
  }
});
```

Omitted policy fields inherit the SDK defaults for the selected client. Per-call
`timeoutMs` and `signal` options remain separate from the constructor policy so
each request can still set its own cancellation and latency boundary.

## Typed Failures: `TransportError` And `TransportErrorClass`

Every transport adapter funnels failures into the same typed enum:

```rust,ignore
#[non_exhaustive]
pub enum TransportError {
    Transport { class: TransportErrorClass, detail: Redacted<String> },
    Configuration { message: Redacted<String> },
    HttpStatus {
        status: u16,
        headers: Vec<(String, Redacted<String>)>,
        body: Redacted<String>,
    },
}
```

The detail, message, and body strings are `Redacted<String>`, so any URL or
secret is stripped before the error is constructed. The `HttpStatus` variant
carries the numeric status, response headers, and body together, letting the
orderbook and subgraph layers classify a non-2xx response without re-parsing
rendered error text. The enum is `#[non_exhaustive]`, so downstream `match`
arms include a wildcard to stay forward-compatible.

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
use cow_sdk::http::{HttpTransport, TransportError};
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
use cow_sdk::core::SupportedChainId;
use cow_sdk::orderbook::OrderbookApi;

let transport: Arc<dyn cow_sdk::http::HttpTransport + Send + Sync> = Arc::new(fixture_transport);
let orderbook = OrderbookApi::builder()
    .chain(SupportedChainId::Mainnet)
    .env(/* prod | staging */)
    .transport(transport)
    .build()?;
```

The same pattern works for bridging deployments, custom retry layers,
authenticated proxies, or in-process mock servers.

## Transport Policy

The transport-policy layer (retry, rate-limit, user-agent, cooldowns, and
classification) sits above the trait and is unchanged by the transport choice.
`cow_sdk_core::transport::policy::TransportPolicy` is consumed by both the orderbook
and subgraph builders through `.transport_policy(...)`, while `cow-sdk` exposes
the same types under `cow_sdk::http`. The default orderbook and subgraph
policies preserve the reviewed retryable status set (`408`, `425`, `429`,
`500`, `502`, `503`, `504`), honor `Retry-After` on `429` and `503`, and keep
rate-limit state instance-scoped.

The same crate owns the retry driver itself: the orderbook, subgraph, and IPFS
clients run every attempt through one shared `run_with_retry` loop, so the
retry, backoff, `Retry-After`, rate-limit acquisition, and retry telemetry
behavior is defined once rather than per client. `Retry-After` HTTP-date
evaluation reads a target-neutral wall clock, so the retry path behaves the
same on native and browser targets.

### Retry safety for writes

The retry loop applies to every method, including the order-creation,
order-cancellation, and app-data write paths. This is safe because the
CoW Protocol write endpoints are idempotent on the server: order creation is
content-addressed by order UID (a replayed create is rejected as a duplicate,
never stored twice), cancellation is keyed by order state (a replayed cancel is
a no-op once the order is cancelled), and app-data registration is
content-addressed by hash (a replayed register matches the existing entry). A
quote request carries no durable state. So a retried write cannot create a
duplicate side effect. The one residual is benign: if a write commits on the
server but its response is lost in transit, the retry can surface a
"duplicate"/"already cancelled" response for an operation that actually
succeeded — callers confirm the real state with an order lookup
(`GET /orders/{uid}`). Retrying writes mirrors the upstream
`@cowprotocol/cow-sdk` policy.

## Related Docs

- [Architecture](architecture.md) — how `HttpTransport` fits into the
  workspace's published family
- [Integrations](integrations.md) — broader runtime-adapter guide
  covering `Signer`, `Provider`, `SigningProvider`, and `HttpTransport`
- [Performance](performance.md) — shared-client pooling recipes and
  default-transport policy
- [Observability](observability.md) — tracing boundary and the
  transport-layer span lattice
- [ADR 0013](adr/0013-http-transport-injection-and-typestate-builders.md)
  — the architectural rule behind the seam
- [ADR 0041](adr/0041-transport-policy-l3-layering.md)
  — the shared retry and rate-limit policy layer
- [ADR 0039](adr/0039-typescript-callable-wasm-sdk-surface.md)
  — the TypeScript-callable wasm SDK surface
- [ADR 0040](adr/0040-wallet-provider-callback-boundary-for-js-consumers.md)
  — the JavaScript callback boundary
