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

## Note on canonical primitive layer

The EIP-191 prehash builder
(`cow_sdk_contracts::signature::eth_sign_digest_prehash`) delegates to
`alloy_primitives::eip191_hash_message`, which assembles the canonical
`"\x19Ethereum Signed Message:\n" || ascii_len(msg) || msg` preimage
and hashes it through `alloy_primitives::keccak256`. For a 32-byte
digest input the alloy primitive produces the identical 60-byte
preimage and identical keccak output as the hand-rolled assembler.

`cow_sdk_contracts::RecoverableSignature` is the canonical
contracts-boundary recoverable-signature value. The closed construction
surface (`RecoverableSignature::parse_hex` and
`RecoverableSignature::parse_bytes`) validates the trailing recovery
byte against the COW-accepted set `{0, 1, 27, 28}` per ADR 0022,
reduces the accepted byte to a parity bool, and hands the parity bit
to `alloy_primitives::Signature::from_bytes_and_parity`. The canonical
output bytes emerge from `alloy_primitives::Signature::as_bytes`, which
writes the legacy `r || s || (27 + y_parity)` layout by construction.
The strict cow accept set is a proper subset of alloy's parity input
range — the alloy primitive itself accepts EIP-155 chain-encoded
`v >= 35`, a wider surface than the ADR 0022 contract permits. The
pre-validation guarantees the typed
`ContractsError::InvalidSignatureRecoveryByte` rejection for every
EIP-155 v value and for every other byte outside the canonical set.

The `crates/contracts/tests/v_normalization_contract.rs` fixture-driven
parity contract pins both the accept and rejection rows in
`parity/fixtures/ecdsa/v_normalization.json`. The
`Signature::recover_ecdsa_address` helper on the scheme-tagged
`Signature` enum delegates through
`RecoverableSignature::parse_hex(...)?.recover(...)` and reaches
secp256k1 recovery through the `alloy-primitives` 1.5 API.

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

`RecoverableSignature` lives in `crates/contracts/src/signature.rs` and
is the canonical typestate for recoverable ECDSA signatures on the
contracts boundary. Construction is closed at `parse_hex` (for
hex-wire callers) and `parse_bytes` (for byte-wire callers). Both
validate the hex envelope or 65-byte shape, reduce the trailing
recovery byte against the ADR 0022 accept set `{0, 1, 27, 28}` to a
parity bool, and hand the parity to
`alloy_primitives::Signature::from_bytes_and_parity`. The legacy
`r || s || (27 + y_parity)` byte layout emerges from
`alloy_primitives::Signature::as_bytes` by construction.

### Typed Failure Surface

The length and unsupported-byte failures are explicit on
`ContractsError`. Non-65-byte payloads fail through
`InvalidSignatureLength { actual }`. Any trailing byte outside
`{0, 1, 27, 28}` fails through
`InvalidSignatureRecoveryByte { value }`, including the EIP-155
chain-encoded range `35..=255` that the wider alloy
parity-normalization input surface would otherwise admit. Hex-prefix
and hex-decode errors keep flowing through the pre-existing typed
error variants, so callers can distinguish envelope failures from
ECDSA-shape failures.

### Downstream Callers

`crates/signing/src/order_signing.rs` routes recoverable signatures
through `RecoverableSignature::parse_hex` before building EIP-1271
payloads and before returning `SigningResult` from the public signing
helpers. The canonicalization rule lives on one reviewed typestate
rather than being re-encoded at every call site.

### ECDSA Recovery

`RecoverableSignature::recover(digest, scheme)` selects the digest
preimage by scheme — EIP-712 recovers against the supplied `Hash32`
digest directly; `eth_sign` recovers against
`keccak256("\x19Ethereum Signed Message:\n32" || digest_bytes)`,
matching the 32-byte digest message shape used by CoW order signing —
and reaches secp256k1 recovery through the `alloy-primitives` 1.5
recovery API. Non-ECDSA scheme variants fail through
`ContractsError::SignatureSchemeNotEcdsa`. The scheme-tagged
`Signature::recover_ecdsa_address` enum method delegates through
`RecoverableSignature::parse_hex(...)?.recover(...)`.

