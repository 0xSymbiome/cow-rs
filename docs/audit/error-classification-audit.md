# Error Classification Audit

Status: Current
Last reviewed: 2026-06-17
Owning surface: the `class()`, `is_retryable()`, and `backoff_hint()` accessors on the `cow-sdk` error family and the shared `cow_sdk_core::ErrorClass`
Refresh trigger: a new `ErrorClass` bucket; a new error type aggregated by `cow_sdk::CowError`; a change to any type's `class()` mapping; a change to the `is_retryable()` / `backoff_hint()` mapping or the retained `Retry-After` capture; or a new error variant whose class or retry verdict differs from its type's existing default arm
Related docs:
- [ADR 0060](../adr/0060-uniform-error-classification.md)
- [ADR 0053](../adr/0053-typed-signer-rejection-classification.md)
- [ADR 0017](../adr/0017-typed-orderbook-rejection-parser.md)
- [ADR 0041](../adr/0041-transport-policy-l3-layering.md)
- [ADR 0025](../adr/0025-workspace-url-redaction-convention.md)

## Scope

This audit covers:

- the shared `cow_sdk_core::ErrorClass` partition and its re-export from the
  `cow-sdk` facade
- the `const fn class(&self) -> ErrorClass` accessor on each facade-family
  error type (`CoreError`, `AppDataError`, `SigningError`, `ContractsError`,
  `OrderbookError`, `TradingError`, and — behind the
  off-by-default `subgraph` feature — `SubgraphError`)
- the `OrderbookError::is_retryable()` and `OrderbookError::backoff_hint()`
  retry-decision accessors, the `Retry-After` value retained on
  `OrderbookApiError`, and the facade and trading delegation of both accessors
- the facade delegation performed by `CowError::class()`,
  `CowError::is_retryable()`, and `CowError::backoff_hint()`

It does not cover the native Alloy adapter error classes
(`ProviderErrorClass`, `SignerErrorClass`, `AlloyClientErrorClass`), which keep
their own per-type enums under the [ADR 0053](../adr/0053-typed-signer-rejection-classification.md)
convention because their taxonomies differ from the facade family.

It also does not cover `cow_sdk_trading::WaitError`, the receipt-wait outcome
type. `WaitError` is generic over the caller's signer and provider error types
(runtime neutrality, [ADR 0024](../adr/0024-asyncprovider-asyncsigningprovider-capability-split.md)),
so it stays outside the `ErrorClass` family and is not a `CowError` variant. Its
on-chain verdict is the purpose-built `WaitError::reverted()` accessor — which
distinguishes a mined revert from the transient broadcast, lookup, timeout, and
cancellation variants — rather than a `class()` mapping.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Shared partition | `ErrorClass` lives in `cow-sdk-core`, is `#[non_exhaustive]`, and is re-exported as `cow_sdk::ErrorClass` | Conforms |
| Label rendering | `ErrorClass` exposes `as_str()` and a `Display` impl with stable lowercase labels, mirroring `TransportErrorClass` and the adapter class enums | Conforms |
| Per-type accessors | Every facade-family error type exposes `const fn class(&self) -> ErrorClass` | Conforms |
| Facade delegation | `CowError::class()` delegates to each leaf accessor and holds no classification logic of its own | Conforms |
| Composite granularity | `TradingError::class()` delegates to the wrapped error, so a wrapped 429 orderbook rejection stays `RateLimited` | Conforms |
| Retry verdict | `OrderbookError::is_retryable()` keys off the retained HTTP status (the retryable set) for structured responses and the transient transport-class mapping for transport failures | Conforms |
| Backoff hint | `OrderbookError::backoff_hint()` surfaces the `Retry-After` parsed from the failing response, and is `None` for transport failures and headerless responses | Conforms |
| Retry delegation | `TradingError` and `CowError` delegate `is_retryable()` / `backoff_hint()` to the wrapped orderbook error and return `false` / `None` for non-orderbook variants | Conforms |
| Subgraph (feature-gated) | With the `subgraph` feature enabled, `SubgraphError::class()` joins the family and `CowError::Subgraph` delegates to it; off by default, so the default family is unchanged | Conforms |
| Redaction posture | Classification and retry accessors read only typed discriminants and a parsed delay; they render no credential-bearing content (ADR 0025) | Conforms |
| `UserRejection` trait | `cow_sdk_core::UserRejection::user_rejection_code` exposes only the EIP-1193 numeric code, never an implementer-controlled string | Conforms |
| `Some(code)` contract | A typed rejection variant returns the carried EIP-1193 code; every non-rejection variant returns `None` | Conforms |
| Alloy signer impls | Every variant of `cow_sdk_alloy_signer::SignerError` and `AlloyClientError` returns `None`; local-key signing and the umbrella adapter never produce EIP-1193 rejections | Conforms |
| `signer_error` helper | Routes through the trait, emitting `SigningError::SignerRejection` only when the trait returns `Some(_)`, otherwise the redacted `SigningError::Signer` | Conforms |
| `SignerRejection` display | `Display` renders `user rejected {label} ({code})`; fields are a static operation label plus the numeric code only, with no wallet-supplied text | Conforms |

