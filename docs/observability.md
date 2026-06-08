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
cow-sdk-contracts = { version = "0.1", features = ["tracing"] }
cow-sdk-orderbook = { version = "0.1", features = ["tracing"] }
cow-sdk-subgraph = { version = "0.1", features = ["tracing"] }
cow-sdk-signing = { version = "0.1", features = ["tracing"] }
cow-sdk-browser-wallet = { version = "0.1", features = ["tracing"] }
cow-sdk-cow-shed = { version = "0.1", features = ["tracing"] }
cow-sdk-transport-wasm = { version = "0.1", features = ["tracing"] }
cow-sdk-alloy-provider = { version = "0.1", features = ["tracing"] }
cow-sdk-alloy-signer = { version = "0.1", features = ["tracing"] }
cow-sdk-alloy = { version = "0.1", features = ["tracing"] }
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
| `chain` | numeric or string/debug | Active chain id, `SupportedChainId` variant, or platform label such as `wasm32` |
| `chain_id` | debug | Active chain id on caller spans that wrap lower-level contract helpers |
| `env` | string | Environment label (`prod` / `staging`) |
| `endpoint` | string | Stable route identity, GraphQL operation name, or path-only transport endpoint with scheme, authority, query, and fragment stripped |
| `method` | string | HTTP method (`GET`, `POST`, `PUT`, `DELETE`) for transport calls, or JSON-RPC-like operation name for wallet-mediated calls |
| `bytes_sent` | numeric | Request body byte length on transport-layer spans |
| `bytes_received` | numeric | Response body byte length on transport-layer spans after a response body is read |
| `status` | numeric | HTTP status code once a response is received |
| `attempts` | numeric | Attempt index on retry-bearing paths |
| `attempt_index` | numeric | Attempt index on retry events |
| `backoff_ms` | numeric | Retry wait duration in milliseconds |
| `transport_error_class` | string | Transport failure class on retry events without an HTTP response |
| `duration_ms` | numeric | Elapsed time in milliseconds for the span |
| `order_uid` | string | Order UID of the target order |
| `order_uid_count` | numeric | Number of order UIDs included in a cancellation-signing event |
| `quote_id` | numeric | Orderbook quote id returned by a quote or attached to an order submission |
| `owner` | string | Owner address exposed on the request parameters |
| `verifier` | string | Public on-chain verifier address for EIP-1271 verification |
| `version` | debug | COW Shed deployment version on cow-shed signing spans |
| `tx_hash` | string | Broadcast transaction hash on transaction-lifecycle spans |
| `tx_status` | string | Mined terminal status on a receipt span: `success`, `reverted`, or `unknown` |
| `block_number` | numeric | Mined block number on a receipt span, when the provider reports it |
| `gas_used` | numeric | Gas used by the mined transaction on a receipt span, when the provider reports it |
| `scheme` | string | Signing scheme (`eip712`, `eth_sign`, `eip1271`, `pre_sign`) |
| `cache_status` | string | EIP-1271 verification cache state: `hit`, `miss`, `store`, or `skip` |
| `verification_result` | string | EIP-1271 verification result when known: `valid`, `invalid`, or `error` |
| `cancelled` | boolean | Cooperative cancellation event marker |

## Coverage

Tracing spans are emitted by every long-running public async method on
`cow-sdk-orderbook`, `cow-sdk-subgraph`, `cow-sdk-trading`,
`cow-sdk-signing`, `cow-sdk-browser-wallet`, and, behind its opt-in
`cow-shed` facade feature, `cow-sdk-cow-shed`. Each canonical public async
method carries `#[tracing::instrument]` and emits exactly one span per call.
The native Alloy adapter crates participate in the facade `tracing` feature
family and follow the same redaction posture for any adapter diagnostics.
Callers that need cooperative cancellation wrap the returned future through
[`cow_sdk_core::Cancellable::cancel_with`] at the call site; the span is
emitted through the wrapped future without additional instrumentation.

### Transport Layer

When the `tracing` feature is enabled, the native
`cow_sdk_core::ReqwestTransport` and browser
`cow_sdk_transport_wasm::FetchTransport` emit one `info` span named
`transport.dispatch` for each low-level dispatch. Both adapters record
`method`, path-only `endpoint`, `bytes_sent`, and `bytes_received`; the
browser adapter also records `chain = "wasm32"`. The endpoint field never
contains the URL scheme, host, credentials, query string, or fragment.

### `cow-sdk-orderbook`

