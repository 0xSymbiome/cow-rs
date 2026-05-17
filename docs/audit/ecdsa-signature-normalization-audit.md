# ECDSA Signature Normalization Audit

Status: Current
Last reviewed: 2026-05-17
Owning surface: `cow_sdk_contracts::normalized_ecdsa_signature` and `Signature::recover_ecdsa_address`
Refresh trigger: Changes to the accepted ECDSA recovery-byte set, the
65-byte length contract, the `InvalidSignatureLength` or
`InvalidSignatureRecoveryByte` typed error variants, the signing
helpers and recovery helpers that route recoverable signatures through
this normalizer, or the canonical keccak256 invocation backend used by
the EIP-191 prehash builder
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

`cow_sdk_contracts::normalized_ecdsa_signature` parses the input hex,
validates the trailing recovery byte against the COW-accepted set
`{0, 1, 27, 28}` per ADR 0022, then delegates parity normalization to
`alloy_primitives::Signature::from_raw` plus `Signature::as_bytes`
which emits the canonical `r || s || (27 + y_parity)` byte layout.
The pre-validation step preserves the typed
`ContractsError::InvalidSignatureRecoveryByte` rejection for inputs
outside the COW-accepted set (the alloy primitive itself accepts
EIP-155 encoded `v` values starting at 35, a wider input surface than
the ADR 0022 contract permits). The
`crates/contracts/tests/v_normalization_contract.rs` fixture-driven
parity contract pins the byte mapping against
`parity/fixtures/ecdsa/v_normalization.json`. The
`Signature::recover_ecdsa_address` helper continues to delegate
secp256k1 recovery to the `alloy-primitives` 1.5 recovery API as
documented in ADR 0022.

## Scope

This audit covers:

- the `cow_sdk_contracts::normalized_ecdsa_signature` helper
- the additive `ContractsError::InvalidSignatureLength` and
  `ContractsError::InvalidSignatureRecoveryByte` variants
- the signing helpers that consume the contracts-boundary normalizer
  before emitting ECDSA bytes
- the `Signature::recover_ecdsa_address` helper for EIP-712 and
  `eth_sign` digest recovery
- the `Signature::declared_address` helper for variants that carry an
  address without ECDSA recovery
- the parity fixture and fuzz target that pin the reviewed byte-mapping
  contract

It does not cover order hashing or EIP-1271 on-chain verification beyond
confirming that EIP-1271 remains routed through
`cow_sdk_contracts::verify::verify_eip1271_signature_async`.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Input contract | The helper accepts only 65-byte hex signatures and canonicalizes the reviewed `v` set `{0, 1, 27, 28}` onto `{27, 28}` | Conforms |
| Error surface | Length mismatch and unsupported trailing bytes fail through dedicated typed `ContractsError` variants | Conforms |
| Downstream signing path | Contracts and signing helpers emit normalized recoverable signatures before they reach on-chain verification boundaries | Conforms |
| ECDSA recovery path | `Signature::recover_ecdsa_address` recovers only ECDSA variants, uses the supplied digest for EIP-712, and applies the EIP-191 32-byte digest prehash for `eth_sign` | Conforms |
| Declared address path | `Signature::declared_address` returns the verifier for EIP-1271, the owner for pre-sign, and `None` for ECDSA variants whose owner must be recovered cryptographically | Conforms |
| Regression depth | Curated tests, pinned parity cases, and the dedicated fuzz target cover the canonicalization and rejection matrix | Conforms |

## Current Contract

### Signature Boundary

`normalized_ecdsa_signature` lives in
`crates/contracts/src/signature.rs` and is the canonical helper for
recoverable ECDSA signature normalization on the contracts boundary. The
helper first validates the hex envelope, then decodes the payload into
bytes and requires an exact 65-byte signature shape. It preserves the
first 64 bytes (`r || s`) unchanged and rewrites only the trailing
recovery byte: `0` and `27` normalize to `27`; `1` and `28` normalize
to `28`.

### Typed Failure Surface