## Current Contract

### Shared partition

`ErrorClass` (`Validation | Transport | Remote | RateLimited | Signing |
Cancelled | Internal`) is defined in `crates/core/src/errors.rs` and
re-exported from the facade, so `cow_sdk::ErrorClass` resolves to the same type
a leaf crate returns. Retry layers treat `Transport` and `Remote` as retryable;
`RateLimited` is reached only once the transport retry budget has honored
`Retry-After`. Telemetry and logging layers render the class through
`ErrorClass::as_str()` — a stable lowercase label (`validation`, `transport`,
`remote`, `rate-limited`, `signing`, `cancelled`, `internal`) — or its `Display`
impl, mirroring `TransportErrorClass` and the adapter class enums; `Debug` is
not a stability contract.

### Per-type accessors and delegation

Each facade-family error type owns its mapping in its own crate. Composite
types delegate: `OrderbookError::class()` resolves a wrapped `CoreError`
through `CoreError::class()`; `TradingError::class()` resolves wrapped
`Core`/`AppData`/`Orderbook`/`Signing`/`Contracts` errors through their
accessors. `CowError::class()` is a pure delegation over the six leaf
accessors, so the class is identical whether a caller holds the facade error or
a bare leaf error. `ContractsError::class()` partitions its variants by meaning
rather than to a single bucket: caller-supplied shape and range failures map to
`Validation`, serialization/ABI/decode invariants map to `Internal` (matching
`CoreError`), and the EIP-1271, provider, and ECDSA-recovery operations map to
`Signing`. When the off-by-default `subgraph` feature is enabled,
`SubgraphError::class()` is the seventh accessor and the feature-gated
`CowError::Subgraph` variant delegates to it the same way
([ADR 0003](../adr/0003-separate-read-only-subgraph-crate.md)); the
retry-decision accessors stay orderbook-and-trading-scoped, so a subgraph error
reports as non-retryable with no backoff hint.

### Retry-decision accessors

`OrderbookError::is_retryable()` returns the same verdict the SDK transport
retry loop applies: a structured non-2xx response keys off the retained HTTP
status through `cow_sdk_core::transport::policy::is_retryable_status`, and a transport
failure keys off its `TransportErrorClass` through
`RetryPolicy::should_retry_network`. It keys off the status rather than
`class()` because the coarse partition collapses every non-429 remote response
into `Remote`, so the status-precise accessor separates a retryable `503` from
a non-retryable `400`. `OrderbookError::backoff_hint()` returns the
`Retry-After` parsed from the failing response (delta-seconds or HTTP-date,
resolved against the wasm-safe wall clock at error construction), retained on
`OrderbookApiError` and exposed through both the `Rejected` and `Api` promotion
paths; it is `None` for transport failures and headerless responses.
`TradingError` and `CowError` delegate both accessors to the wrapped orderbook
error and return `false` / `None` for every non-orderbook variant. The
TypeScript-callable `cow-sdk-wasm` surface projects the same verdict to
JavaScript: the `WasmError` `orderbook` variant carries a `retryable` boolean
and an optional `retryAfterMs` populated from these accessors.

### Signer Error Classification

`cow_sdk_core::UserRejection::user_rejection_code` returns `Option<i32>` with a
`None` default, so a signer that never represents EIP-1193 rejections adopts the
trait with a one-line `impl`. The trait deliberately exposes only the numeric
code: implementers must not return strings, free-text labels, or any
wallet-supplied content. Every variant of `cow_sdk_alloy_signer::SignerError`
and `AlloyClientError` returns `None`, because local-key signing and the
umbrella adapter never route wallet prompts.

