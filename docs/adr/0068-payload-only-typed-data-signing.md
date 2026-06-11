# ADR 0068: Typed-Data Signing Is Payload-Only At The Signer Seam

- Status: Accepted
- Date: 2026-06-11
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: core, signer, eip712, traits
- Related: [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md), [ADR 0023](0023-legacy-compatibility-shim-removal.md), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md), [ADR 0036](0036-alloy-signer-adapter.md), [ADR 0045](0045-async-signer-trait-narrowing.md)

## Decision

`cow_sdk_core::Signer` and the narrow `cow_sdk_core::TypedDataSigner`
capability take the canonical EIP-712 typed-data payload:
`sign_typed_data_payload(&TypedDataPayload)` is the single required typed-data
method. The payload carries the domain, the full types map, the primary-type
name, and the message — everything a backend needs to compute the canonical
EIP-712 digest.

Field-based signing — a `(domain, fields, message)` triple — is not a trait
obligation. Wallet-protocol compatibility for the field-based layouts that
legacy browser-wallet integrations expect lives in `cow-sdk-browser-wallet` as
the inherent `sign_typed_data_compatibility` helper on its signer, which
narrows the triple into a `TypedDataPayload` before delegating to the trait
method.

## Why

A `(domain, fields, message)` triple cannot carry the primary-type name or
nested struct type definitions, so it cannot express the canonical EIP-712
digest for an arbitrary payload: the backend must either guess a placeholder
primary type or reject the call. A trait method that can only produce a
protocol-meaningless digest for the general case fails the doctrine of
[ADR 0023](0023-legacy-compatibility-shim-removal.md), which removed
compatibility shims that produced protocol-incorrect digests. Keeping one
honest payload method also keeps the trait runtime-neutral per
[ADR 0010](0010-runtime-neutral-async-and-transport-posture.md): every
implementor — local key, browser wallet, JS callback — receives the same
complete signing input rather than a backend-specific reconstruction problem.

The flip is taken pre-release, before the first functional crate release, so
there are no published consumers to migrate.

## Must Remain True

- Public surface: `Signer` and `TypedDataSigner` expose exactly one typed-data
  method, `sign_typed_data_payload(&TypedDataPayload)`. No field-based
  typed-data method returns to either trait.
- Adapters: every shipped `Signer` implementation signs the payload's own
  primary type; no adapter substitutes a placeholder primary type.
- Compatibility boundary: field-based signing support is a wallet-protocol
  concern owned by `cow-sdk-browser-wallet`'s inherent
  `sign_typed_data_compatibility` helper, limited to the reviewed CoW order
  and order-cancellation field layouts, and implemented by conversion into a
  `TypedDataPayload`.
- Validation: primary-type preservation stays covered by the alloy-signer and
  umbrella adapter typed-data tests and the committed EIP-712 reference
  vectors (`PROP-AS-005` in `PROPERTIES.md`).

## Alternatives Rejected

- Keep the field-based method beside the payload method on the traits: every
  implementor must then maintain a second path that cannot express a canonical
  digest for the general case, and the placeholder-primary-type behavior the
  flat path requires contradicts ADR 0023.
- Default-implement field-based signing on the trait by converting into a
  payload: the conversion needs the primary-type name and nested type
  definitions the triple does not carry, so the default would still guess; the
  conversion is only sound for closed, reviewed layouts, which is a
  wallet-protocol concern rather than a trait contract.
- Move the compatibility helper into `cow-sdk-core`: the two-layout
  compatibility rule exists for browser-wallet integrations specifically, and
  core would gain wallet-protocol knowledge it otherwise does not have.

## Links

- [ADR 0023](0023-legacy-compatibility-shim-removal.md)
- [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md)
- [ADR 0036](0036-alloy-signer-adapter.md)
- [Properties Registry](../../PROPERTIES.md)
