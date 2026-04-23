# ECDSA Signature Normalization Audit

Status: Current
Last reviewed: 2026-04-23
Owning surface: `cow_sdk_contracts::normalized_ecdsa_signature`
Refresh trigger: Changes to the accepted ECDSA recovery-byte set, the
65-byte length contract, the `InvalidSignatureLength` or
`InvalidSignatureRecoveryByte` typed error variants, or the signing
helpers that route recoverable signatures through this normalizer
Related docs:
- [ADR 0022](../adr/0022-ecdsa-signature-v-normalization.md)
- [ADR 0015](../adr/0015-client-side-order-bounds-validator.md)
- [ADR 0017](../adr/0017-typed-orderbook-rejection-parser.md)

## Scope

This audit covers:

- the `cow_sdk_contracts::normalized_ecdsa_signature` helper
- the additive `ContractsError::InvalidSignatureLength` and
  `ContractsError::InvalidSignatureRecoveryByte` variants
- the signing helpers that consume the contracts-boundary normalizer
  before emitting ECDSA bytes
- the parity fixture and fuzz target that pin the reviewed byte-mapping
  contract

It does not cover full EIP-1271 verification, order hashing, or
non-ECDSA signing schemes.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Input contract | The helper accepts only 65-byte hex signatures and canonicalizes the reviewed `v` set `{0, 1, 27, 28}` onto `{27, 28}` | Conforms |
| Error surface | Length mismatch and unsupported trailing bytes fail through dedicated typed `ContractsError` variants | Conforms |
| Downstream signing path | Contracts and signing helpers emit normalized recoverable signatures before they reach on-chain verification boundaries | Conforms |
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

### Review Evidence

The curated contract suite at
`crates/contracts/tests/signature_contract.rs` pins the exact accepted
and rejected boundary cases, including lowercase output, `0` / `1`
canonicalization, `27` / `28` preservation, wrong-length rejection, and
hex-envelope failures. The shared property test at
`crates/contracts/tests/property_contract.rs` keeps the EIP-1271 payload
codec green under the stricter 65-byte ECDSA contract. The signing
parity harness reads the dedicated normalization cases from
`parity/fixtures/signing.json`, and the fuzz target
`fuzz/fuzz_targets/fuzz_ecdsa_v_normalization.rs` asserts the canonical
mapping on arbitrary 65-byte inputs.

## Evidence

Primary implementation points:

- `crates/contracts/src/signature.rs`
- `crates/contracts/src/errors.rs`
- `crates/signing/src/order_signing.rs`

Primary regression coverage:

- `crates/contracts/tests/signature_contract.rs`
- `crates/contracts/tests/property_contract.rs::signature_codecs_preserve_verifier_and_payload_bytes`
- `crates/signing/tests/parity_contract.rs::parity_fixture_cases_hold`
- `fuzz/fuzz_targets/fuzz_ecdsa_v_normalization.rs`

Validation surface:

```text
cargo test -p cow-sdk-contracts --test signature_contract
cargo test -p cow-sdk-contracts --test property_contract
cargo test -p cow-sdk-signing --test parity_contract
cargo +nightly fuzz build --fuzz-dir fuzz fuzz_ecdsa_v_normalization
cargo +nightly fuzz run --fuzz-dir fuzz fuzz_ecdsa_v_normalization -- -runs=65536
```
