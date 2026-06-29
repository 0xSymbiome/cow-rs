---
type: Property
id: alloy
title: "Alloy client invariants"
description: "The `cow-sdk-alloy` umbrella client surface: typed construction, error classification, and cancellation propagation (`AlloyClientError` carries a `Cancelled` variant bridged from `cow_sdk_core::Cancelled`)."
resource: https://github.com/0xSymbiome/cow-rs/blob/main/docs/properties/alloy.md
families: [PROP-AU, PROP-AU-CANCEL]
tags: [property, invariants]
timestamp: 2026-06-29T00:00:00Z
---

# Alloy client invariants

The `cow-sdk-alloy` umbrella client surface: typed construction, error classification, and cancellation propagation (`AlloyClientError` carries a `Cancelled` variant bridged from `cow_sdk_core::Cancelled`). Part of the [Properties Registry](index.md): 9 invariant(s), 9 covered.

## Composed client & signer handle

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-AU-001` | `cow-sdk-alloy` | `AlloyClient` exposes an SDK-owned composed native API while upstream Alloy provider, wallet, transport, and local signer values stay private and redacted. Governed by [ADR 0035](../adr/0035-alloy-provider-adapter.md). | Public API | Yes | `crates/sdk/tests/error_redaction_contract.rs::alloy_adapter_errors_redact_secret_bearing_payloads`, `crates/alloy/src/lib.rs`, `crates/alloy/src/client.rs`, `crates/alloy/src/handle.rs`, `crates/alloy/README.md`, `docs/audit/alloy-adapters-audit.md` | 2026-05-26 |
| `PROP-AU-002` | `cow-sdk-alloy` | `AlloyClient` implements every method on `Provider` and implements `SigningProvider` by returning an owned `AlloyClientSignerHandle`. Governed by [ADR 0035](../adr/0035-alloy-provider-adapter.md). | Contract | Yes | `crates/alloy/tests/provider_contract.rs`, `crates/alloy/tests/signing_provider_contract.rs::create_signer_returns_owned_handle`, `tests/alloy_umbrella_composition.rs::alloy_client_satisfies_trading_sdk_boundaries` | 2026-05-26 |
| `PROP-AU-003` | `cow-sdk-alloy` | `AlloyClientSignerHandle` remains usable after the parent `AlloyClient` is dropped, while the client does not implement `Signer` and the handle does not implement `Provider`. Governed by [ADR 0035](../adr/0035-alloy-provider-adapter.md). | Contract | Yes | `crates/alloy/tests/handle_survives_drop.rs::signer_handle_remains_usable_after_parent_client_drop`, `crates/alloy/tests/compile_fail.rs::public_surface_rejects_signer_on_client_and_provider_on_handle`, `crates/alloy/tests/trybuild/no_signer_on_client.rs`, `crates/alloy/tests/trybuild/no_provider_on_handle.rs` | 2026-05-27 |
| `PROP-AU-004` | `cow-sdk-alloy` | The umbrella signer handle preserves typed-data primary types, normalizes ECDSA signatures, submits transactions through the Alloy wallet-filler provider, returns `TransactionBroadcast` with the broadcast hash read through `*pending.tx_hash()` without waiting for confirmation, estimates gas through the provider, and delegates rich receipt lookup to the provider adapter. Governed by [ADR 0035](../adr/0035-alloy-provider-adapter.md) and [ADR 0038](../adr/0038-transaction-lifecycle-types.md). | Contract | Yes | `crates/alloy/tests/eip712_reference_vectors.rs::sign_typed_data_payload_preserves_primary_type`, `crates/alloy/tests/send_transaction_does_not_wait_for_confirmation.rs::send_transaction_does_not_dispatch_get_transaction_receipt`, `crates/alloy/tests/provider_contract.rs::get_transaction_receipt_populates_rich_fields_from_alloy_receipt`, `tests/alloy_umbrella_composition.rs::alloy_client_satisfies_trading_sdk_boundaries`, `tests/transaction_lifecycle_cross_adapter_invariant.rs`, `examples/native/scenarios/alloy_trading_full_flow.rs` | 2026-06-15 |

## Read & logs

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-AU-007` | `cow-sdk-alloy` | `AlloyClient::read_contract` consumes the provider leaf's `execute_read_contract` through the doc-hidden inter-crate seam, lifts `ProviderError` into `AlloyClientError` through the existing `From` impl, and surfaces `AlloyClientError::Validation` end-to-end for malformed inputs (invalid ABI type, wrong argument count, type mismatches, length mismatches, null arguments, object-for-address, overloaded functions) without panicking. Every documented scalar (`uint256`, `int256`, `bool`, `string`, `bytes`, `bytes32`, `address`, `uint8`, `uint64`), every compound shape (dynamic arrays, fixed arrays, multi-output tuples, address arrays), and every documented argument-shape variant (JSON array, JSON object with named params, single scalar form) flows through this single path. Governed by [ADR 0035](../adr/0035-alloy-provider-adapter.md). | Contract | Yes | `crates/alloy/tests/read_contract_contract.rs`, `crates/alloy/src/client.rs`, `crates/alloy-provider/src/read_contract.rs`, `crates/alloy-provider/src/lib.rs`, `tests/alloy_read_contract_parity_invariant.rs` | 2026-05-26 |
| `PROP-AU-008` | `cow-sdk-alloy` | `AlloyClient` implements `LogProvider`, issuing a single bounded `eth_getLogs` over the composed Alloy provider and reusing the provider leaf's `LogQuery` → filter and Alloy-log → `RawLog` conversions through the doc-hidden inter-crate seam, so an event-log fetch needs no second provider for the same RPC endpoint. Governed by [ADR 0035](../adr/0035-alloy-provider-adapter.md) and [ADR 0057](../adr/0057-log-provider-capability-trait.md). | Contract | Yes | `crates/alloy/tests/log_provider_contract.rs::alloy_client_implements_log_provider_and_returns_typed_error_on_unreachable_rpc`, `crates/alloy/src/client.rs`, `crates/alloy-provider/tests/seam_contract.rs::seam_exposes_log_conversions_for_the_umbrella`, `docs/audit/alloy-adapters-audit.md` | 2026-06-03 |

