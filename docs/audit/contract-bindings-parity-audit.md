---
type: Audit
id: contract-bindings-parity
title: "Contract Bindings Parity Audit"
description: "Every alloy::sol! binding shipped in cow-sdk-contracts matches the TypeScript-SDK-derived call-data and EIP-712 fixtures byte-for-byte, is provenance-pinned to upstream, and builds under wasm32."
status: Current
owning_surface: "cow-sdk-contracts alloy::sol! bindings and their byte-identity parity"
related: [ADR-0012, ADR-0020, ADR-0026]
timestamp: 2026-06-20
---

# Contract Bindings Parity Audit

## Scope

Reviews the `alloy::sol!`-generated bindings in `cow-sdk-contracts` (settlement,
EthFlow, on-chain-order events, wrapped-native, IERC20), their byte-identity
parity against the TypeScript-SDK-derived fixtures, the shared EIP-712 domain
separator, and the wasm32 build path. It does not cover the COW Shed bindings
(the COW Shed Contract Bindings Audit) or the event decoders (the Event Log
Decoding Audit).

## Findings

- Every shipped binding is generated through `alloy::sol!` with no hand-rolled
  encoder, across the settlement, EthFlow, on-chain-order-event, wrapped-native,
  and ERC-20 families.
- Each binding's upstream mirror is pinned by commit in `parity/source-lock.yaml`
  so a reviewer can diff the inline interface against the pinned source.
- Encoded call-data and hashed payloads match the TypeScript-SDK-derived
  fixtures byte-for-byte.
- The EIP-712 domain separator routes through `alloy_sol_types` in both the
  contracts and signing crates, pinned by a shared fixture.
- The order typed-data struct hashes through `SolStruct`, with fixture rows
  pinning the wire bytes, and the EIP-1271 verifier payload reproduces the
  canonical word layout.
- The `alloy-primitives` `k256` path keeps the bindings buildable under
  `wasm32-unknown-unknown` via the `getrandom` `wasm_js` backend.

## Evidence

- Decision: [ADR 0012](../adr/0012-alloy-sol-bindings-and-registry-authority.md), [ADR 0020](../adr/0020-ethflow-owner-threading.md), [ADR 0026](../adr/0026-alloy-major-release-absorption-plan.md).
- Rule: [Canonical Contract Bindings](../principles/canonical-contract-bindings.md).
- Invariants: the `PROP-CON` ([contracts](../properties/contracts.md)) and `PROP-SIG` ([signing](../properties/signing.md)) families.
- Governing gate: `crates/contracts/tests/parity_contract.rs`.
- Code: `crates/contracts/src/`, `crates/signing/src/`, `parity/source-lock.yaml`, `parity/fixtures/`.
