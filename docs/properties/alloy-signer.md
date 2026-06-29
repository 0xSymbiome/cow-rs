---
type: Property
id: alloy-signer
title: "Alloy signer invariants"
description: "The native `cow-sdk-alloy-signer` boundary: EIP-712/EIP-191 signing and signature recovery, plus cancellation propagation (`SignerError` carries a `Cancelled` variant bridged from `cow_sdk_core::Cancelled`)."
resource: https://github.com/0xSymbiome/cow-rs/blob/main/docs/properties/alloy-signer.md
families: [PROP-AS, PROP-AS-CANCEL]
tags: [property, invariants]
timestamp: 2026-06-29T00:00:00Z
---

# Alloy signer invariants

The native `cow-sdk-alloy-signer` boundary: EIP-712/EIP-191 signing and signature recovery, plus cancellation propagation (`SignerError` carries a `Cancelled` variant bridged from `cow_sdk_core::Cancelled`). Part of the [Properties Registry](index.md): 9 invariant(s), 8 covered.

## Signer surface & capability boundary

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-AS-001` | `cow-sdk-alloy-signer` | `LocalAlloySigner` exposes an SDK-owned documented API while upstream Alloy private-key signer values stay internal and redacted. The crate also exposes a `#[doc(hidden)] pub mod __seam` module re-exporting the EIP-712 typed-data conversion helper (`cow_typed_data_payload_to_alloy`) and the shared signature normalizer (`alloy_signature_to_hex`) so sibling adapter crates reuse the reviewed implementation without copying it; the seam is not part of the documented consumer API. Governed by [ADR 0035](../adr/0035-alloy-provider-adapter.md). | Public API | Partial | `crates/alloy-signer/src/lib.rs`, `crates/alloy-signer/src/signer.rs`, `crates/alloy-signer/src/conversion.rs`, `crates/alloy-signer/README.md`, `docs/audit/alloy-adapters-audit.md` | 2026-06-20 |
| `PROP-AS-002` | `cow-sdk-alloy-signer` | `LocalAlloySigner` implements `cow_sdk_core::Signer` for address, EIP-191 message, and canonical EIP-712 typed-data payload signing — typed-data signing is payload-only, with no field-based fallback — while provider-backed transaction methods return `ProviderRequired`. Governed by [ADR 0035](../adr/0035-alloy-provider-adapter.md) and [ADR 0068](../adr/0068-payload-only-typed-data-signing.md). | Contract | Yes | `crates/alloy-signer/src/signer.rs`, `crates/alloy-signer/tests/signer_contract.rs`, `crates/alloy-signer/tests/eip191_reference_vectors.rs`, `crates/alloy-signer/tests/eip712_reference_vectors.rs` | 2026-06-11 |
| `PROP-AS-003` | `cow-sdk-alloy-signer` | `LocalAlloySigner` does not implement `Provider` or `SigningProvider`; compile-fail tests enforce the capability boundary. Governed by [ADR 0035](../adr/0035-alloy-provider-adapter.md). | Contract | Yes | `crates/alloy-signer/tests/compile_fail.rs`, `crates/alloy-signer/tests/trybuild/no_provider.rs`, `crates/alloy-signer/tests/trybuild/no_signing_provider.rs` | 2026-06-11 |

## Signing & recovery

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-AS-005` | `cow-sdk-alloy-signer` | `sign_typed_data_payload` preserves the caller's primary type: the digest is computed for the primary-type name carried on the `TypedDataPayload` (for example `Order`), never a placeholder, and the resulting signature recovers against reference vectors for that type. Governed by [ADR 0035](../adr/0035-alloy-provider-adapter.md) and [ADR 0068](../adr/0068-payload-only-typed-data-signing.md). | Contract | Yes | `crates/alloy-signer/src/conversion.rs`, `crates/alloy-signer/tests/signer_contract.rs::sign_typed_data_payload_preserves_order_primary_type`, `crates/alloy/tests/eip712_reference_vectors.rs` | 2026-06-11 |
| `PROP-AS-006` | `cow-sdk-alloy-signer` | EIP-191 and EIP-712 signatures are normalized through `cow-sdk-contracts`, recover to the local signer, and stay covered by committed reference vectors plus property-based recovery checks. Governed by [ADR 0035](../adr/0035-alloy-provider-adapter.md). | Property | Yes | `crates/alloy-signer/src/conversion.rs`, `crates/alloy-signer/tests/eip191_reference_vectors.rs`, `crates/alloy-signer/tests/eip712_reference_vectors.rs`, `crates/alloy-signer/tests/proptests.rs`, `docs/audit/ecdsa-signature-normalization-audit.md` | 2026-06-20 |
| `PROP-AS-008` | `cow-sdk-alloy-signer` | `cow_typed_data_payload_to_alloy` resolves struct-typed fields declared in the type map, directly or as an array (for example `Call[]`), so nested multi-type EIP-712 payloads convert and produce a signing digest byte-identical to the macro-emitted `SolStruct` envelope, while a field referencing a struct that is not declared in the type map stays fail-closed. Governed by [ADR 0035](../adr/0035-alloy-provider-adapter.md). | Contract | Yes | `crates/alloy-signer/src/conversion.rs`, `crates/alloy-signer/src/conversion.rs::tests::nested_struct_payload_matches_macro_digest`, `crates/alloy-signer/src/conversion.rs::tests::undeclared_struct_reference_is_rejected` | 2026-06-03 |

## Construction & errors

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-AS-004` | `cow-sdk-alloy-signer` | `LocalAlloySignerBuilder::build` is callable only after private-key source and chain id have been selected, and externally constructed typestate markers cannot bypass the builder. Governed by [ADR 0035](../adr/0035-alloy-provider-adapter.md). | Contract | Yes | `crates/alloy-signer/src/builder.rs`, `crates/alloy-signer/tests/compile_fail.rs`, `crates/alloy-signer/tests/trybuild/external_marker_construction_fails.rs` | 2026-06-11 |
| `PROP-AS-007` | `cow-sdk-alloy-signer` | `SignerError::class()` returns one of the six signer classes for every variant, and validation, signing, internal, and private-key parsing details are redacted before public formatting. Governed by [ADR 0035](../adr/0035-alloy-provider-adapter.md). | Contract | Yes | `crates/alloy-signer/src/error.rs`, `crates/alloy-signer/tests/redaction_contract.rs`, `crates/alloy-signer/src/builder.rs` | 2026-05-06 |

## Cancellation propagation

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-AS-CANCEL-001` | `cow-sdk-alloy-signer` | `SignerError` carries a `Cancelled` variant and an `impl From<cow_sdk_core::Cancelled>` bridge so `Cancellable::cancel_with(...).await?` propagates cancellation through the signer error type. Governed by [ADR 0035](../adr/0035-alloy-provider-adapter.md). | Contract | Yes | `crates/alloy-signer/src/error.rs`, `crates/alloy-signer/tests/cancellation_contract.rs::cancel_with_propagates_cancelled_through_question_mark` | 2026-05-06 |
