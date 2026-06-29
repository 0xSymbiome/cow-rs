---
type: Audit
id: eip1271-verification-cache
title: "EIP-1271 Verification Cache Audit"
description: "The Eip1271Cache trait memoizes only positive verification outcomes keyed on full probe identity; the SDK ships only the always-available no-op cache, with native and wasm payload parity."
status: Current
owning_surface: "the Eip1271Cache trait, NoopEip1271Cache, and the wasm EIP-1271 payload parity"
related: [ADR-0014, ADR-0027, ADR-0028, ADR-0040, ADR-0045]
timestamp: 2026-06-24
---

# EIP-1271 Verification Cache Audit

## Scope

Reviews the `Eip1271Cache` trait and its always-available `NoopEip1271Cache`,
the `verify_eip1271_signature_cached` orchestration, the cache-key and
positive-only recording policy, and the native/wasm/TS payload parity. It does
not cover pre-interaction verification or the on-chain `isValidSignature` call
semantics beyond the magic-value contract.

## Findings

- The trait is two methods (`contains_valid` / `record_valid`) with
  `Send + Sync + 'static` bounds, so a consumer cache plugs in without touching
  the verification path.
- The cache key is the full probe identity — verifier, digest, and
  `keccak256(signature)` — so a `VALID` recorded for one signature is never
  served for a different signature on the same digest.
- Only a magic-value match is recorded; mismatches and all error classes bypass
  recording, so a transient failure never pins a signer as valid.
- The orchestrator emits a verification tracing span carrying the cache status
  and result and never records payload bytes.
- The SDK ships only the zero-sized `NoopEip1271Cache`; a consumer implements the
  trait to memoize, and the wasm EIP-1271 payload matches the native and
  upstream-TypeScript vectors.

## Evidence

- Decision: [ADR 0014](../adr/0014-eip1271-verification-cache.md), [ADR 0027](../adr/0027-post-quantum-signing-absorption-plan.md), [ADR 0028](../adr/0028-account-abstraction-integration-plan.md), [ADR 0040](../adr/0040-wallet-provider-callback-boundary-for-js-consumers.md), [ADR 0045](../adr/0045-async-signer-trait-narrowing.md).
- Invariants: the `PROP-CON` family ([contracts](../properties/contracts.md)).
- Governing gate: the `verify_eip1271_signature_cached` contract in `crates/contracts/tests/`.
- Code: `crates/contracts/src/verify.rs`, `crates/signing/src/cache.rs`, `crates/js/src/exports/eip1271.rs`.
