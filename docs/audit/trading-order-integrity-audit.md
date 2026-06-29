---
type: Audit
id: trading-order-integrity
title: "Trading Order Integrity Audit"
description: "Order assembly in cow-sdk-trading preserves balance and same-token semantics, gates every submission through the bounds validator, threads EthFlow owner identity once, and merges app-data through one typed pipeline."
status: Current
owning_surface: "cow-sdk-trading order assembly, bounds validation, app-data merge, and EthFlow owner threading"
related: [ADR-0002, ADR-0015, ADR-0018, ADR-0020]
timestamp: 2026-06-20
---

# Trading Order Integrity Audit

## Scope

Reviews order construction and submission in `cow-sdk-trading`: balance and
same-token semantics, the mandatory `OrderBoundsValidator` pre-transport gate,
the post-sign owner-recovery check, the typed quote-to-post app-data merge, and
EthFlow owner identity. It does not cover the orderbook wire DTOs or the
transport that uploads the signed order.

## Findings

- Every public submission seam runs `OrderBoundsValidator::validate` between
  order construction and HTTP upload; the validator is pure (it takes `now` as a
  parameter and performs no I/O), so a submission is deterministic and replayable.
- Rejections surface through the `#[non_exhaustive]` `ClientRejection` set lifted
  onto `TradingError::ClientRejected`, so a new invariant adds a typed variant
  rather than a silent pass.
- Same-token orders match the services policy — a buy-side exact same-token order
  is rejected while a sell-side order is accepted — including the WETH-paired
  native-sentinel case.
- The post-sign owner-recovery gate recovers the signer from the produced
  signature and rejects `OwnerMismatch` when it differs from the declared owner;
  the pre-sign and EIP-1271 paths skip the gate by construction.
- App-data edits run through one typed merge helper: an override `metadata.hooks`
  replaces the base set while `metadata.signer` and `metadata.flashloan` survive,
  and the document round-trips idempotently.
- EthFlow threads the signer-derived owner once onto the transaction's `from`
  field and passes that — not the order receiver — to the validator.

## Evidence

- Decision: [ADR 0002](../adr/0002-dedicated-trading-orchestration-crate.md), [ADR 0015](../adr/0015-client-side-order-bounds-validator.md), [ADR 0018](../adr/0018-typed-app-data-merge.md), [ADR 0020](../adr/0020-ethflow-owner-threading.md).
- Invariants: the `PROP-TRD` family ([trading lifecycle](../properties/trading.md)).
- Governing gate: `cargo test -p cow-sdk-trading --test validation_contract`.
- Code: `crates/trading/src/validation.rs`, `crates/trading/src/app_data.rs`, `crates/trading/src/onchain.rs`, `crates/trading/src/post.rs`.
