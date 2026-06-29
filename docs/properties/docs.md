---
type: Property
id: docs
title: "Documentation governance invariants"
description: "Documentation and audit-lane governance: doc-agreement gates, the curated facade documentation, and the two named audit-lane docs covering the panic-free public surface and workflow-security contracts."
resource: https://github.com/0xSymbiome/cow-rs/blob/main/docs/properties/docs.md
families: [PROP-AUD, PROP-DOCS]
tags: [property, invariants]
timestamp: 2026-06-29T00:00:00Z
---

# Documentation governance invariants

Documentation and audit-lane governance: doc-agreement gates, the curated facade documentation, and the two named audit-lane docs covering the panic-free public surface and workflow-security contracts. Part of the [Properties Registry](index.md): 5 invariant(s), 5 covered.

## Documentation governance

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-DOCS-001` | whole workspace | Every published crate with a per-crate `README.md` declares it in package metadata, every per-crate README that carries Rust examples is compile-gated through crate rustdoc via `cargo test --workspace --doc`, and the source-level `check-readme-include` policy asserts that the docs.rs-rendered crates' `lib.rs` files render their README on docs.rs (via `#![cfg_attr(any(doctest, docsrs), doc = include_str!("../README.md"))]`), with the `docs-quality` workflow building the docs.rs-style documentation under `-D warnings`. The policy-enforced docs.rs-rendering set is `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data`, `cow-sdk-orderbook`, `cow-sdk-trading`, `cow-sdk-subgraph`, and `cow-sdk`; the thin native alloy adapter crates (`cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, `cow-sdk-alloy`) deliberately include their README under `doctest` only and stay out of the docs.rs-rendering policy. | Contract | Yes | `crates/core/README.md`, `crates/contracts/README.md`, `crates/signing/README.md`, `crates/app-data/README.md`, `crates/orderbook/README.md`, `crates/trading/README.md`, `crates/subgraph/README.md`, `crates/js/README.md`, `crates/alloy-provider/README.md`, `crates/alloy-signer/README.md`, `crates/alloy/README.md`, `crates/sdk/README.md`, `crates/core/src/lib.rs`, `crates/contracts/src/lib.rs`, `crates/signing/src/lib.rs`, `crates/app-data/src/lib.rs`, `crates/orderbook/src/lib.rs`, `crates/trading/src/lib.rs`, `crates/subgraph/src/lib.rs`, `crates/sdk/src/lib.rs`, `crates/js/src/lib.rs`, `crates/alloy-provider/src/lib.rs`, `crates/alloy-signer/src/lib.rs`, `crates/alloy/src/lib.rs`, `xtask/src/policy/check_readme_include.rs`, `.github/workflows/docs-quality.yml` | 2026-06-20 |
| `PROP-DOCS-002` | whole workspace | Workspace MSRV bump policy is documented at `docs/guides/msrv-policy.md` with explicit cadence, notice window, and trigger criteria; the root workspace MSRV stays aligned with CI. | Contract | Yes | `docs/guides/msrv-policy.md`, `README.md`, `CONTRIBUTING.md`, `tests/msrv_consistency.rs`, `.github/workflows/ci.yml` | 2026-05-01 |
| `PROP-DOCS-003` | `cow-sdk` | The `cow-sdk` facade ships no prelude: the crate root re-exports each leaf crate as a named module plus the cross-cutting aggregate error (`CowError` / `ErrorClass`) and the typed transport leaf surface (`cow_sdk::http`), while every workflow and identity type is reached on its module path. The root public surface stays explicit and compile-probed for headline-type reachability, and no crate in the workspace ships a prelude. Governed by [ADR 0001](../adr/0001-multi-crate-sdk-family-with-thin-facade.md). | Public API | Yes | `crates/sdk/src/lib.rs`, `crates/sdk/tests/public_api.rs`, `crates/sdk/tests/public_api_default_features_only.rs`, `crates/sdk/tests/public_api_with_all_features.rs` | 2026-06-08 |
| `PROP-DOCS-004` | whole workspace | `SECURITY.md` documents the base-URL override threat surface with operator-side mitigations. | Contract | Yes | `SECURITY.md` | 2026-06-16 |

## Audit lanes

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-AUD-001` | whole workspace | Two named audit-lane docs cover the panic-free public surface contract and the workflow security posture. | Contract | Yes | `docs/audit/panic-free-public-surface-audit.md`, `docs/audit/workflow-security-audit.md`, `.github/workflows/_quality-gate.yml` | 2026-05-26 |
