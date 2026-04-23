# ADR 0022: Canonicalize ECDSA Signature `v` At The Contracts Boundary

- Status: Accepted
- Date: 2026-04-23
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: contracts, signing, ecdsa, normalization, error-typing
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0015](0015-client-side-order-bounds-validator.md), [ADR 0017](0017-typed-orderbook-rejection-parser.md)

## Decision

`cow_sdk_contracts::normalized_ecdsa_signature` is the single
contracts-boundary normalizer for recoverable ECDSA signatures. It
accepts only 65-byte signatures, canonicalizes modern `v = 0` / `1`
inputs onto the legacy Solidity-compatible `27` / `28` range, preserves
already-canonical `27` / `28` inputs byte-for-byte, and rejects every
other trailing byte through the typed
`ContractsError::InvalidSignatureRecoveryByte` variant. Length mismatch
fails through the typed `ContractsError::InvalidSignatureLength`
variant.

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

- Public surface: `normalized_ecdsa_signature(&str) -> Result<String,
  ContractsError>` accepts only hex-encoded 65-byte signatures. The
  helper lowercases the output, preserves `r || s`, maps `v = 0` and
  `27` to `27`, maps `v = 1` and `28` to `28`, rejects any other `v`
  byte through `InvalidSignatureRecoveryByte { value }`, and rejects any
  non-65-byte payload through `InvalidSignatureLength { actual }`.
- Runtime and support: every contracts or signing helper that emits or
  repackages recoverable ECDSA signatures routes through the same
  normalizer, so downstream Solidity verification paths always receive
  the legacy-compatible form. The change is additive: it expands the
  typed error surface but does not widen the accepted signature family
  beyond the four reviewed values.
- Validation and review: curated regression coverage in
  `crates/contracts/tests/signature_contract.rs` pins the accepted and
  rejected boundary cases; the signing parity fixture in
  `parity/fixtures/signing.json` locks the cross-SDK byte mapping; and
  the fuzz target `fuzz/fuzz_targets/fuzz_ecdsa_v_normalization.rs`
  asserts that arbitrary 65-byte inputs either preserve `r || s` with a
  canonical `27` / `28` output or fail through the typed recovery-byte
  rejection.
- Cost: two additive `ContractsError` variants and one stricter
  contracts-boundary helper. No new signing scheme, wire format, or
  on-chain ABI surface is introduced.

## Alternatives Rejected

- Keep `normalized_ecdsa_signature` as a hex-only passthrough: simplest
  implementation, but leaves the Rust SDK divergent from the TypeScript
  SDK and the services backend on a reviewed signing invariant.
- Normalize only in `cow-sdk-signing`: fixes some call sites, but leaves
  other contracts-boundary consumers free to reintroduce the bug and
  weakens the contracts crate as the canonical signature helper surface.
- Silently coerce every trailing byte into the legacy range: avoids an
  error, but destroys typed failure semantics and risks turning malformed
  signatures into different malformed signatures rather than rejecting
  them explicitly.

## Links

- [Architecture](../architecture.md)
- [Verification Guide](../verification-guide.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0015](0015-client-side-order-bounds-validator.md)
- [ADR 0017](0017-typed-orderbook-rejection-parser.md)

**Proven by:**

- [ECDSA Signature Normalization Audit](../audit/ecdsa-signature-normalization-audit.md)