The `cow_sdk_signing::order_signing::signer_error` helper consumes the upstream
error by value, calls `user_rejection_code`, and returns either
`SigningError::SignerRejection { label, code }` when the trait returned
`Some(code)` — `label` being the static call-site operation label
(`"typed-data signature"`, `"message signature"`, or `"signing request"`) — or
`SigningError::Signer { operation, message: Redacted<String> }` otherwise,
carrying the upstream redacted `Display` verbatim so the workspace redaction
convention ([ADR 0025](../adr/0025-workspace-url-redaction-convention.md)) stays
intact. `SignerRejection`'s `Display` renders `user rejected {label} ({code})`,
so a downstream JavaScript or TypeScript console can scan for the parenthesised
EIP-1193 code and use the static label as a stable `errorText` substring without
exposing any wallet-controlled message text.

## Evidence

Primary implementation points:

- `crates/core/src/errors.rs` (`ErrorClass`, `CoreError::class`)
- `crates/app-data/src/errors.rs`, `crates/signing/src/errors.rs`,
  `crates/contracts/src/errors.rs`, `crates/orderbook/src/error.rs`
  (`class`, `is_retryable`, `backoff_hint`),
  `crates/trading/src/error.rs`,
  `crates/subgraph/src/error.rs` (`class`, behind the `subgraph` feature)
- `crates/orderbook/src/request.rs` (`OrderbookApiError` `Retry-After` capture)
- `crates/core/src/transport/policy/retry_after.rs` (`retry_after_from_headers`)
- `crates/sdk/src/lib.rs` (`CowError` `class` / `is_retryable` / `backoff_hint`
  delegation)
- `crates/core/src/traits/signer.rs` (`UserRejection` trait + courtesy impls for
  `String`, `&str`, `core::convert::Infallible`)
- `crates/signing/src/order_signing.rs` (`signer_error` routing helper plus the
  `signer_operation_label` mapping)
- `crates/signing/src/cancellation.rs` (bound propagation on the cancellation
  helpers)
- `crates/alloy-signer/src/error.rs`, `crates/alloy/src/error.rs` (no-op
  classification)

Primary regression coverage:

- `crates/sdk/tests/error_class_contract.rs::error_class_partitions_every_bucket`
- `crates/sdk/tests/error_class_contract.rs::error_class_delegates_through_trading_and_facade`
- `crates/sdk/tests/error_class_contract.rs::exhausted_retry_429_classifies_as_rate_limited`
- `crates/sdk/tests/error_class_contract.rs::non_429_remote_responses_stay_remote`
- `crates/sdk/tests/error_class_contract.rs::is_retryable_delegates_through_trading_and_facade`
- `crates/sdk/tests/error_class_contract.rs::backoff_hint_delegates_through_trading_and_facade`
- `crates/sdk/tests/error_class_contract.rs::subgraph::subgraph_error_class_partitions_every_bucket` (with `--features subgraph`)
- `crates/sdk/tests/error_class_contract.rs::subgraph::subgraph_error_class_delegates_through_facade` (with `--features subgraph`)
- `crates/contracts/tests/error_contract.rs::class_partitions_validation_internal_and_signing`
- `crates/orderbook/src/error.rs` retry-classification unit tests
- `crates/core/src/transport/policy/retry_after.rs` `Retry-After` header tests
- `crates/wasm/tests/wasm_error_abi_contract.rs::orderbook_variant_carries_retry_hints`
- `crates/alloy-signer/tests/signer_error_trait_contract.rs`
- `crates/alloy/tests/signer_error_trait_contract.rs`
- `crates/signing/src/order_signing.rs::signer_error_tests` (helper-routing unit tests)
- `tests/signer_rejection_propagation_invariant.rs` (workspace end-to-end
  propagation through `sign_order`)
- `crates/sdk/tests/error_redaction_contract.rs` (redaction sweep including
  `SignerRejection`)

Validation surface:

```text
cargo test -p cow-sdk --test error_class_contract --all-features
cargo test -p cow-sdk-orderbook --lib
cargo test -p cow-sdk-core --features transport-policy --lib
cargo check-enum-policy
cargo test -p cow-sdk-alloy-signer --test signer_error_trait_contract
cargo test -p cow-sdk-alloy --test signer_error_trait_contract
cargo test -p cow-sdk-signing --lib signer_error_tests
cargo test -p cow-rs-workspace-tests --test signer_rejection_propagation_invariant
cargo test -p cow-sdk --test error_redaction_contract
```
