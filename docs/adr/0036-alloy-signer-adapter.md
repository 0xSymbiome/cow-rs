# ADR 0036: Ship A Native Alloy Local Signer Adapter

- Status: Accepted
- Date: 2026-05-06
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: alloy, signer, adapter, native, eip712
- Related: [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md), [ADR 0022](0022-ecdsa-signature-v-normalization.md), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md), [ADR 0026](0026-alloy-major-release-absorption-plan.md), [ADR 0035](0035-alloy-provider-adapter.md)

## Decision

The workspace ships `cow-sdk-alloy-signer` as a native Alloy local-keystore
adapter. `LocalAlloyKeystoreSigner` wraps an Alloy private-key signer internally
and exposes it through `cow_sdk_core::AsyncSigner`.

The crate is a signer leaf, not a provider. It signs EIP-191 messages and
EIP-712 typed-data payloads, preserves explicit typed-data primary types on the
canonical payload path, and routes returned ECDSA signatures through the shared
`cow-sdk-contracts` normalizer. Provider-backed transaction methods return a
typed `ProviderRequired` error because nonce, fee, and transaction context are
outside this crate's authority.

The adapter is native-only. Wasm targets fail at compile time and should use
browser-wallet signing plus consumer-supplied EIP-1193 provider reads.

## Why

Native SDK consumers need a first-party bridge from Alloy local keys to the SDK
signing contract without also depending on provider machinery. Keeping this
bridge in a leaf crate preserves ADR 0024's provider/signer split and keeps the
default SDK surface free of local-keystore dependencies.

The shared signature normalizer from ADR 0022 remains the single recovery-byte
authority. That prevents adapter-local divergence between Alloy's emitted
signature bytes and the Solidity-compatible form used by the contracts crate.

## Must Remain True

- Public surface: documented constructors and signer methods expose SDK-owned
  types, not upstream Alloy provider or transport types.
- Trait coverage: `LocalAlloyKeystoreSigner` implements `AsyncSigner` and does
  not implement `AsyncProvider`, `AsyncSigningProvider`, or sync `Signer`.
- Builder state: `build()` is available only after a private key and chain id
  have both been selected; builder markers remain sealed from external
  construction.
- Runtime support: native local-keystore signing is the only supported runtime.
  Wasm targets fail closed at compile time.
- Signature behavior: message and typed-data signatures are normalized through
  `cow-sdk-contracts`, and canonical typed-data payload signing preserves the
  caller's primary type.
- Validation: tests cover EIP-191 and EIP-712 vectors, primary-type
  preservation, redaction, cancellation, dependency boundaries, compile-fail
  capability exclusions, and property-based recovery checks.

## Alternatives Rejected

- Re-export upstream Alloy signer types directly: this would couple the SDK
  surface to Alloy's signer semver.
- Put signer support into the provider crate: read-only users would pull local
  key dependencies and the capability split from ADR 0024 would be weakened.
- Hand-roll EIP-712 hashing in this crate: the adapter should translate into
  Alloy's dynamic typed-data surface and rely on existing workspace order
  hashing tests for protocol parity.
- Return placeholder transaction signatures: signing incomplete transaction
  payloads without provider context would be misleading and unsafe.

## Stability

The public `AsyncSignerError::from_alloy_signer` constructor is gated
`#[doc(hidden)]` and documented in source as an inter-crate seam constructor.
It exists so sibling adapter crates can lift `alloy_signer::Error` values into
the signer's typed error surface. It is not a semver-stable consumer API and
may change in any minor release.

The documented consumer surface is limited to `LocalAlloyKeystoreSigner`,
`LocalAlloyKeystoreSignerBuilder`, `AsyncSignerError`, and the typestate
markers explicitly exported from `lib.rs`.

## Links

- [Alloy Provider Adapter ADR](0035-alloy-provider-adapter.md)
- [ECDSA Signature Normalization ADR](0022-ecdsa-signature-v-normalization.md)
- [Architecture](../architecture.md)

**Proven by:**

- [Alloy Signer Adapter Audit](../audit/alloy-signer-adapter-audit.md)
