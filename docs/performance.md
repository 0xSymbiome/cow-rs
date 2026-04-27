# Performance Posture

This document maps the performance-sensitive surfaces of the `cow-rs` SDK
family and records the benchmark coverage that protects them against
regressions. Absolute microbenchmark numbers are hardware-sensitive; the
reported ranges below are intended to track order-of-magnitude shifts rather
than pin exact timings.

## Hot Paths

The benchmarked hot paths align with the `0x`-bounded workflows that appear in
every trading and settlement pipeline:

- Order hashing and UID packing for every signed order and batch cancellation.
- Typed-data payload construction for every signing round-trip.
- Deterministic app-data stringification for every persisted order metadata
  payload.
- Orderbook quote fee aggregation for every public quote surface.
- Limit-order construction for every quote-to-post flow.

## Benchmark Coverage

Each benchmark target uses the `criterion` harness and lives under the owning
crate's `benches/` directory. The workflow at `.github/workflows/benchmarks.yml`
compiles and runs the full suite on a scheduled cadence and publishes the HTML
and JSON reports as non-blocking build artifacts.

| Surface | Benchmark | Owning crate |
| --- | --- | --- |
| Order EIP-712 digest | `order_hashing::hash_order` | `cow-sdk-contracts` |
| Order UID pack and extract | `uid_packing::pack_order_uid_params`, `uid_packing::extract_order_uid_params` | `cow-sdk-contracts` |
| Signing typed-data envelope | `typed_data::order_typed_data_payload` | `cow-sdk-signing` |
| App-data deterministic stringify | `stringify::stringify_deterministic` | `cow-sdk-app-data` |
| Orderbook quote fee aggregation | `quote_cost::calculate_total_fee` | `cow-sdk-orderbook` |
| Trading limit-order construction | `order_build::get_order_to_sign` | `cow-sdk-trading` |

## Reported Ranges

The benchmarks are reported as coarse ranges because microbenchmark absolutes
are hardware-sensitive and day-to-day variance is expected. The ranges below
capture the latest scheduled-run measurements on a GitHub-hosted `ubuntu-latest`
runner; each bound is the min-max observed across a representative sampling
window, not a single absolute number.

| Benchmark | Reported range |
| --- | --- |
| `hash_order` | single-digit microseconds |
| `pack_order_uid_params` | sub-microsecond |
| `extract_order_uid_params` | sub-microsecond |
| `order_typed_data_payload` | single-digit microseconds |
| `stringify_deterministic` | single-digit microseconds |
| `calculate_total_fee` | sub-microsecond |
| `get_order_to_sign` | low-single-digit microseconds |

Refresh the table when the next scheduled run reports a shift that crosses one
of these order-of-magnitude boundaries.

## Running Locally

Compile the benchmarks without running them:

```text
cargo bench --workspace --no-run
```

Run a specific surface and print a textual summary:

```text
cargo bench -p cow-sdk-contracts -- --output-format bencher
```

Interactive HTML reports are written under `target/criterion/` after a full
run. The scheduled workflow uploads those reports as build artifacts for each
crate in the matrix.

## Zero-Copy Call Data

Settlement, interaction, and swap encoder outputs hold their call-data payload
as `bytes::Bytes`. Reference-counted cloning means fanning the same encoded
payload across multiple settlement candidates no longer reallocates, which
matters most inside tight solver-evaluation loops. Public JSON wire
serialisation remains a `0x`-prefixed hexadecimal string, so the storage change
is invisible to downstream consumers.

## Address Equality

`cow_sdk_core::Address` compares and hashes case-insensitively through the
lowercase normalised key while its `as_str` accessor preserves the input
casing. Equality on the public address boundary is therefore `O(n)` byte
comparisons without any intermediate allocation, which keeps token-registry
lookups and order-owner checks out of the allocator on every signed-order
path.

## Shared HTTP Transport Pattern

Production deployments that issue orderbook or subgraph requests across
several chains should pool a single native transport and share it with
every SDK client they construct. A shared transport keeps one TCP, TLS,
and HTTP/2 connection cache warm across all routes, cuts first-byte
latency for every subsequent request, and bounds the per-host
file-descriptor footprint.

The production HTTP injection point is the `HttpTransport` trait in
`cow-sdk-core`. On native targets, the shipped default adapter is
`ReqwestTransport`, which is a thin wrapper over a shared
`reqwest::Client`. Both public clients accept a pre-configured
`reqwest::Client` through their typestate builder's `.client(...)`
convenience step, which constructs a `ReqwestTransport` around the
supplied client and preserves any custom keep-alive, timeout, or TLS
settings verbatim:

- [`cow_sdk_orderbook::OrderBookApi::builder`] exposes `.client(shared)`.
- [`cow_sdk_subgraph::SubgraphApi::builder`] exposes the matching
  `.client(shared)` step on the subgraph gateway surface.

Callers that want to install a bespoke transport implementation — an
authenticated proxy, an in-process fixture transport, a retry adapter —
pass it through the builder's `.transport(Arc::new(...))` setter
instead. When neither `.client(...)` nor `.transport(...)` is called on
native targets, the builder installs a conservative `ReqwestTransport`
that tracks `reqwest`'s upstream defaults, which is the right choice
for the common single-chain consumer.

On `wasm32-unknown-unknown`, the shipped browser adapter is
`FetchTransport` from `cow-sdk-transport-wasm`. Browser consumers
install it explicitly through `.transport(...)`; the connection-pool
tuning recipe below does not apply because browser `fetch` manages its
own pool.

## HTTP/2 Keep-Alive Recipe

HTTP/2 keep-alive is a user opt-in, not a default, because the right values
depend on deployment topology. The recipe below reflects the typical
production-bot configuration: one shared client, long-lived connections, and
active HTTP/2 ping frames so the pool detects dead peers before user-facing
requests inherit the latency.

```rust,ignore
use std::time::Duration;

use cow_sdk_core::SupportedChainId;
use cow_sdk_orderbook::{CowEnv, OrderBookApi, DEFAULT_ORDERBOOK_USER_AGENT};
use cow_sdk_subgraph::SubgraphApi;

fn build_shared_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(DEFAULT_ORDERBOOK_USER_AGENT)
        // Cap request-level latency so a stalled peer cannot hold a worker
        // thread indefinitely.
        .timeout(Duration::from_secs(10))
        // HTTP/2 ping frames at a cadence well below the server keep-alive
        // window keep idle connections observably healthy.
        .http2_keep_alive_interval(Duration::from_secs(30))
        .http2_keep_alive_timeout(Duration::from_secs(10))
        .http2_keep_alive_while_idle(true)
        // Connection pool tuning: keep idle connections warm for 5 minutes
        // and cap concurrency per host so pool growth is predictable.
        .pool_idle_timeout(Duration::from_secs(300))
        .pool_max_idle_per_host(16)
        // TCP keep-alive at the socket layer catches half-open NAT entries
        // that never surface an HTTP/2 PING failure.
        .tcp_keepalive(Duration::from_secs(60))
        .build()
        .expect("shared client configuration must build")
}

fn assemble_sdk_clients(
    shared: reqwest::Client,
    chain: SupportedChainId,
    environment: CowEnv,
    subgraph_api_key: impl Into<String>,
) -> (OrderBookApi, SubgraphApi) {
    let orderbook = OrderBookApi::builder()
        .chain(chain)
        .environment(environment)
        .client(shared.clone())
        .build()
        .expect("orderbook client builds with canonical defaults");
    let subgraph = SubgraphApi::builder()
        .chain(chain)
        .api_key(subgraph_api_key)
        .client(shared)
        .build()
        .expect("subgraph client builds with canonical defaults");
    (orderbook, subgraph)
}
```

### Knob Summary

| Setting | Purpose |
| --- | --- |
| `timeout` | Upper bound on end-to-end request latency before the call fails. |
| `http2_keep_alive_interval` | Cadence of HTTP/2 PING frames on open connections. |
| `http2_keep_alive_timeout` | Grace period before a missing PING ack closes the connection. |
| `http2_keep_alive_while_idle` | Enables keep-alive even for connections with no active streams. |
| `pool_idle_timeout` | Longest an idle connection stays warm before eviction. |
| `pool_max_idle_per_host` | Cap on idle connections retained per destination host. |
| `tcp_keepalive` | Socket-layer keep-alive for catching half-open NAT entries. |
| `user_agent` | Stable identifier sent on every request so operators can correlate traffic. |

All settings above are operator opt-ins; the shipped default
`ReqwestTransport` adapter keeps upstream `reqwest` defaults so
single-chain consumers and short-lived scripts stay simple. Browser
consumers building for `wasm32-unknown-unknown` install `FetchTransport`
from `cow-sdk-transport-wasm` instead; the knob summary above does not
apply to that adapter because browser `fetch` owns its connection pool.