`Signature::declared_address` keeps the non-ECDSA owner surface explicit:
EIP-1271 returns its verifier contract, pre-sign returns its owner, and
ECDSA returns `None` because the signer is recovered cryptographically.
EIP-1271 smart-account verification remains on the existing
`verify_eip1271_signature_cached` path in `cow_sdk_contracts::verify`.

### Compact Form and Malleability Posture

`RecoverableSignature::to_erc2098` returns the packed 64-byte
ERC-2098 representation; `parse_erc2098` reconstructs a
`RecoverableSignature` from the same form. Both delegate to the alloy
primitive's `as_erc2098` / `from_erc2098`. The `s` component is
normalized to low-s during packing so the parity bit fits in the high
bit of `s`.

`RecoverableSignature::canonicalized_low_s` exposes BIP-62 low-s
canonicalisation as an opt-in operation. The orderbook accepts both
low-s and high-s recoverable signatures today, so this canonicalisation
is **not** applied at parse time — callers opt in when their downstream
invariants require a uniquely-shaped signature.

### Review Evidence

The curated contract suites at
`crates/contracts/tests/signature_contract.rs` and
`crates/contracts/tests/recoverable_signature_contract.rs` pin the
exact accepted and rejected boundary cases, including lowercase
output, `0` / `1` canonicalization, `27` / `28` preservation,
wrong-length rejection, EIP-155 boundary rejection, hex-envelope
failures, EIP-712 recovery, `eth_sign` digest recovery, non-ECDSA
rejection through `SignatureSchemeNotEcdsa`, ERC-2098 round-trip, and
the opt-in low-s canonicalisation. The shared property test at
`crates/contracts/tests/property_contract.rs` keeps the EIP-1271
payload codec green under the stricter 65-byte ECDSA contract. The
`crates/contracts/tests/v_normalization_contract.rs` parity contract
reads the dedicated normalization accept rows and the EIP-155 rejection
rows from `parity/fixtures/ecdsa/v_normalization.json`. The fuzz target
`fuzz/fuzz_targets/fuzz_ecdsa_v_normalization.rs` asserts the canonical
mapping on arbitrary 65-byte inputs; the fuzz target
`fuzz/fuzz_targets/fuzz_recoverable_signature_differential.rs`
asserts that the cow rejection set is a strict refinement of alloy's
parity-normalization rejection set. The recovery fuzz target
`fuzz/fuzz_targets/fuzz_recover_ecdsa_address.rs` asserts that recovery
either returns a valid 20-byte address or a reviewed typed failure.

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
cargo +nightly fuzz build --fuzz-dir fuzz fuzz_ecdsa_v_normalization
cargo +nightly fuzz run --fuzz-dir fuzz fuzz_ecdsa_v_normalization -- -runs=65536
cargo +nightly fuzz build --fuzz-dir fuzz fuzz_recoverable_signature_parse_hex
cargo +nightly fuzz run --fuzz-dir fuzz fuzz_recoverable_signature_parse_hex -- -runs=65536
cargo +nightly fuzz build --fuzz-dir fuzz fuzz_recoverable_signature_differential
cargo +nightly fuzz run --fuzz-dir fuzz fuzz_recoverable_signature_differential -- -runs=65536
cargo +nightly fuzz build --fuzz-dir fuzz fuzz_eip1271_signature_data_codec
cargo +nightly fuzz run --fuzz-dir fuzz fuzz_eip1271_signature_data_codec -- -runs=65536
cargo +nightly fuzz build --fuzz-dir fuzz fuzz_recover_ecdsa_address
cargo +nightly fuzz run --fuzz-dir fuzz fuzz_recover_ecdsa_address -- -runs=65536
```
