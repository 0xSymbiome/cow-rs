---
type: Audit
id: ecdsa-signature-normalization
title: "ECDSA Signature Normalization Audit"
description: "RecoverableSignature canonicalizes 65-byte ECDSA input to a Solidity-compatible 27/28 recovery byte through closed typestate construction, with typed failures and an opt-in low-s form."
status: Current
owning_surface: "RecoverableSignature and ECDSA address recovery"
related: [ADR-0022, ADR-0027]
timestamp: 2026-06-20
---

# ECDSA Signature Normalization Audit

## Scope

Reviews `RecoverableSignature` and ECDSA recovery at the contracts boundary:
the 65-byte input contract, the recovery-byte canonicalization, the typed
failure surface, scheme-bundled recover, the ERC-2098 compact bridge, and the
opt-in low-s form. It does not cover EIP-712 typed-data hashing (the Contract
Bindings Parity Audit) or EIP-1271 verification (the EIP-1271 Verification Cache
Audit).

## Findings

- The boundary validates only 65-byte payloads and reduces `v ∈ {0, 1, 27, 28}`
  to the canonical `{27, 28}` range before emitting bytes.
- Holding a `RecoverableSignature` is a compile-time proof the input contract was
  satisfied; construction is closed through `parse_hex` / `parse_bytes`.
- A length mismatch fails with `InvalidSignatureLength` and an unsupported
  recovery byte with `InvalidSignatureRecoveryByte` — a strict superset of the
  raw alloy rejection.
- Recovery applies the EIP-191 prehash for `eth_sign` and the EIP-712 digest for
  typed data, rejecting non-ECDSA schemes.
- Low-s canonicalization is opt-in and is not applied at parse time, preserving
  the orderbook's full accepted input set; the ERC-2098 compact form round-trips.

## Evidence

- Decision: [ADR 0022](../adr/0022-ecdsa-signature-v-normalization.md), [ADR 0027](../adr/0027-post-quantum-signing-absorption-plan.md).
- Invariants: the `PROP-CON` ([contracts](../properties/contracts.md)) and `PROP-SIG` ([signing](../properties/signing.md)) families.
- Governing gate: the `RecoverableSignature` parse contract in `crates/contracts/tests/`.
- Code: `crates/contracts/src/signature.rs`, `crates/contracts/src/errors.rs`, `crates/signing/src/order_signing.rs`.
