# ECDSA Signature Normalization Audit

Status: Current
Last reviewed: 2026-05-28
Owning surface: `cow_sdk_contracts::RecoverableSignature` and
`Signature::recover_ecdsa_address`
Refresh trigger: Changes to the accepted ECDSA recovery-byte set, the
65-byte length contract, the `InvalidSignatureLength` or
`InvalidSignatureRecoveryByte` typed error variants, the
`RecoverableSignature` construction surface, the signing helpers and
recovery helpers that route recoverable signatures through this
typestate, the ERC-2098 compact-form bridge, the opt-in low-s
canonicalisation, or the canonical keccak256 invocation backend used
by the EIP-191 prehash builder
Related docs:
- [ADR 0022](../adr/0022-ecdsa-signature-v-normalization.md)
- [ADR 0027](../adr/0027-post-quantum-signing-absorption-plan.md)

## Scope

This audit covers:

- the `cow_sdk_contracts::RecoverableSignature` typestate (closed
  construction through `parse_hex` / `parse_bytes`, canonical output
  through `to_bytes` / `to_hex_string`, scheme-bundled `recover`,
  opt-in `canonicalized_low_s`, and the ERC-2098 compact-form bridge)
- the additive `ContractsError::InvalidSignatureLength` and
  `ContractsError::InvalidSignatureRecoveryByte` variants
- the signing helpers that consume the contracts-boundary typestate
  before emitting ECDSA bytes
- the `Signature::recover_ecdsa_address` helper for EIP-712 and
  `eth_sign` digest recovery
- the `Signature::declared_address` helper for variants that carry an
  address without ECDSA recovery
- the parity fixtures and fuzz targets that pin the reviewed
  byte-mapping contract and the strict-rejection contract against
  the wider alloy parity-normalization input surface

It does not cover order hashing or EIP-1271 on-chain verification beyond
confirming that EIP-1271 remains routed through
`cow_sdk_contracts::verify::verify_eip1271_signature_cached`.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Input contract | `RecoverableSignature::parse_bytes` accepts only 65-byte payloads and reduces the reviewed `v` set `{0, 1, 27, 28}` onto `{27, 28}` through alloy `from_bytes_and_parity` after a strict v-set guard | Conforms |
| Typestate proof | Holding a `RecoverableSignature` is a compile-time proof that the ADR 0022 input contract was satisfied; construction is closed at `parse_hex` and `parse_bytes` | Conforms |
| Error surface | Length mismatch and unsupported trailing bytes fail through dedicated typed `ContractsError` variants; the cow rejection set is a proper superset of the alloy parity-normalization rejection set | Conforms |
| Downstream signing path | Contracts and signing helpers route recoverable signatures through the `RecoverableSignature` typestate before they reach on-chain verification boundaries | Conforms |
| ECDSA recovery path | `RecoverableSignature::recover` and the scheme-tagged `Signature::recover_ecdsa_address` recover only ECDSA variants, use the supplied digest for EIP-712, and apply the EIP-191 32-byte digest prehash for `eth_sign` | Conforms |
| Declared address path | `Signature::declared_address` returns the verifier for EIP-1271, the owner for pre-sign, and `None` for ECDSA variants whose owner must be recovered cryptographically | Conforms |
| Compact form bridge | `RecoverableSignature::to_erc2098` / `parse_erc2098` round-trip the ERC-2098 64-byte form through the same backing alloy `Signature` value | Conforms |
| Low-s posture | `RecoverableSignature::canonicalized_low_s` is opt-in defense in depth; not applied at parse time so the orderbook's accepted input set is preserved | Conforms |
| Regression depth | Curated tests, pinned parity accept and rejection rows, and the differential fuzz target against alloy cover the canonicalization, rejection, and strict-superset rejection matrices | Conforms |

## Current Contract

### Signature Boundary