The length and unsupported-byte failures are explicit on
`ContractsError`. Non-65-byte payloads fail through
`InvalidSignatureLength { actual }`. Any trailing byte outside
`{0, 1, 27, 28}` fails through
`InvalidSignatureRecoveryByte { value }`. Hex-prefix and hex-decode
errors keep flowing through the pre-existing typed error variants, so
callers can distinguish envelope failures from ECDSA-shape failures.

### Downstream Callers

`crates/signing/src/order_signing.rs` routes recoverable signatures
through the contracts-boundary normalizer before building EIP-1271
payloads and before returning `SigningResult` from the public signing
helpers. That keeps the canonicalization rule local to one reviewed
helper rather than re-encoding it at every call site.

### ECDSA Recovery

`Signature::recover_ecdsa_address` first normalizes the recoverable ECDSA
signature, then delegates secp256k1 recovery to the `alloy-primitives`
1.5 recovery API. EIP-712 signatures recover against the supplied
`Hash32` digest directly. `eth_sign` signatures recover against
`keccak256("\x19Ethereum Signed Message:\n32" || digest_bytes)`, matching
the 32-byte digest message shape used by CoW order signing. Non-ECDSA
variants fail through `ContractsError::SignatureSchemeNotEcdsa`.

`Signature::declared_address` keeps the non-ECDSA owner surface explicit:
EIP-1271 returns its verifier contract, pre-sign returns its owner, and
ECDSA returns `None` because the signer is recovered cryptographically.
EIP-1271 smart-account verification remains on the existing
`verify_eip1271_signature_async` path in `cow_sdk_contracts::verify`.

### Review Evidence

The curated contract suite at
`crates/contracts/tests/signature_contract.rs` pins the exact accepted
and rejected boundary cases, including lowercase output, `0` / `1`
canonicalization, `27` / `28` preservation, wrong-length rejection, and
hex-envelope failures. It also pins EIP-712 recovery, `eth_sign` digest
recovery, and non-ECDSA rejection through
`SignatureSchemeNotEcdsa`. The shared property test at
`crates/contracts/tests/property_contract.rs` keeps the EIP-1271 payload
codec green under the stricter 65-byte ECDSA contract. The signing
parity harness reads the dedicated normalization cases from
`parity/fixtures/signing.json`, and the fuzz target
`fuzz/fuzz_targets/fuzz_ecdsa_v_normalization.rs` asserts the canonical
mapping on arbitrary 65-byte inputs. The recovery fuzz target
`fuzz/fuzz_targets/fuzz_recover_ecdsa_address.rs` asserts that recovery
either returns a valid 20-byte address or a reviewed typed failure.

## Evidence

Primary implementation points:

- `crates/contracts/src/signature.rs`
- `crates/contracts/src/errors.rs`
- `crates/signing/src/order_signing.rs`

Primary regression coverage:

- `crates/contracts/tests/signature_contract.rs`
- `crates/contracts/tests/v_normalization_contract.rs`
- `crates/contracts/tests/property_contract.rs::signature_codecs_preserve_verifier_and_payload_bytes`
- `crates/signing/tests/parity_contract.rs::parity_fixture_cases_hold`
- `parity/fixtures/ecdsa/v_normalization.json`
- `fuzz/fuzz_targets/fuzz_ecdsa_v_normalization.rs`
- `fuzz/fuzz_targets/fuzz_recover_ecdsa_address.rs`

Validation surface:

```text
cargo test -p cow-sdk-contracts --test signature_contract
cargo test -p cow-sdk-contracts --test v_normalization_contract
cargo test -p cow-sdk-contracts --test property_contract
cargo test -p cow-sdk-signing --test parity_contract
cargo +nightly fuzz build --fuzz-dir fuzz fuzz_ecdsa_v_normalization
cargo +nightly fuzz run --fuzz-dir fuzz fuzz_ecdsa_v_normalization -- -runs=65536
cargo +nightly fuzz build --fuzz-dir fuzz fuzz_recover_ecdsa_address
cargo +nightly fuzz run --fuzz-dir fuzz fuzz_recover_ecdsa_address -- -runs=65536
```
