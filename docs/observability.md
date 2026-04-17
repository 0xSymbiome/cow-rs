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
| `method` | string | HTTP method (`GET`, `POST`, `DELETE`) |
| `status` | numeric | HTTP status code once a response is received |
| `attempts` | numeric | Attempt index on retry-bearing paths |
| `duration_ms` | numeric | Elapsed time in milliseconds for the span |
| `order_uid` | string | Redacted order UID (no owner bytes) |
| `quote_id` | numeric | Orderbook quote id returned by the service |
| `owner` | string | Lowercase-normalised owner address |
| `scheme` | string | Signing scheme (`eip712`, `eth_sign`, `eip1271`, `pre_sign`) |

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