`RecoverableSignature` (`crates/contracts/src/signature.rs`) is the canonical
typestate for recoverable ECDSA signatures on the contracts boundary.
Construction is closed at `parse_hex` and `parse_bytes`: both validate the
65-byte shape, reduce the trailing recovery byte against the ADR 0022 accept set
`{0, 1, 27, 28}` to a parity bit, and emit the legacy `r || s || (27 + y_parity)`
layout through the backing `alloy_primitives::Signature`. Holding a
`RecoverableSignature` is a compile-time proof the input contract was satisfied.

### Typed Failure Surface

Non-65-byte payloads fail through `ContractsError::InvalidSignatureLength`. Any
trailing byte outside `{0, 1, 27, 28}` fails through `InvalidSignatureRecoveryByte`,
including the EIP-155 chain-encoded range that the wider alloy parity input surface
would otherwise admit. Hex-envelope errors keep flowing through their pre-existing
typed variants, distinct from ECDSA-shape failures.

### Downstream Callers

`crates/signing/src/order_signing.rs` routes recoverable signatures through
`RecoverableSignature::parse_hex` before building EIP-1271 payloads and before
returning from the public signing helpers, so the canonicalization rule lives on
one reviewed typestate rather than at every call site.

### ECDSA Recovery

`RecoverableSignature::recover(digest, scheme)` recovers against the supplied
`Hash32` for EIP-712 and against the EIP-191 32-byte digest prehash for
`eth_sign`, reaching secp256k1 recovery through `alloy-primitives`. Non-ECDSA
variants fail through `ContractsError::SignatureSchemeNotEcdsa`. The scheme-tagged
`Signature::recover_ecdsa_address` delegates through
`RecoverableSignature::parse_hex(...)?.recover(...)`. `Signature::declared_address`
returns the verifier for EIP-1271, the owner for pre-sign, and `None` for ECDSA
variants whose signer must be recovered cryptographically; EIP-1271 verification
remains on `verify_eip1271_signature_cached` in `cow_sdk_contracts::verify`.

### Compact Form and Malleability Posture

`RecoverableSignature::to_erc2098` / `parse_erc2098` round-trip the packed 64-byte
ERC-2098 form through the same backing alloy `Signature`.
`RecoverableSignature::canonicalized_low_s` exposes BIP-62 low-s canonicalisation
as opt-in defense in depth; it is **not** applied at parse time, so the orderbook's
accepted input set (both low-s and high-s) is preserved.

## Evidence

Primary implementation points:

- `crates/contracts/src/signature.rs`
- `crates/contracts/src/errors.rs`
- `crates/signing/src/order_signing.rs`

Primary regression coverage:

- `crates/contracts/tests/signature_contract.rs`
- `crates/contracts/tests/recoverable_signature_contract.rs`
- `crates/contracts/tests/v_normalization_contract.rs`
- `crates/contracts/tests/property_contract.rs::signature_codecs_preserve_verifier_and_payload_bytes`
- `parity/fixtures/ecdsa/v_normalization.json`
- `fuzz/fuzz_targets/fuzz_ecdsa_v_normalization.rs`
- `fuzz/fuzz_targets/fuzz_recoverable_signature_parse_hex.rs`
- `fuzz/fuzz_targets/fuzz_recoverable_signature_differential.rs`
- `fuzz/fuzz_targets/fuzz_eip1271_signature_data_codec.rs`
- `fuzz/fuzz_targets/fuzz_recover_ecdsa_address.rs`

Validation surface:

```text
cargo test -p cow-sdk-contracts --test signature_contract
cargo test -p cow-sdk-contracts --test recoverable_signature_contract
cargo test -p cow-sdk-contracts --test v_normalization_contract
cargo test -p cow-sdk-contracts --test property_contract
cargo test -p cow-sdk-signing --test parity_contract
# build and run each fuzz target above, e.g.:
cargo +nightly fuzz run --fuzz-dir fuzz fuzz_ecdsa_v_normalization -- -runs=65536
```
