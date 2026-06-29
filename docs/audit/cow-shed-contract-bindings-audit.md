---
type: Audit
id: cow-shed-contract-bindings
title: "COW Shed Contract Bindings Audit"
description: "The inline COW Shed bindings mirror the deployed v1.0.x generation, with byte-pinned creation code, CREATE2 derivation, EIP-712 hashing, and EOA signature byte order."
status: Current
owning_surface: "COW Shed bindings, proxy creation code, and app-data hook integration"
related: [ADR-0049, ADR-0051]
timestamp: 2026-06-20
---

# COW Shed Contract Bindings Audit

## Scope

Reviews the inline COW Shed `alloy::sol!` bindings against the deployed v1.0.x
sources: the selector record, the proxy creation-code blobs and deployment
record, CREATE2 derivation, EIP-712 domain/digest hashing, EOA signature byte
order, and the app-data hook integration. It does not cover per-chain deployed
addresses (the Deployment Registry Audit) or the general binding parity (the
Contract Bindings Parity Audit).

## Findings

- Both `sol!` interfaces declare only the functions, events, and errors present
  in the deployed v1.0.x sources at the pinned tag.
- Selectors are triple-checked: an independent keccak derivation, a pinned
  fixture value, and the macro-emitted `SELECTOR` constant all agree.
- The proxy creation-code blobs are byte-identical to the arbiter constants and
  pinned by length plus keccak256; the deployment record pins per-version
  factory/implementation pairs and `VERSION()` domain strings. (Per-chain
  provenance lives in the Deployment Registry Audit.)
- CREATE2 derivation runs through `alloy_primitives::Address::create2` over the
  user-word salt and the keccak-hashed init code, with arbiter golden vectors as
  external anchors.
- The EIP-712 domain separator and `ExecuteHooks` digest are produced through
  `alloy_sol_types`, with fixtures locking the bytes and independent keccak
  re-derivation of the type hashes.
- EOA signature byte order is `r‖s‖v` with `v ∈ {27, 28}`; hook metadata reuses
  the existing app-data `Hook` schema, and the EIP-1271 path uses the
  signing-owned `Eip1271Signer` trait.

## Evidence

- Decision: [ADR 0049](../adr/0049-cow-shed-account-abstraction-proxy.md), [ADR 0051](../adr/0051-signing-owned-eip1271-signature-provider-trait.md).
- Rule: [Canonical Contract Bindings](../principles/canonical-contract-bindings.md).
- Invariants: the `PROP-CON` / `PROP-SHED` ([contracts](../properties/contracts.md)) and `PROP-SIG` ([signing](../properties/signing.md)) families.
- Governing gate: `crates/contracts/tests/selector_parity_cow_shed_contract.rs`.
- Code: `crates/contracts/src/cow_shed/`, `crates/app-data/src/metadata/hooks.rs`, `crates/signing/src/eip1271/`, `parity/fixtures/cow_shed/`.
