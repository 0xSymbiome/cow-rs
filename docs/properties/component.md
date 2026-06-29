---
type: Property
id: component
title: "WASM Component boundary invariants"
description: "The `cow-sdk-component` WebAssembly Component / WIT boundary: the one-world-per-cdylib guard, the keys-out / node-out host-import posture, the host-free deterministic engine, the native golden parity that pins the wrapped-SDK outputs, the hand-mirrored book-record drift gate, and the versioned `cow:protocol` contract."
resource: https://github.com/0xSymbiome/cow-rs/blob/main/docs/properties/component.md
families: [PROP-CMP]
tags: [property, invariants]
timestamp: 2026-06-29T00:00:00Z
---

# WASM Component boundary invariants

The `cow-sdk-component` WebAssembly Component / WIT boundary: the one-world-per-cdylib guard, the keys-out / node-out host-import posture, the host-free deterministic engine, the native golden parity that pins the wrapped-SDK outputs, the hand-mirrored book-record drift gate, and the versioned `cow:protocol` contract. The protocol math these interfaces lower is owned by the wrapped crates — order identity and amounts by [Core codec invariants](core.md), the TWAP encoding by [Contract binding invariants](contracts.md), and the quote pipeline by [Trading lifecycle invariants](trading.md) — so the rows here assert only the boundary: the contract shape, the lowering, and the no-fork parity. Part of the [Properties Registry](index.md): 7 invariant(s), 3 covered.

## Contract shape & build worlds

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-CMP-001` | `cow-sdk-component` | One `cdylib` is one component, so exactly one of `world-engine` / `world-client-sync` / `world-client-async` may be selected per build. A wasm32 `compile_error!` guard fails the build — with the message `select exactly one world` — when none or more than one world feature is active, so a misconfigured build cannot emit an ambiguous component. The guard is asserted by the component CI job, which builds `--no-default-features` and matches the message. Governed by [ADR 0071](../adr/0071-wasm-component-distribution-channel.md). | Contract | Partial | `crates/component/src/lib.rs`, `.github/workflows/component.yml` | 2026-06-29 |
| `PROP-CMP-002` | `cow-sdk-component` | The contract is published under the versioned package id `cow:protocol@0.1.0`; pre-1.0 it is experimental (`0.x`). The version travels with the WIT source the drift gate compiles against, so a consumer pins a named contract version rather than an unversioned shape. A `.wit` snapshot gate that pins the whole contract is planned and not yet wired (ADR 0071). Governed by [ADR 0071](../adr/0071-wasm-component-distribution-channel.md). | Contract | Partial | `crates/component/wit/world.wit`, `tests/component_wit_record_drift.rs` | 2026-06-29 |

## Host-free engine & keys-out / node-out posture

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-CMP-003` | `cow-sdk-component` | The deterministic `order-engine` world declares no host imports — only exports (order identity, chain and deployment introspection, app-data, the gas-free transaction builders, the composable and trading-math surfaces, the signing payloads, and event-log decoding) — so it runs in any component runtime with no host wiring. The engine world builds clean for `wasm32-wasip2` under `-D warnings` in CI; a host import pulled into the engine module would fail that build. Governed by [ADR 0071](../adr/0071-wasm-component-distribution-channel.md). | Contract | Partial | `crates/component/wit/world.wit`, `crates/component/src/engine/mod.rs`, `.github/workflows/component.yml` | 2026-06-29 |
| `PROP-CMP-004` | `cow-sdk-component` | Keys-out and node-out: signing is a host `signer` import and read-only contract access is a host `contract-read` import, so the WIT exposes no private-key input and the component never holds a signing key or an RPC socket. The component computes the EIP-712 digest and asks the host to sign it; the host-backed `Signer` carries no key field and the host-backed read `Provider` answers only `read_contract`, every other provider method returning a read-only error. Governed by [ADR 0071](../adr/0071-wasm-component-distribution-channel.md). | Contract | Partial | `crates/component/wit/world.wit`, `crates/component/src/client/core.rs` | 2026-06-29 |

## No-fork parity & record drift

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-CMP-005` | `cow-sdk-component` | The crate wraps the SDK crates and never forks protocol logic; a native golden suite pins the deterministic outputs the boundary lowers — the order UID embeds its EIP-712 digest, the gas-free transaction builders resolve the canonical targets and selectors, the signing payloads name their primary types and agree with the standalone id, and event decoding fails closed on an unknown topic. The suite runs natively against the linked engine; runtime reproduction through jco and a Wasmtime host is planned and not yet wired (ADR 0071), so this row pins the native parity, not the compiled component's runtime output. Governed by [ADR 0071](../adr/0071-wasm-component-distribution-channel.md). | Contract | Yes | `crates/component/src/engine/tests.rs::uid_embeds_the_order_digest`, `crates/component/src/engine/tests.rs::tx_helpers_resolve_targets_and_encode_canonical_calls`, `crates/component/src/engine/tests.rs::order_signing_payloads_are_canonical`, `crates/component/src/engine/tests.rs::event_decoding_is_wired_and_fails_closed` | 2026-06-29 |
| `PROP-CMP-006` | `cow-sdk-component` | The hand-mirrored `book.order` / `book.trade` WIT records are pinned to the native `cow_sdk_orderbook::Order` / `Trade` serde shapes: a drift gate serializes the native type from the full-metadata parity fixture and asserts every emitted wire field is present in the WIT record, so a native field added upstream fails CI rather than being silently dropped from the typed surface. The rich quote tree and long-tail reads stay JSON by deliberate cost tradeoff (ADR 0071). Governed by [ADR 0071](../adr/0071-wasm-component-distribution-channel.md). | Contract | Yes | `tests/component_wit_record_drift.rs::book_order_record_mirrors_native_order`, `tests/component_wit_record_drift.rs::book_trade_record_mirrors_native_trade`, `crates/component/wit/world.wit` | 2026-06-29 |

## Composable & trading-math lowering

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-CMP-007` | `cow-sdk-component` | The engine lowers the pure composable (TWAP) and quote-pipeline surfaces without reimplementing their math — the TWAP encoding and schedule classification are `cow-sdk-contracts::composable` (see [contracts.md](contracts.md)) and the amounts / slippage / app-data document are `cow-sdk-trading` and `cow-sdk-core` (see [trading.md](trading.md), [core.md](core.md)). The boundary contribution is the fail-closed mapping of the `#[non_exhaustive]` `TwapTiming` classifier — a future schedule state errors rather than emitting an unmatchable WIT variant — and the integer-exact lowering of the stage breakdown into the WIT record, identical to the client lanes. A native golden pins both: the TWAP targets `ComposableCoW` and classifies not-started / active / expired, and the quote breakdown pins the stage amounts, the slippage arithmetic, the eth-flow floor, and the stamped app-data document. Governed by [ADR 0071](../adr/0071-wasm-component-distribution-channel.md). | Contract | Yes | `crates/component/src/engine/tests.rs::twap_encoding_targets_composablecow_and_classifies_schedule`, `crates/component/src/engine/tests.rs::trading_math_breaks_down_amounts_suggests_slippage_and_builds_app_data`, `crates/component/src/engine/world.rs` | 2026-06-29 |
