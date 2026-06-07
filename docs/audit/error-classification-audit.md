# Error Classification Audit

Status: Current
Last reviewed: 2026-06-07
Owning surface: the `class()`, `is_retryable()`, and `backoff_hint()` accessors on the `cow-sdk` error family and the shared `cow_sdk_core::ErrorClass`
Refresh trigger: a new `ErrorClass` bucket; a new error type aggregated by `cow_sdk::CowError`; a change to any type's `class()` mapping; a change to the `is_retryable()` / `backoff_hint()` mapping or the retained `Retry-After` capture; or a new error variant whose class or retry verdict differs from its type's existing default arm
Related docs:
- [ADR 0060](../adr/0060-uniform-error-classification.md)
- [ADR 0053](../adr/0053-typed-signer-rejection-classification.md)
- [ADR 0017](../adr/0017-typed-orderbook-rejection-parser.md)
- [ADR 0041](../adr/0041-transport-policy-l3-layering.md)

## Scope

This audit covers:

- the shared `cow_sdk_core::ErrorClass` partition and its re-export from the
  `cow-sdk` facade
- the `const fn class(&self) -> ErrorClass` accessor on each facade-family
  error type (`CoreError`, `AppDataError`, `SigningError`, `ContractsError`,
  `OrderbookError`, `TradingError`, `BrowserWalletError`, and — behind the
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

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Shared partition | `ErrorClass` lives in `cow-sdk-core`, is `#[non_exhaustive]`, and is re-exported as `cow_sdk::ErrorClass` | Conforms |
| Per-type accessors | Every facade-family error type exposes `const fn class(&self) -> ErrorClass` | Conforms |
| Facade delegation | `CowError::class()` delegates to each leaf accessor and holds no classification logic of its own | Conforms |
| Composite granularity | `TradingError::class()` delegates to the wrapped error, so a wrapped 429 orderbook rejection stays `RateLimited` | Conforms |
| Retry verdict | `OrderbookError::is_retryable()` keys off the retained HTTP status (the retryable set) for structured responses and the transient transport-class mapping for transport failures | Conforms |
| Backoff hint | `OrderbookError::backoff_hint()` surfaces the `Retry-After` parsed from the failing response, and is `None` for transport failures and headerless responses | Conforms |
| Retry delegation | `TradingError` and `CowError` delegate `is_retryable()` / `backoff_hint()` to the wrapped orderbook error and return `false` / `None` for non-orderbook variants | Conforms |
| Subgraph (feature-gated) | With the `subgraph` feature enabled, `SubgraphError::class()` joins the family and `CowError::Subgraph` delegates to it; off by default, so the default family is unchanged | Conforms |
| Redaction posture | Classification and retry accessors read only typed discriminants and a parsed delay; they render no credential-bearing content (ADR 0025) | Conforms |

## Current Contract

### Shared partition

`ErrorClass` (`Validation | Transport | Remote | RateLimited | Signing |
Cancelled | Internal`) is defined in `crates/core/src/errors.rs` and
re-exported from the facade, so `cow_sdk::ErrorClass` resolves to the same type
a leaf crate returns. Retry layers treat `Transport` and `Remote` as retryable;
`RateLimited` is reached only once the transport retry budget has honored
`Retry-After`.

### Per-type accessors and delegation

Each facade-family error type owns its mapping in its own crate. Composite
types delegate: `OrderbookError::class()` resolves a wrapped `CoreError`
through `CoreError::class()`; `TradingError::class()` resolves wrapped
`Core`/`AppData`/`Orderbook`/`Signing`/`Contracts` errors through their
accessors. `CowError::class()` is a pure delegation over the seven leaf
accessors, so the class is identical whether a caller holds the facade error or
a bare leaf error. `ContractsError::class()` partitions its variants by meaning
rather than to a single bucket: caller-supplied shape and range failures map to
`Validation`, serialization/ABI/decode invariants map to `Internal` (matching
`CoreError`), and the EIP-1271, provider, and ECDSA-recovery operations map to
`Signing`. When the off-by-default `subgraph` feature is enabled,
`SubgraphError::class()` is the eighth accessor and the feature-gated
`CowError::Subgraph` variant delegates to it the same way
([ADR 0003](../adr/0003-separate-read-only-subgraph-crate.md)); the
retry-decision accessors stay orderbook-and-trading-scoped, so a subgraph error
reports as non-retryable with no backoff hint.

### Retry-decision accessors

`OrderbookError::is_retryable()` returns the same verdict the SDK transport
retry loop applies: a structured non-2xx response keys off the retained HTTP
status through `cow_sdk_transport_policy::is_retryable_status`, and a transport
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

## Evidence

Primary implementation points:

- `crates/core/src/errors.rs` (`ErrorClass`, `CoreError::class`)
- `crates/app-data/src/errors.rs`, `crates/signing/src/errors.rs`,
  `crates/contracts/src/errors.rs`, `crates/orderbook/src/error.rs`
  (`class`, `is_retryable`, `backoff_hint`),
  `crates/browser-wallet/src/error.rs`, `crates/trading/src/error.rs`,
  `crates/subgraph/src/error.rs` (`class`, behind the `subgraph` feature)
- `crates/orderbook/src/request.rs` (`OrderbookApiError` `Retry-After` capture)
- `crates/transport-policy/src/retry_after.rs` (`retry_after_from_headers`)
- `crates/sdk/src/lib.rs` (`CowError` `class` / `is_retryable` / `backoff_hint`
  delegation)

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
- `crates/transport-policy/src/retry_after.rs` `Retry-After` header tests
- `crates/wasm/tests/wasm_error_abi_contract.rs::orderbook_variant_carries_retry_hints`

Validation surface:

```text
cargo test -p cow-sdk --test error_class_contract --all-features
cargo test -p cow-sdk-orderbook --lib
cargo test -p cow-sdk-transport-policy --lib
cargo check-enum-policy
```