## Construction & errors

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-AU-005` | `cow-sdk-alloy` | `ClientBuilder` rejects every documented malformed input (invalid URL, invalid prefixed and bare-hex private keys, all-zero key bytes, curve-order key) without echoing the offending value, the chain-mismatch detection in `build_checked` carries both configured and remote chain ids in its display, the `From<AlloyClientError>` lift propagates transparently into the `AlloyClient` variant, and the fully-typed builder `Debug` impl redacts key bytes while surfacing the chain id. Governed by [ADR 0035](../adr/0035-alloy-provider-adapter.md). | Contract | Yes | `crates/alloy/tests/builder_contract.rs` | 2026-05-14 |
| `PROP-AU-006` | `cow-sdk-alloy` | The documented `From<ProviderError>` lift maps every provider-error variant (validation, transport, remote, cancelled, internal) into the composed adapter's typed surface without leaking detail. Governed by [ADR 0035](../adr/0035-alloy-provider-adapter.md). | Contract | Yes | `crates/alloy/tests/error_contract.rs`, `crates/alloy/tests/redaction_contract.rs` | 2026-06-15 |

## Cancellation propagation

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-AU-CANCEL-001` | `cow-sdk-alloy` | `AlloyClientError` carries a `Cancelled` variant and an `impl From<cow_sdk_core::Cancelled>` bridge so `Cancellable::cancel_with(...).await?` propagates cancellation through the composed adapter error type. Governed by [ADR 0035](../adr/0035-alloy-provider-adapter.md). | Contract | Yes | `crates/alloy/src/error.rs`, `crates/alloy/tests/cancellation_contract.rs::cancel_with_propagates_cancelled_through_question_mark` | 2026-05-06 |
