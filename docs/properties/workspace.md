---
type: Property
id: workspace
title: "Workspace policy invariants"
description: "Workspace-wide contracts: native Alloy dependency boundaries, typestate marker fences, cross-crate dependency-pin lockstep, `source-lock.yaml` form validation, MSRV alignment, atomic-unit `Amount` parsing, and the native receipt/transaction-submission lifecycle adapters."
resource: https://github.com/0xSymbiome/cow-rs/blob/main/docs/properties/workspace.md
families: [PROP-WS, PROP-WS-RX, PROP-WS-TX]
tags: [property, invariants]
timestamp: 2026-06-29T00:00:00Z
---

# Workspace policy invariants

Workspace-wide contracts: native Alloy dependency boundaries, typestate marker fences, cross-crate dependency-pin lockstep, `source-lock.yaml` form validation, MSRV alignment, atomic-unit `Amount` parsing, and the native receipt/transaction-submission lifecycle adapters. Part of the [Properties Registry](index.md): 9 invariant(s), 8 covered.

## Dependency, version & release policy

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-WS-001` | whole workspace | Native Alloy dependency boundaries are explicit: `alloy-provider` is allowed only in `cow-sdk-alloy-provider` and `cow-sdk-alloy`, while `alloy-signer-local` is allowed only in `cow-sdk-alloy-signer` and `cow-sdk-alloy`; the default facade still excludes these native adapters unless their opt-in features are enabled. | Contract | Yes | `.github/workflows/_quality-gate.yml` (policy job), `xtask/src/policy/dependency_invariant.rs` | 2026-05-06 |
| `PROP-WS-004` | whole workspace | CI enforces dependency-pin lockstep for the cross-crate protocol stack: Alloy runtime crates remain on the `2.0` release family, Alloy ABI/core crates remain on the `1.5` release family, direct nested Alloy pins stay in the same two-family policy. | Contract | Yes | `.github/workflows/_quality-gate.yml`, `.github/workflows/alloy-release-candidate.yml`, `tests/workspace_alloy_pin_lockstep.rs`, `tests/alloy_two_family_lockfile_invariant.rs`, `Cargo.toml`, `Cargo.lock`, `parity/source-lock.yaml` | 2026-05-06 |
| `PROP-WS-009` | whole workspace | Workspace release-gate policy inputs remain executable: the MSRV declared in Cargo matches CI, dependency default-feature policy is enforced from the root workspace table, enum-policy classifications match source attributes, and every RustSec audit ignore token has documented dependency-gate rationale. | Contract | Yes | `tests/msrv_consistency.rs`, `tests/dependency_default_features_audit.rs`, `xtask/src/policy/check_enum_policy.rs` (run via `cargo check-enum-policy`), `xtask/src/docs/agree.rs`, `.github/config/enum-policy.yaml`, `.github/config/deny.toml`, `docs/audit/dependency-gate-audit.md` | 2026-06-14 |

## Public-surface fences

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-WS-002` | whole workspace | SDK-constructed response and error-payload structs in `cow-sdk-app-data` (`AppDataValidated`, `AppDataInfo`), `cow-sdk-subgraph` (`SubgraphGraphQlError`, `SubgraphGraphQlErrorLocation`, `SubgraphRequestErrorContext`), and `cow-sdk-signing` (`SigningResult`, `GeneratedOrderId`) carry `#[non_exhaustive]` at the struct head so additive fields remain compatible for downstream consumers. Caller-built request and configuration structs deliberately do not carry the marker (see the Forward-Compatible Public Surfaces principle); they expose `new()` plus `with_*()` builders so additive fields land without blocking literal construction. | Public API | Partial | `crates/app-data/src/info.rs`, `crates/app-data/src/types/validation.rs`, `crates/subgraph/src/error.rs`, `crates/signing/src/order_signing.rs`, `docs/principles/index.md` | 2026-06-11 |
| `PROP-WS-003` | whole workspace | Typestate marker structs across `cow-sdk-orderbook`, `cow-sdk-subgraph`, and `cow-sdk-trading` are sealed with private tuple fields, so external callers cannot construct marker values directly and must use the documented builder transition methods. | Public API | Yes | `crates/orderbook/src/builder.rs`, `crates/subgraph/src/builder.rs`, `crates/trading/src/client/mod.rs`, `crates/trading/src/client/builder.rs`, `crates/contracts/tests/ui/typestate_marker_sealing.rs` | 2026-05-31 |

## Provenance & primitives

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-WS-007` | whole workspace | `xtask` validates `source-lock.yaml` by form: the typed model rejects unknown or missing fields (`deny_unknown_fields`), and each repository row must carry a GitHub `.git` remote, a 40-character lowercase hex commit, and unique non-traversing producer paths. Every fixture under `parity/fixtures/**/*.json` is validated per-file: a unique `surface`, a `sources`/`standards` provenance header, source commits equal to the owning pin (the freshness ratchet), refs and case-level `source_ref`s confined to declared producer paths, provenance-lookalike keys (`source`, `source_refs`, `@source_ref`) rejected, and the vendored OpenAPI stamp equal to the services pin. The lock carries no schema-version field — its only parser ships in the same commit, so skew cannot occur and malformed shapes fail closed at parse time. | Contract | Yes | `xtask/src/parity/mod.rs (tests)`, `.github/workflows/_quality-gate.yml` | 2026-06-11 |
| `PROP-WS-008` | whole workspace | Representative atomic-unit `Amount` strings (zero through the full uint256 ceiling) parse through `cow_sdk_core::Amount::new` and render back byte-identically, and the parse is deterministic (the same literal always decodes to the same typed `Amount`). | Contract | Yes | `crates/core/tests/property_contract.rs::amount_roundtrips_through_u256_and_wire_string` | 2026-05-01 |

## Native transaction lifecycle

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-WS-RX-001` | whole workspace | The native Alloy receipt adapters expose a typed `TransactionReceipt` lifecycle contract: absent optional fields remain empty, present malformed receipt fields fail closed, EIP-658 status maps to success or reverted, and contract creation keeps `to` empty. Governed by [ADR 0038](../adr/0038-transaction-lifecycle-types.md). | Contract | Yes | `crates/alloy-provider/tests/provider_contract.rs::get_transaction_receipt_populates_status_block_gas_from_to`, `crates/alloy/tests/provider_contract.rs::get_transaction_receipt_populates_rich_fields_from_alloy_receipt`, `tests/transaction_lifecycle_cross_adapter_invariant.rs` | 2026-06-16 |
| `PROP-WS-TX-001` | whole workspace | Signer-backed transaction submission returns broadcast acknowledgement without implicit receipt polling across the native Alloy adapters. Governed by [ADR 0038](../adr/0038-transaction-lifecycle-types.md). | Contract | Yes | `crates/alloy/tests/send_transaction_does_not_wait_for_confirmation.rs::send_transaction_does_not_dispatch_get_transaction_receipt`, `tests/transaction_lifecycle_cross_adapter_invariant.rs::alloy_send_transaction_does_not_poll_for_receipt`, `examples/native/scenarios/transaction_lifecycle.rs` | 2026-06-16 |
