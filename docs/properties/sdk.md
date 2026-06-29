---
type: Property
id: sdk
title: "SDK facade invariants"
description: "The curated `cow-sdk` facade: exported-symbol snapshot stability and the public re-export surface."
resource: https://github.com/0xSymbiome/cow-rs/blob/main/docs/properties/sdk.md
families: [PROP-SDK]
tags: [property, invariants]
timestamp: 2026-06-29T00:00:00Z
---

# SDK facade invariants

The curated `cow-sdk` facade: exported-symbol snapshot stability and the public re-export surface. Part of the [Properties Registry](index.md): 4 invariant(s), 4 covered.

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-SDK-001` | `cow-sdk` | The facade remains curated and feature-gated, without widening the default surface beyond leaf-crate ownership. | Public API | Yes | `crates/sdk/tests/public_api.rs`, `crates/sdk/tests/public_api_default_features_only.rs`, `crates/sdk/tests/public_api_with_all_features.rs` | 2026-05-31 |
| `PROP-SDK-002` | `cow-sdk` | `CowError::class()` maps an orderbook response that signalled HTTP 429 after the transport retry budget was exhausted to `ErrorClass::RateLimited` on both the recognised-rejection (`Rejected`) and unparsed-envelope (`Api`) paths, distinct from the generic `ErrorClass::Remote` used for other structured responses; the transport layer already retries 429s with `Retry-After` honoring, so the class is an exhausted-retry telemetry signal rather than a control-flow hook. | Contract | Yes | `crates/sdk/tests/error_class_contract.rs::exhausted_retry_429_classifies_as_rate_limited`, `crates/sdk/tests/error_class_contract.rs::non_429_remote_responses_stay_remote`, `crates/sdk/src/lib.rs` | 2026-05-31 |
| `PROP-SDK-003` | `cow-sdk` | Every public error type the facade aggregates exposes a `class() -> ErrorClass` accessor (`CoreError`, `AppDataError`, `SigningError`, `ContractsError`, `OrderbookError`, `TradingError`, and `SubgraphError` behind the off-by-default `subgraph` feature); `CowError::class()` delegates to them and composite errors delegate to the wrapped error so wrapped granularity (a wrapped 429 staying `ErrorClass::RateLimited`) is preserved. `ErrorClass` is defined in `cow-sdk-core` and re-exported from `cow-sdk`. Governed by [ADR 0060](../adr/0060-uniform-error-classification.md). | Contract | Yes | `crates/sdk/tests/error_class_contract.rs::error_class_partitions_every_bucket`, `crates/sdk/tests/error_class_contract.rs::error_class_delegates_through_trading_and_facade`, `crates/sdk/tests/error_class_contract.rs::subgraph::subgraph_error_class_partitions_every_bucket` (with `--features subgraph`), `crates/contracts/tests/error_contract.rs::class_partitions_validation_internal_and_signing`, `crates/core/src/errors.rs`, `crates/contracts/src/errors.rs`, `crates/subgraph/src/error.rs`, `crates/sdk/src/lib.rs` | 2026-06-07 |
| `PROP-SDK-004` | `cow-sdk` | `OrderbookError` exposes `is_retryable() -> bool` and `backoff_hint() -> Option<Duration>`; `is_retryable` keys off the retained HTTP status (the `PROP-TPP-008` retryable set) for structured responses and the transient `TransportErrorClass` mapping for transport failures, while `backoff_hint` surfaces the `Retry-After` parsed from the failing response (delta-seconds or HTTP-date) when present. `TradingError` and `CowError` delegate both accessors to the wrapped orderbook error and return `false` / `None` for every non-orderbook variant. Governed by [ADR 0060](../adr/0060-uniform-error-classification.md). | Contract | Yes | `crates/orderbook/src/error.rs` (retry-classification unit tests), `crates/sdk/tests/error_class_contract.rs::is_retryable_delegates_through_trading_and_facade`, `crates/sdk/tests/error_class_contract.rs::backoff_hint_delegates_through_trading_and_facade`, `crates/core/src/transport/policy/retry_after.rs` | 2026-06-06 |
