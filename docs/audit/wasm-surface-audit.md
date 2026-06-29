---
type: Audit
id: wasm-surface
title: "WASM Surface Audit"
description: "The cow-sdk-js four-layer surface maps to the native crates through uniform transforms, pins its TypeScript contract with committed snapshots, and gates flavor builds on size budgets."
status: Current
owning_surface: "cow-sdk-js JavaScript and TypeScript crate, npm package, and runtime boundary"
related: [ADR-0039, ADR-0040, ADR-0041, ADR-0044, ADR-0045]
timestamp: 2026-06-26
---

# WASM Surface Audit

## Scope

Reviews the `cow-sdk-js` public surface — deterministic helpers, wallet
callbacks, service clients, and trading — its npm package layout, the callback
boundary, type generation, the error envelopes, the flavor size budgets, and the
unsupported-target diagnostics. It does not cover the native Alloy adapters (the
Alloy Adapters Audit) or the upstream TypeScript SDK.

## Findings

- The four layers each map to native crates through uniform transforms;
  deterministic helpers stay host-safe in `cow-sdk-js::helpers`, and wallet
  interop crosses only through typed callbacks.
- Types crossing the ABI carry `tsify` derives gated to `wasm32` (inert on
  native); the raw wasm-bindgen output is package-internal and denied as a public
  import target, so consumers see only the curated facade.
- One committed declaration snapshot per flavor pins the published TypeScript
  contract; the build diffs the bundler and nodejs targets against the snapshot
  and fails on drift.
- Success results carry a `schemaVersion` envelope; thrown errors normalize to a
  `CowError` subclass, input-DTO failures map to `invalidInput` (not `internal`),
  and the orderbook variant carries its typed `errorType`, `retryable`, and
  optional `retryAfterMs`.
- Flavor builds expose feature-scoped subpaths and run the release size profile;
  the raw, brotli, and gzip budgets are recorded and gated.
- The native Alloy adapter crates compile empty on `wasm32`, and enabling any
  `alloy` feature on `wasm32` is a compile-time diagnostic, so a browser build
  cannot silently pull native RPC.

## Evidence

- Decision: [ADR 0039](../adr/0039-typescript-callable-wasm-sdk-surface.md), [ADR 0040](../adr/0040-wallet-provider-callback-boundary-for-js-consumers.md), [ADR 0041](../adr/0041-transport-policy-l3-layering.md), [ADR 0044](../adr/0044-bundle-size-profile-and-flavor-builds.md), [ADR 0045](../adr/0045-async-signer-trait-narrowing.md).
- Rule: [Additive Optional Ecosystems](../principles/additive-optional-ecosystems.md).
- Invariants: the `PROP-WB` family ([JS/WASM boundary](../properties/js.md)).
- Governing gate: `wasm-pack test --headless --firefox` plus the declaration-snapshot contract.
- Code: `crates/js/src/exports/`, `crates/js/src/dto/`, `crates/js/snapshots/`, `crates/sdk/src/lib.rs`.
