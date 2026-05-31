# Error Classification Audit

Status: Current
Last reviewed: 2026-05-31
Owning surface: the `class()` accessors on the `cow-sdk` error family and the shared `cow_sdk_core::ErrorClass`
Refresh trigger: a new `ErrorClass` bucket; a new error type aggregated by `cow_sdk::SdkError`; a change to any type's `class()` mapping; or a new error variant whose class differs from its type's existing default arm
Related docs:
- [ADR 0060](../adr/0060-uniform-error-classification.md)
- [ADR 0053](../adr/0053-typed-signer-rejection-classification.md)
- [ADR 0017](../adr/0017-typed-orderbook-rejection-parser.md)

## Scope

This audit covers:

- the shared `cow_sdk_core::ErrorClass` partition and its re-export from the
  `cow-sdk` facade
- the `const fn class(&self) -> ErrorClass` accessor on each facade-family
  error type (`CoreError`, `AppDataError`, `SigningError`, `ContractsError`,
  `OrderbookError`, `TradingError`, `BrowserWalletError`)
- the facade delegation performed by `SdkError::class()`

It does not cover the native Alloy adapter error classes
(`ProviderErrorClass`, `SignerErrorClass`, `AlloyClientErrorClass`), which keep
their own per-type enums under the [ADR 0053](../adr/0053-typed-signer-rejection-classification.md)
convention because their taxonomies differ from the facade family.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Shared partition | `ErrorClass` lives in `cow-sdk-core`, is `#[non_exhaustive]`, and is re-exported as `cow_sdk::ErrorClass` | Conforms |
| Per-type accessors | Every facade-family error type exposes `const fn class(&self) -> ErrorClass` | Conforms |
| Facade delegation | `SdkError::class()` delegates to each leaf accessor and holds no classification logic of its own | Conforms |
| Composite granularity | `TradingError::class()` delegates to the wrapped error, so a wrapped 429 orderbook rejection stays `RateLimited` | Conforms |
| Redaction posture | Classification reads only typed discriminants; it renders no credential-bearing content (ADR 0025) | Conforms |

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
accessors. `SdkError::class()` is a pure delegation over the seven leaf
accessors, so the class is identical whether a caller holds the facade error or
a bare leaf error.

## Evidence

Primary implementation points:

- `crates/core/src/errors.rs` (`ErrorClass`, `CoreError::class`)
- `crates/app-data/src/errors.rs`, `crates/signing/src/errors.rs`,
  `crates/contracts/src/errors.rs`, `crates/orderbook/src/error.rs`,
  `crates/browser-wallet/src/error.rs`, `crates/trading/src/error.rs`
- `crates/sdk/src/lib.rs` (`SdkError::class` delegation)

Primary regression coverage:

- `crates/sdk/tests/error_class_contract.rs::error_class_partitions_every_bucket`
- `crates/sdk/tests/error_class_contract.rs::error_class_delegates_through_trading_and_facade`
- `crates/sdk/tests/error_class_contract.rs::exhausted_retry_429_classifies_as_rate_limited`
- `crates/sdk/tests/error_class_contract.rs::non_429_remote_responses_stay_remote`

Validation surface:

```text
cargo test -p cow-sdk --test error_class_contract --all-features
cargo check-enum-policy
```
