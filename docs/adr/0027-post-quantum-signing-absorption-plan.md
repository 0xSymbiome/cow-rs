---
type: Decision Record
id: ADR-0027
title: "ADR 0027: Add New Signing Schemes Through Non-Exhaustive Signature Boundaries"
description: "Future signature families land additively through non-exhaustive signing and signature enums."
status: Accepted
date: 2026-04-27
last_reviewed: 2026-06-25
authors: ["0xSymbiotic"]
tags: [signing, signatures, compatibility, eip1271, eip7212]
related: [ADR-0014, ADR-0022, ADR-0024, ADR-0052]
timestamp: 2026-06-25T00:00:00Z
---

# ADR 0027: Add New Signing Schemes Through Non-Exhaustive Signature Boundaries

## Decision

Future signature families land additively through non-exhaustive signing and
signature enums. Existing ECDSA variants, wire spellings, and recovery-byte
behavior stay stable. New schemes get scheme-keyed normalization and
verification paths rather than widening the ECDSA normalizer.

Verifier-only or contract-mediated schemes use the EIP-1271 verification path
until protocol support requires a dedicated typed variant. Consumers matching on
`SigningScheme` or `Signature` must keep wildcard arms because these enums are
explicitly open to future protocol-side signature forms.

When cowprotocol upstream specifies a post-quantum signing scheme, cow-rs
absorbs it through an additive ADR and focused audit before exposing a stable
SDK-owned type.

## Why

Post-quantum and passkey-style signatures differ from recoverable ECDSA in
size, encoding, verification location, and key material. Treating those schemes
as ECDSA-shaped byte arrays would weaken validation and make the existing
Solidity-compatible recovery-byte contract ambiguous. The current
non-exhaustive boundaries let the SDK grow without reassigning old variants or
creating a breaking match exhaustiveness change for downstream code.

## Must Remain True

- Public surface: existing ECDSA variants and orderbook signing-scheme wire
  values remain stable; additional schemes are additive variants or leaf-local
  capability types until the protocol wire contract is reviewed.
- Runtime and support: ECDSA normalization remains specific to 65-byte
  recoverable signatures and the `27` / `28` recovery-byte range.
- Validation and review: new schemes require scheme-specific tests for
  encoding, normalization, and verification routing, plus parity-scope review
  for the contracts, signing, and orderbook surfaces they touch.
- Cost: downstream exhaustive matches must already include wildcard handling;
  adding a scheme is not a license to reinterpret legacy ECDSA payloads.

## Alternatives Rejected

- Reuse the ECDSA signature variant for every future scheme: this hides scheme
  identity and makes invalid lengths or recovery semantics harder to reject.
- Replace the signing enums with opaque strings: this weakens typed validation
  and loses compile-time visibility for supported variants.
- Promise specific post-quantum protocol support before the upstream wire
  contract exists: the SDK can preserve extension space without overclaiming
  protocol readiness.

## Links

- [Forward-compatible public surfaces principle](../principles/forward-compatible-public-surfaces.md) — the general enum and struct openness doctrine; this ADR instantiates it for signing.
- [Parity scope source lock](../guides/parity.md#source-lock)
- [Parity scope surface boundaries](../guides/parity.md#surface-matrix)
- [Parity matrix signing and contract rows](../guides/parity.md#surface-matrix)
- [Contracts signature boundary](../../crates/contracts/src/signature.rs)
- [Orderbook signing scheme boundary](../../crates/orderbook/src/types/enums.rs)
- [EIP-1271 signature provider](../../crates/signing/src/eip1271/provider.rs)

**Proven by:**

- [ECDSA Signature Normalization Audit](../audit/ecdsa-signature-normalization-audit.md)
- [EIP-1271 Verification Cache Audit](../audit/eip1271-verification-cache-audit.md)
- `xtask/src/policy/check_enum_policy.rs` (run via `cargo check-enum-policy`),
  the syn-based gate that asserts the `#[non_exhaustive]` marker on every
  signature-family enum classified in `.github/config/enum-policy.yaml`
- `crates/contracts/tests/ui/non_exhaustive_external_match.rs`
- `.github/config/enum-policy.yaml` entries classifying the contracts
  `SigningScheme`, contracts `Signature`, and orderbook `SigningScheme`
  enums as `upstream-growing`
