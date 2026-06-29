---
type: Decision Record
id: ADR-0045
title: "ADR 0045: Narrow Async Signer Capabilities By Operation"
description: "Async signing capabilities stay split by operation."
status: Accepted
date: 2026-05-11
last_reviewed: 2026-05-11
authors: ["0xSymbiotic"]
tags: [signing, wasm, callbacks, capability-traits]
related: [ADR-0024, ADR-0040]
timestamp: 2026-05-11T00:00:00Z
---

# ADR 0045: Narrow Async Signer Capabilities By Operation

## Decision

Async signing capabilities stay split by operation. JavaScript and TypeScript signing
surfaces request only the capability they need: typed-data signing, digest
signing, cancellation signing, or custom EIP-1271 signing. Unsupported
operations fail before wallet dispatch instead of flowing through a broad
catch-all signer contract.

## Why

Wallets and smart-account clients often support different signing methods.
Narrow capability traits and typed callbacks make each method reviewable,
avoid placeholder implementations, and keep cancellation and EIP-1271 behavior
from depending on unrelated wallet methods.

## Must Remain True

- Public APIs name the required signing capability through their callback or
  config shape.
- Unsupported capability combinations fail with typed configuration or wallet
  errors before network or wallet side effects.
- ECDSA signatures normalize recovery bytes before leaving the signing seam.
- EIP-1271 custom callbacks preserve caller-provided contract-wallet
  signatures without forcing an EOA recovery path.
- Adding a new signing operation adds a targeted callback or capability trait
  rather than widening every signer.

## Alternatives Rejected

- Use one broad async signer trait for every operation: fewer names, but it
  hides unsupported methods until runtime.
- Force every wallet path through typed-data signing: simple for limit orders,
  but incorrect for digest and cancellation flows.
- Bundle JavaScript wallet adapters: easier demos, but it would make one wallet
  ecosystem part of the stable SDK contract.

## Links

- [WASM Surface Audit](../audit/wasm-surface-audit.md)
- [EIP-1271 Verification Cache Audit](../audit/eip1271-verification-cache-audit.md)

**Proven by:**

- [WASM Surface Audit](../audit/wasm-surface-audit.md)
- [EIP-1271 Verification Cache Audit](../audit/eip1271-verification-cache-audit.md)