Every public async method on `OrderbookApi` emits one span. Spans carry
`chain`, `env`, `endpoint`, and `method`; retry-bearing spans populate
`attempts` and `status`, and `quote_id`, `order_uid`, and `owner` are added
where the request or response exposes them. When the `tracing` feature is
enabled, retry decisions also emit events with `attempt_index`, `backoff_ms`,
and either `status` or `transport_error_class`.

- `version`
- `quote`
- `send_order`
- `send_signed_order_cancellations`
- `order`
- `order_multi_env`
- `orders`
- `tx_orders`
- `trades`
- `order_competition_status`
- `native_price`
- `total_surplus`
- `app_data`
- `upload_app_data`
- `solver_competition_by_auction_id`
- `solver_competition_by_tx_hash`
- `latest_solver_competition`

### `cow-sdk-subgraph`

Every top-level public async method on `SubgraphApi` emits one span.
Spans carry `chain`, `endpoint`, and `method`; subgraph does not have a
protocol `env` axis.

- `totals`
- `last_days_volume`
- `last_hours_volume`
- `run_query`

### `cow-sdk-trading`

Every public async method on `Trading` plus the module-level async
helpers emit one span each. Spans carry `chain`, `env`, and `endpoint`;
`order_uid` is added on order-bound helpers. The EIP-1271 order verifier
also wraps its lower-level contract call in a
`trading.verify_eip1271_caller` span carrying `chain_id` and `verifier`.

- `quote_only`
- `quote_results`
- `post_swap_order`
- `post_swap_order_from_quote`
- `post_limit_order`
- `pre_sign_transaction`
- `order`
- `off_chain_cancel_order`
- `on_chain_cancel_order`
- `cow_protocol_allowance`
- `approve_cow_protocol`
- `post_sell_native_currency_order` (module-level)

The transaction receipt-wait helpers emit a separated lifecycle pair rather
than a single method span, so a submission is never conflated with inclusion.
`submit_and_wait_for_receipt` emits a `transaction.submit` span that records
only the broadcast `tx_hash`, followed by a `transaction.receipt` span that
records `tx_status`, `block_number`, and `gas_used` once a provider receipt is
observed. `poll_for_receipt`, which never broadcasts, carries only the
`transaction.receipt` span. These spans record no signer, signature, calldata,
sender, or recipient material, and a `transaction.submit` span never implies
inclusion or execution success
([ADR 0038](adr/0038-transaction-lifecycle-types.md)).

- `transaction.submit` (module-level, `submit_and_wait_for_receipt`)
- `transaction.receipt` (module-level, `submit_and_wait_for_receipt` and `poll_for_receipt`)

### `cow-sdk-contracts`

`verify_eip1271_signature_cached` emits one span named `verify.eip1271`
with target `cow_sdk::verify_eip1271`. The contracts-layer span records
only `verifier`; it does not record `chain_id`, signature bytes, raw
digest content, provider URLs, or response bodies.

The same target emits `debug` events for cache and verification outcomes.
`cache_status` is one of `hit`, `miss`, `store`, or `skip`. Because the
cache records only positive verdicts, a `hit` and a `store` always carry
`valid`; a magic-value mismatch surfaces as `skip` with `invalid`, and a
transient error as `skip` with `error`. `verification_result` is present
when the result is known and is one of `valid`, `invalid`, or `error`.

### `cow-sdk-signing`

Local signing helpers carry `chain`, `scheme`, and `endpoint`. Signing is
chain-bound, not env-bound; the owner is determined by the supplied signer
and is not surfaced as a span field. Cancellation signing also emits a
`debug` event with target `cow_sdk::signing` that records the first
`order_uid` and `order_uid_count` so batch cancellation activity is visible
without logging signatures or private material.

- `sign_order_with_scheme`
- `sign_order_cancellation_with_scheme`
- `sign_order_cancellations_with_scheme`

### `cow-sdk-cow-shed`

`CowShedHooks::sign` is the crate's one signer-mediated async method. It emits a
single `sign` span carrying `chain`, `version`, and `endpoint = "cow_shed.sign"`.
Like local signing, COW Shed signing is chain-bound and the owner is resolved
from the supplied signer and is not surfaced; the span records no signer,
signature, owner, nonce, deadline, or typed-data payload material. The
deterministic building blocks (proxy derivation, EIP-712 digest, calldata
encoding) are pure transforms and carry no spans. This crate is opt-in behind
the facade `cow-shed` feature ([ADR 0049](adr/0049-cow-shed-account-abstraction-proxy.md)).

