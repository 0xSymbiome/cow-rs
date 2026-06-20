# ADR 0068: Typed-Data Signing Is Payload-Only At The Signer Seam

- Status: Accepted
- Date: 2026-06-11
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: core, signer, eip712, traits
- Related: [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md), [ADR 0059](0059-hash-concrete-orderdata-directly.md), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md), [ADR 0035](0035-alloy-provider-adapter.md), [ADR 0045](0045-async-signer-trait-narrowing.md)

## Decision

`cow_sdk_core::Signer` and the narrow `cow_sdk_core::TypedDataSigner`
capability take the canonical EIP-712 typed-data payload:
`sign_typed_data_payload(&TypedDataPayload)` is the single required typed-data
method. The payload carries the domain, the full types map, the primary-type
name, and the message — everything a backend needs to compute the canonical
EIP-712 digest.

Field-based signing — a `(domain, fields, message)` triple — is not a trait
obligation. The former wallet-compatibility carve-out (a
`sign_typed_data_compatibility` helper that narrowed a field-based triple into a
`TypedDataPayload` for legacy field-based wallet layouts) has been removed along
with the native browser-wallet crate it lived on. JavaScript and TypeScript
consumers reach their wallet through the `cow-sdk-wasm` EIP-1193 request callback
(ADR 0040), which carries the canonical typed-data payload directly.

## Why

A `(domain, fields, message)` triple cannot carry the primary-type name or
nested struct type definitions, so it cannot express the canonical EIP-712
digest for an arbitrary payload: the backend must either guess a placeholder
primary type or reject the call. A trait method that can only produce a
protocol-meaningless digest for the general case fails the doctrine of
ADR 0059, which removed
compatibility shims that produced protocol-incorrect digests. Keeping one
honest payload method also keeps the trait runtime-neutral per
[ADR 0010](0010-runtime-neutral-async-and-transport-posture.md): every
implementor — local key or JS callback — receives the same
complete signing input rather than a backend-specific reconstruction problem.

The flip is taken pre-release, before the first functional crate release, so
there are no published consumers to migrate.

## Must Remain True

- Public surface: `Signer` and `TypedDataSigner` expose exactly one typed-data
  method, `sign_typed_data_payload(&TypedDataPayload)`. No field-based
  typed-data method returns to either trait.
- Adapters: every shipped `Signer` implementation signs the payload's own
  primary type; no adapter substitutes a placeholder primary type.
- Compatibility boundary: the field-based wallet-compatibility carve-out is
  voided. No trait or inherent helper in the workspace accepts a field-based
  `(domain, fields, message)` triple; wallet protocol compatibility for
  JavaScript and TypeScript consumers is a host-app concern reached through the
  `cow-sdk-wasm` EIP-1193 request callback (ADR 0040), which carries the
  canonical `TypedDataPayload`.
- Validation: primary-type preservation stays covered by the alloy-signer and
  umbrella adapter typed-data tests and the committed EIP-712 reference
  vectors (`PROP-AS-005` in `PROPERTIES.md`).

## Alternatives Rejected

- Keep the field-based method beside the payload method on the traits: every
  implementor must then maintain a second path that cannot express a canonical
  digest for the general case, and the placeholder-primary-type behavior the
  flat path requires contradicts ADR 0059.
- Default-implement field-based signing on the trait by converting into a
  payload: the conversion needs the primary-type name and nested type
  definitions the triple does not carry, so the default would still guess; the
  conversion is only sound for closed, reviewed layouts, which is a
  wallet-protocol concern rather than a trait contract.
- Reintroduce a field-based compatibility helper in `cow-sdk-core`: the
  two-layout compatibility rule is a wallet-protocol concern, and core would gain
  wallet-protocol knowledge it otherwise does not have; the canonical payload
  already crosses the `cow-sdk-wasm` EIP-1193 callback (ADR 0040) intact.

## Links

- [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md)
- [ADR 0040](0040-wallet-provider-callback-boundary-for-js-consumers.md)
- [Properties Registry](../../PROPERTIES.md)
