# ADR 0027: Add New Signing Schemes Through Non-Exhaustive Signature Boundaries

- Status: Accepted
- Date: 2026-04-27
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: signing, signatures, compatibility, eip1271, eip7212
- Related: [ADR 0014](0014-eip1271-verification-cache.md), [ADR 0022](0022-ecdsa-signature-v-normalization.md), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md)

## Decision

Future signature families such as Dilithium, Falcon, SPHINCS+, and secp256r1
through EIP-7212 land additively through non-exhaustive signing and signature
enums. Existing ECDSA variants, wire spellings, and recovery-byte behavior stay
stable. New schemes get scheme-keyed normalization and verification paths rather
than widening the ECDSA normalizer.

Verifier-only or contract-mediated schemes use the EIP-1271 verification path
until protocol support requires a dedicated typed variant. Consumers matching on
`SigningScheme` or `Signature` must keep wildcard arms because these enums are
explicitly open to future protocol-side signature forms.

## Why

Post-quantum and passkey-style signatures differ from recoverable ECDSA in
size, encoding, verification location, and key material. Treating those schemes
as ECDSA-shaped byte arrays would weaken validation and make the existing
Solidity-compatible recovery-byte contract ambiguous. The current non-exhaustive
boundaries let the SDK grow without reassigning old variants or creating a
breaking match exhaustiveness change for downstream code.

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

- [Parity scope source lock](../parity-scope.md#source-lock)
- [Parity scope surface boundaries](../parity-scope.md#surface-boundaries)
- [Parity matrix signing and contract rows](../parity-matrix.md#workspace-parity-map)
- [Contracts signature boundary](../../crates/contracts/src/signature.rs)
- [Orderbook signing scheme boundary](../../crates/orderbook/src/types.rs)
- [Trading EIP-1271 signature provider](../../crates/trading/src/types.rs)

**Proven by:**

- [ECDSA Signature Normalization Audit](../audit/ecdsa-signature-normalization-audit.md)
- [EIP-1271 Verification Cache Audit](../audit/eip1271-verification-cache-audit.md)
