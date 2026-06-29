---
type: Audit
id: alloy-adapters
title: "Alloy Adapters Audit"
description: "The read-only Alloy provider, the local EIP-712 signer, and the composed umbrella each implement their SDK-owned trait contracts, redact secrets, and stay within the native adapter boundary."
status: Current
owning_surface: "native Alloy adapter family and the LogProvider capability"
related: [ADR-0026, ADR-0035, ADR-0038, ADR-0057]
timestamp: 2026-06-21
---

# Alloy Adapters Audit

## Scope

Reviews the native Alloy adapters: the read-only provider and its six `Provider`
methods, the local signer and its EIP-191 / payload-only EIP-712 signing, the
umbrella that composes `Provider` + `SigningProvider` + `LogProvider`, the
transaction-lifecycle / receipt types, redaction, cancellation, and the
dependency boundary. It does not cover the core trait definitions themselves or
the contract bindings (the Contract Bindings Parity Audit).

## Findings

- The read-only provider implements all six `Provider` methods; the local signer
  implements EIP-191 and payload-only EIP-712; the umbrella composes `Provider`,
  `SigningProvider`, and `LogProvider`.
- EIP-712 signatures preserve the order primary type and match the reference
  vectors, and every signature normalizes through `RecoverableSignature` so the
  emitted recovery byte stays Solidity-compatible.
- Transaction broadcast returns a hash only without waiting; receipts populate
  status via EIP-658 with no coercion of pre-Byzantium post-state.
- `read_contract` ABI-encodes, dispatches `eth_call`, decodes through
  `alloy-dyn-abi`, returns a typed result, and fails malformed input without
  panicking.
- `LogProvider` issues exactly one bounded `get_logs` call with no watcher or
  iterator loop, mirroring the `SigningProvider` split.
- The adapter error types redact provider URLs and key material and propagate
  cancellation; only the reviewed adapter crates may carry `alloy-provider` /
  `alloy-signer-local`, enforced by a dependency gate.

## Evidence

- Decision: [ADR 0035](../adr/0035-alloy-provider-adapter.md), [ADR 0026](../adr/0026-alloy-major-release-absorption-plan.md), [ADR 0038](../adr/0038-transaction-lifecycle-types.md), [ADR 0057](../adr/0057-log-provider-capability-trait.md).
- Rule: [Chain-RPC Runtime Neutrality](../principles/chain-rpc-runtime-neutrality.md).
- Invariants: the `PROP-AU` ([alloy](../properties/alloy.md)), `PROP-AP` ([alloy provider](../properties/alloy-provider.md)), and `PROP-AS` ([alloy signer](../properties/alloy-signer.md)) families.
- Governing gate: the cross-adapter `read_contract` parity invariant test.
- Code: `crates/alloy-provider/src/`, `crates/alloy-signer/src/`, `crates/alloy/src/`.
