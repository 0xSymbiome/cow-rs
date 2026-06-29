---
type: Decision Record
id: ADR-0022
title: "ADR 0022: Canonicalize ECDSA Signature v At The Contracts Boundary"
description: "cow_sdk_contracts::RecoverableSignature is the single contracts-boundary typestate for recoverable ECDSA signatures."
status: Accepted
date: 2026-04-23
last_reviewed: 2026-05-28
authors: ["0xSymbiotic"]
tags: [contracts, signing, ecdsa, normalization, error-typing]
related: [ADR-0005, ADR-0015, ADR-0017, ADR-0052]
timestamp: 2026-05-28T00:00:00Z
---

# ADR 0022: Canonicalize ECDSA Signature `v` At The Contracts Boundary

## Decision

`cow_sdk_contracts::RecoverableSignature` is the single
contracts-boundary typestate for recoverable ECDSA signatures. The
closed constructors (`RecoverableSignature::parse_hex` and
`RecoverableSignature::parse_bytes`) accept only 65-byte signatures,
canonicalize modern `v = 0` / `1` inputs onto the legacy
Solidity-compatible `27` / `28` range, preserve already-canonical
`27` / `28` inputs byte-for-byte, and reject every other trailing byte
through the typed `ContractsError::InvalidSignatureRecoveryByte`
variant. Length mismatch fails through the typed
`ContractsError::InvalidSignatureLength` variant. Holding a
`RecoverableSignature` is a compile-time proof that the recovery byte
has been canonicalised, so downstream signing, recovery, and
verification helpers consume the typestate directly rather than passing
raw bytes around.

## Why

Solidity `ecrecover` expects the legacy `27` / `28` recovery-byte range.
Modern signers frequently emit `0` / `1`, which remains valid
off-chain but fails once a signature is forwarded unchanged into an
on-chain verification path. The CoW services backend and the TypeScript
SDK already normalize this boundary, so leaving the Rust contracts crate
as a pass-through surface created an avoidable cross-SDK divergence on a
trust-significant signing invariant. Putting the rule in
`cow-sdk-contracts` keeps the boundary local to the helper every signing
path already uses instead of scattering the fix across downstream call
sites.

## Must Remain True

- Public surface: `RecoverableSignature::parse_hex(&str)` and
  `RecoverableSignature::parse_bytes(&[u8])` are the only construction
  paths. Both return a `RecoverableSignature` whose canonical output
  (via `to_bytes` / `to_hex_string`) lowercases the hex, preserves
  `r || s`, maps `v = 0` and `27` to `27`, maps `v = 1` and `28` to
  `28`, rejects any other `v` byte through
  `InvalidSignatureRecoveryByte { value }`, and rejects any non-65-byte
  payload through `InvalidSignatureLength { actual }`. The scheme-bundled
  `RecoverableSignature::recover(digest, scheme)` reaches secp256k1
  recovery for the contracts-boundary surface.
- Runtime and support: every contracts, signing, alloy-signer, and
  WASM helper that emits or repackages recoverable ECDSA signatures
  routes through `RecoverableSignature`, so downstream Solidity
  verification paths always receive the legacy-compatible form. The
  change does not widen the accepted signature family beyond the four
  reviewed values.
- Validation and review: curated regression coverage in
  `crates/contracts/tests/signature_contract.rs` and
  `crates/contracts/tests/recoverable_signature_contract.rs` pins the
  accepted and rejected boundary cases; the parity contract at
  `crates/contracts/tests/v_normalization_contract.rs` drives the accept
  and rejection rows in `parity/fixtures/ecdsa/v_normalization.json`; the fuzz target
  `fuzz/fuzz_targets/fuzz_ecdsa_v_normalization.rs` asserts that
  arbitrary 65-byte inputs either preserve `r || s` with a canonical
  `27` / `28` output or fail through the typed recovery-byte rejection;
  and the differential fuzz target
  `fuzz/fuzz_targets/fuzz_recoverable_signature_differential.rs`
  asserts that the cow rejection set is a strict refinement of the
  alloy parity-normalization rejection set on the same 65-byte input
  space.
- Additional surfaces on the same typestate: `to_erc2098` / `parse_erc2098`
  expose the ERC-2098 compact 64-byte form (delegating to alloy's `as_erc2098` /
  `from_erc2098`), and `canonicalized_low_s` exposes opt-in BIP-62 low-s
  canonicalisation (not applied at parse time, since the orderbook accepts both).
  Recovery delegates to alloy's `from_bytes_and_parity` /
  `recover_address_from_prehash`; the `ecdsa-v-normalization` source fence forbids
  `Signature::from_raw` and `Signature::as_rsy` (either would readmit the wider
  alloy parity-normalization input or emit the raw `{0, 1}` parity byte).
- Cost: two additive `ContractsError` variants and one stricter
  contracts-boundary helper. No new signing scheme, wire format, or
  on-chain ABI surface is introduced.

## Alternatives Rejected

- Leave the contracts crate's recoverable-signature surface as a
  hex-only passthrough: simplest implementation, but leaves the Rust
  SDK divergent from the TypeScript SDK and the services backend on a
  reviewed signing invariant.
- Normalize only in `cow-sdk-signing`: fixes some call sites, but leaves
  other contracts-boundary consumers free to reintroduce the bug and
  weakens the contracts crate as the canonical signature authority.
- Silently coerce every trailing byte into the legacy range: avoids an
  error, but destroys typed failure semantics and risks turning malformed
  signatures into different malformed signatures rather than rejecting
  them explicitly.
- Expose the alloy `Signature` primitive directly as the cow recoverable
  type (type alias): admits the wider alloy parity-normalization input
  surface (EIP-155 `v >= 35`) and prevents the typed
  `InvalidSignatureRecoveryByte` rejection contract from being enforced
  by the type system.

## Prefix Ownership

EthSign signing and recovery split EIP-191 prefix ownership across the signing
and contracts crates. `cow-sdk-signing` routes the raw 32-byte order digest to
`Signer::sign_message`; the wallet's personal-sign semantics adds the
`"\x19Ethereum Signed Message:\n32"` prefix. The signing crate must not prepend
that prefix itself, because doing so would double-prefix the payload and make
the signature fail against the settlement contract.

`cow-sdk-contracts` applies the EIP-191 prefix only at recovery time, through
the private `eth_sign_digest_prehash` helper, because recovery has no wallet
step that can add the prefix. The helper remains private; integration coverage
exercises the invariant through the public
`Signature::recover_ecdsa_address` API.

The pinned upstream signing posture signs the keccak digest of
`hashTypedData(...)` as a personal-sign message, so the wallet/provider layer
owns the EIP-191 prefix application and recovery-byte direction during signing.
The SDK normalizes the recovery byte through
`cow_sdk_contracts::RecoverableSignature` regardless of which adapter
produced the signature.

## Links

- [Architecture](../guides/architecture.md)
- [Verification Guide](../guides/verification.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0015](0015-client-side-order-bounds-validator.md)
- [ADR 0017](0017-typed-orderbook-rejection-parser.md)

**Proven by:**

- [ECDSA Signature Normalization Audit](../audit/ecdsa-signature-normalization-audit.md)
