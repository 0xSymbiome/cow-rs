# Observability

The `cow-rs` SDK family ships an opt-in [`tracing`](https://docs.rs/tracing)
feature so host applications can route structured spans and events from the
SDK into their own subscriber without paying any dependency or runtime cost
when the feature is off.

## Enabling

The tracing support is gated behind per-crate `tracing` features and a
single facade feature on `cow-sdk` that activates them all in one step:

```toml
[dependencies]
cow-sdk = { version = "0.1", features = ["tracing"] }
# or, reaching individual crates directly:
cow-sdk-trading = { version = "0.1", features = ["tracing"] }
cow-sdk-orderbook = { version = "0.1", features = ["tracing"] }
cow-sdk-subgraph = { version = "0.1", features = ["tracing"] }
cow-sdk-signing = { version = "0.1", features = ["tracing"] }
cow-sdk-browser-wallet = { version = "0.1", features = ["tracing"] }
```

With the feature off the SDK emits zero spans and zero events, and none of
the `tracing` crate's types appear on the public surface.

## Baseline Subscriber

The simplest setup pairs `tracing-subscriber` with an environment-driven
filter so operators can dial the verbosity without recompiling:

```rust,ignore
use tracing_subscriber::{EnvFilter, fmt};

fn install_tracing() {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,cow_sdk=debug")),
        )
        .init();
}
```

## OpenTelemetry

Teams that already operate an OpenTelemetry collector can bridge the SDK's
spans through [`tracing-opentelemetry`](https://docs.rs/tracing-opentelemetry):

```rust,ignore
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::TracerProvider as SdkTracerProvider;
use tracing_subscriber::{Registry, layer::SubscriberExt, util::SubscriberInitExt};

fn install_otel() {
    let provider = SdkTracerProvider::builder().build();
    let tracer = provider.tracer("cow-rs");
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    Registry::default().with(otel_layer).init();
}
```

This is an advanced configuration; the baseline `tracing-subscriber` layer
is the expected entry point for most deployments.

## Field Registry

Instrumented spans and events emit a small, consistent set of fields so
downstream dashboards can pivot on the same names across every SDK call.

| Field | Type | Meaning |
| --- | --- | --- |
| `chain` | numeric or debug | Active chain id or `SupportedChainId` variant |
| `env` | string | Environment label (`prod` / `staging`) |
| `endpoint` | string | Stable route identity or GraphQL operation name |
| `method` | string | HTTP method (`GET`, `POST`, `DELETE`) for transport calls, or JSON-RPC-like operation name for wallet-mediated calls |
| `status` | numeric | HTTP status code once a response is received |
| `attempts` | numeric | Attempt index on retry-bearing paths |
| `duration_ms` | numeric | Elapsed time in milliseconds for the span |
| `order_uid` | string | Order UID of the target order |
| `quote_id` | numeric | Orderbook quote id returned by the service |
| `owner` | string | Owner address exposed on the request parameters |
| `scheme` | string | Signing scheme (`eip712`, `eth_sign`, `eip1271`, `pre_sign`) |

## Coverage

Tracing spans are emitted by every long-running public async method on
`cow-sdk-orderbook`, `cow-sdk-subgraph`, `cow-sdk-trading`, `cow-sdk-signing`,
and `cow-sdk-browser-wallet`. The per-crate surface below lists the
canonical entry points that carry `#[tracing::instrument]`; wrapper
methods that build a default [`cow_sdk_core::CancellationToken`] delegate
through their `_with_cancellation` sibling so the span is emitted exactly
once per call regardless of which public surface the caller used.

### `cow-sdk-orderbook`

Every public async method on `OrderBookApi` emits one span. Spans carry
`chain`, `env`, `endpoint`, and `method`; `order_uid` and `owner` are added
where the input parameters expose them.

- `get_version_with_cancellation`
- `get_quote_with_cancellation`
- `send_order_with_cancellation`
- `send_signed_order_cancellations_with_cancellation`
- `get_order_with_cancellation`
- `get_order_multi_env_with_cancellation`
- `get_orders_with_cancellation`
- `get_tx_orders_with_cancellation`
- `get_trades_with_cancellation`
- `get_order_competition_status_with_cancellation`
- `get_native_price_with_cancellation`
- `get_total_surplus_with_cancellation`
- `get_app_data_with_cancellation`
- `upload_app_data_with_cancellation`
- `get_solver_competition_by_auction_id_with_cancellation`
- `get_solver_competition_by_tx_hash_with_cancellation`
- `get_latest_solver_competition_with_cancellation`
- `get_auction_with_cancellation`

### `cow-sdk-subgraph`

Every top-level public async method on `SubgraphApi` emits one span.
Spans carry `chain`, `endpoint`, and `method`; subgraph does not have a
protocol `env` axis.

- `get_totals_with_cancellation`
- `get_last_days_volume_with_cancellation`
- `get_last_hours_volume_with_cancellation`
- `run_query_with_cancellation`

### `cow-sdk-trading`

Every public async method on `TradingSdk` plus the module-level async
helpers emit one span each. Spans carry `chain`, `env`, and `endpoint`;
`order_uid` is added on order-bound helpers.

- `get_quote_only_with_cancellation`
- `get_quote_results_with_cancellation`
- `get_quote_results_async_with_cancellation`
- `post_swap_order_with_cancellation`
- `post_swap_order_async_with_cancellation`
- `post_swap_order_from_quote_with_cancellation`
- `post_swap_order_from_quote_async_with_cancellation`
- `post_limit_order_with_cancellation`
- `post_limit_order_async_with_cancellation`
- `get_pre_sign_transaction_async_with_cancellation`
- `get_order_with_cancellation`
- `off_chain_cancel_order_with_cancellation`
- `off_chain_cancel_order_async_with_cancellation`
- `on_chain_cancel_order_with_cancellation`
- `on_chain_cancel_order_async_with_cancellation`
- `get_cow_protocol_allowance_async_with_cancellation`
- `approve_cow_protocol_async_with_cancellation`
- `post_swap_order_from_quote_async_with_cancellation` (module-level)
- `post_sell_native_currency_order_async_with_cancellation` (module-level)

### `cow-sdk-signing`

Local signing helpers carry `chain`, `scheme`, and `endpoint`. Signing is
chain-bound, not env-bound; the owner is determined by the supplied signer
and is not surfaced as a span field.

- `sign_order_with_scheme`
- `sign_order_with_scheme_async`
- `sign_order_cancellation_with_scheme`
- `sign_order_cancellation_with_scheme_async`
- `sign_order_cancellations_with_scheme`
- `sign_order_cancellations_with_scheme_async`

### `cow-sdk-browser-wallet`

Wallet-mediated chain operations carry `chain` and an explicit `method`
label identifying the operation.

- `BrowserWallet::signer_for_chain`
- `BrowserWallet::switch_chain`
- `BrowserWallet::switch_or_add_chain`

## Secrets

No traced span or event must ever carry a secret. Concretely:

- The `api_key` field of `ApiContext` and `ApiContextOverride` is
  `Redacted<String>`; its `Debug` implementation emits `[redacted]` so
  accidental `?` formatting cannot leak the value, and no instrumented
  call site records the field regardless.
- `IpfsConfig` credentials (`pinata_api_key`, `pinata_api_secret`) follow
  the same redaction contract and are never captured in traces.
- Wallet signatures, recovered public keys, and private-key material are
  never logged by the SDK. Downstream instrumentation that wants to record
  a signature should do so explicitly in host code.

If a future call site needs to record an identifier that is derived from
secret material, the convention is to hash or prefix-truncate it in the
host application before emitting it through the tracing subscriber.

## Error Classification

`cow_sdk::SdkError::class()` returns an `ErrorClass` so telemetry layers
can partition failures into `Validation`, `Transport`, `Remote`, `Signing`,
`Cancelled`, and `Internal` buckets without pattern-matching every nested
variant by hand. Retry policies typically only retry the `Transport` and
`Remote` classes; the other classes surface caller-side or protocol-level
conditions that benefit from different recovery paths.