- `sign`

### `cow-sdk-browser-wallet`

Wallet-mediated chain operations carry `chain` and an explicit `method`
label identifying the operation.

- `BrowserWallet::signer_for_chain`
- `BrowserWallet::switch_chain`
- `BrowserWallet::switch_or_add_chain`

### Native Alloy Adapters

Native Alloy adapter telemetry follows the same redaction posture as the rest
of the SDK. Provider URLs, private-key material, typed-data payload contents,
signature bytes, raw transaction payloads, and response bodies are not trace
fields. Downstream applications that need provider-specific telemetry can add
their own spans around the `Provider`, `SigningProvider`, or
`Signer` calls.

Transaction lifecycle telemetry stays separated. The `cow-sdk-trading`
receipt-wait helpers realize this contract directly through the
`transaction.submit` and `transaction.receipt` spans described above: a
submission span records only the broadcast hash, a receipt-observation span
records mined fields such as status, block number, and gas used only after an
explicit provider receipt lookup, and a `send_transaction` span never implies
inclusion or execution success. Host applications that instrument an adapter's
`Signer`/`Provider` calls directly should keep the same separation.

## Secrets

No traced span or event must ever carry a secret. Concretely:

- The `api_key` field of `ApiContext` and `ApiContextOverride` is
  `Redacted<String>`; its `Debug` implementation emits `[redacted]` so
  accidental `?` formatting cannot leak the value, and no instrumented
  call site records the field regardless.
- `IpfsConfig` read URIs (`uri`, `read_uri`) are stored as `Redacted<String>`
  and follow the same redaction contract, so a configured gateway endpoint
  is never captured in traces.
- Wallet signatures, recovered public keys, and private-key material are
  never logged by the SDK. Downstream instrumentation that wants to record
  a signature should do so explicitly in host code.
- Native Alloy adapter diagnostics redact configured RPC URLs and signing
  secrets before public formatting or telemetry.
- `cow-sdk-wasm` maps transport, app-data, signing, orderbook, subgraph, and
  trading failures into `WasmError` with display-safe messages and redacted
  response bodies before those values cross into JavaScript.
- EIP-1271 verification telemetry records the verifier address and
  low-cardinality cache/result labels only; it never records signature
  bytes, raw digest content, provider URLs, or response bodies.

If a future call site needs to record an identifier that is derived from
secret material, the convention is to hash or prefix-truncate it in the
host application before emitting it through the tracing subscriber.

## Cooperative Cancellation

`cow_sdk_core::Cancellable::cancel_with` emits a `debug` event with target
`cow_sdk::cancel` and `cancelled = true` when a cancellation token wins the
biased poll. The level is intentionally below `warn` because user
interfaces may cancel and replace in-flight requests at high frequency
during normal operation.

## Retry Cooldowns

`cow-sdk-transport-policy` supplies the shared retry cooldown behavior used by
the orderbook and subgraph clients. Both clients honor `Retry-After` on
`429 Too Many Requests` and `503 Service Unavailable` responses when the
transport surfaces response headers through `TransportError::HttpStatus`. The
retry loop accepts both delta-seconds and HTTP-date values and waits for the
larger of the local backoff schedule and the server-provided cooldown before
retrying. The local backoff supports jitter strategies through
`RetryPolicy::with_jitter`, and callers can select an explicit no-jitter
strategy for deterministic tests. The native cooldown contract is exercised by
`crates/orderbook/tests/api_contract.rs::service_unavailable_retry_after_header_delays_retry_for_at_least_server_cooldown`,
the parser boundary is covered by `crates/transport-policy/tests/policy_contract.rs` and `crates/transport-policy/tests/retry_after_contract.rs`,
and the retry-event contract is covered by
`crates/orderbook/tests/request_contract.rs::tracing_contract::execute_with_emits_retry_events_with_status_and_transport_error_fields`.

## Error Classification

`cow_sdk::CowError::class()` returns an `ErrorClass` so telemetry layers
can partition failures into `Validation`, `Transport`, `Remote`, `RateLimited`,
`Signing`, `Cancelled`, and `Internal` buckets without pattern-matching every
nested variant by hand. Retry policies typically only retry the `Transport` and
`Remote` classes; `RateLimited` means the remote throttled the client past the
transport's retry budget, and the remaining classes surface caller-side or
protocol-level conditions that benefit from different recovery paths. The enum
is `#[non_exhaustive]`, so consumer `match` arms keep a wildcard for future
classes.
