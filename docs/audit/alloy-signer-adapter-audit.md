# Alloy Signer Adapter Audit

Status: Current
Last reviewed: 2026-05-26
Owning surface: `cow-sdk-alloy-signer` `LocalAlloyKeystoreSigner`, its builder, and its `AsyncSigner` implementation
Refresh trigger: ADR 0038 - `send_transaction` return type clarification, or changes to the signer public API, the `AsyncSigner` trait, typed-data conversion, signature normalization, the inter-crate seam entries consumed by sibling Alloy adapters, cancellation propagation, the workspace Alloy signer pin, or the crate dependency boundary
Related docs:
- [ADR 0036](../adr/0036-alloy-signer-adapter.md)
- [ADR 0038](../adr/0038-transaction-lifecycle-types.md)
- [ADR 0022](../adr/0022-ecdsa-signature-v-normalization.md)
- [ADR 0024](../adr/0024-asyncprovider-asyncsigningprovider-capability-split.md)
- [ADR 0026](../adr/0026-alloy-major-release-absorption-plan.md)
- [Alloy Provider Adapter Audit](alloy-provider-adapter-audit.md)
- [ECDSA Signature Normalization Audit](ecdsa-signature-normalization-audit.md)

## Scope

This audit covers:

- the `LocalAlloyKeystoreSigner` public type and its `AsyncSigner`
  implementation
- the private-key plus chain-id typestate builder and builder error type
- the `AsyncSignerError` and `AsyncSignerErrorClass` surfaces
- conversion from SDK EIP-712 typed-data payloads into Alloy dynamic typed data
- EIP-191 and EIP-712 signature normalization through `cow-sdk-contracts`
- the doc-hidden inter-crate seam that re-exports the typed-data conversion
  and signature normalization helpers for sibling Alloy adapter crates
- cancellation propagation through the signer error type
- dependency boundaries for the local signer crate

It does not cover upstream Alloy internals, provider reads, transaction filling
or submission, browser-wallet behavior, or smart-account signing.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Public API exposure | Documented signer and builder methods expose SDK-owned domain types; upstream Alloy local signer values remain private | Conforms |
| Trait coverage | `LocalAlloyKeystoreSigner` implements `AsyncSigner` and compile-fail tests assert it is not an `AsyncProvider`, `AsyncSigningProvider`, or sync `Signer` | Conforms |
| Builder typestate | `build()` is callable only after private-key source and chain id are selected; externally constructed marker states cannot bypass the builder | Conforms |
| EIP-191 signing | Message signatures match the committed reference vector and recover to the local signer address | Conforms |
| EIP-712 signing | Canonical order typed-data signatures preserve `Order` as the primary type, match the committed reference vector, and recover through the contracts crate | Conforms |
| Legacy typed-data fallback | The flat-fields compatibility method uses `Message` as its placeholder primary type and is distinct from canonical payload signing | Conforms |
| Error and cancellation | Public error classes cover validation, signing, provider-required, unsupported, cancelled, and internal failures with redacted formatting where detail may be sensitive | Conforms |
| Dependency boundary | The crate declares no provider or transport dependency and the resolved normal graph excludes `alloy-provider` | Conforms |

## Current Contract

### Public Surface

`cow-sdk-alloy-signer` exposes `LocalAlloyKeystoreSigner`,
`LocalAlloyKeystoreSignerBuilder`, sealed builder-state marker names,
`LocalAlloyKeystoreSignerBuilderError`, `AsyncSignerError`, and
`AsyncSignerErrorClass`. The signer stores the upstream Alloy private-key
signer in private state and redacts it from debug output.

The crate also exposes a `#[doc(hidden)] pub mod __seam` module so
sibling `cow-rs` Alloy adapter crates can reuse the EIP-712 typed-data
conversion helpers (`cow_typed_data_payload_to_alloy`,
`cow_flat_to_alloy_typed_data`) and the shared signature normalizer
(`alloy_signature_to_hex`) without duplicating the implementation.
Anything inside the seam is not part of the documented consumer API and
is not semver-guaranteed for downstream consumers.

The builder accepts hex or raw 32-byte private keys and a
`cow_sdk_core::SupportedChainId`. Invalid key material returns a typed builder
error that does not echo the input.

### Signer Methods

The adapter implements `get_address`, `sign_message`,
`sign_typed_data_payload`, and the legacy flat `sign_typed_data` path.
`sign_transaction`, `send_transaction`, and `estimate_gas` return
`ProviderRequired` because the local signer does not own provider context.
The `send_transaction` method still has the `Result<TransactionBroadcast, _>`
trait shape; the provider-required result means this leaf never fabricates a
broadcast hash without a provider. The composed Alloy umbrella owns the
provider-backed path and returns the broadcast hash through `*pending.tx_hash()`.

`sign_typed_data_payload` converts the explicit SDK payload into Alloy dynamic
typed data without dropping the payload primary type. The legacy flat path has
no primary-type input, so it synthesizes a typed-data payload with `Message` as
the compatibility placeholder.

### Signature Normalization

All returned ECDSA signatures pass through
`cow_sdk_contracts::normalized_ecdsa_signature`, keeping the local signer
aligned with the shared Solidity-compatible recovery-byte contract.

### Error And Cancellation

`AsyncSignerError` is non-exhaustive and partitions errors into validation,
signing, provider-required, unsupported, cancelled, and internal classes.
Validation, signing, and internal details are redacted in public formatting.

`From<cow_sdk_core::Cancelled>` allows consumer code using
`Cancellable::cancel_with(...).await?` to propagate cancellation through the
signer error type.

### Dependency Boundary

The crate depends on Alloy signer, local signer, primitives, consensus, network,
dynamic ABI, and Solidity type crates needed for local signing and EIP-712
conversion. It does not depend on `alloy-provider` or Alloy transport crates.

## Evidence

Primary implementation points:

- `crates/alloy-signer/src/lib.rs`
- `crates/alloy-signer/src/signer.rs`
- `crates/alloy-signer/src/builder.rs`
- `crates/alloy-signer/src/error.rs`
- `crates/alloy-signer/src/conversion.rs`
- `crates/alloy-signer/Cargo.toml`

Primary regression coverage:

- `crates/alloy-signer/tests/asyncsigner_contract.rs`
- `crates/alloy-signer/tests/eip191_reference_vectors.rs`
- `crates/alloy-signer/tests/eip712_reference_vectors.rs`
- `crates/alloy-signer/tests/redaction_contract.rs`
- `crates/alloy-signer/tests/cancellation_contract.rs`
- `crates/alloy-signer/tests/dependency_boundary_contract.rs`
- `crates/alloy-signer/tests/proptests.rs`
- `crates/alloy-signer/tests/compile_fail.rs`
- `crates/alloy-signer/tests/trybuild/no_async_provider.rs`
- `crates/alloy-signer/tests/trybuild/no_async_signing_provider.rs`
- `crates/alloy-signer/tests/trybuild/no_sync_signer.rs`
- `crates/alloy-signer/tests/trybuild/external_marker_construction_fails.rs`

Validation surface:

```text
cargo fmt --all --check
cargo clippy -p cow-sdk-alloy-signer --all-targets -- -D warnings
cargo test -p cow-sdk-alloy-signer --all-features
cargo test -p cow-sdk-alloy-signer --test compile_fail
RUSTDOCFLAGS="-D warnings" cargo doc -p cow-sdk-alloy-signer --no-deps
cargo check-property-citations
```
