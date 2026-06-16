# Signer Error Classification Audit

Status: Current
Last reviewed: 2026-06-16
Owning surface: `cow-sdk-core`, `cow-sdk-signing`, `cow-sdk-alloy-signer`, `cow-sdk-alloy`
Refresh trigger: any new signer crate, any new variant on `cow_sdk_alloy_signer::SignerError` or `AlloyClientError`, any change to `cow_sdk_core::UserRejection`'s method set, or any change to `SigningError::SignerRejection`'s field set
Related docs:
- [ADR 0053](../adr/0053-typed-signer-rejection-classification.md)
- [ADR 0025](../adr/0025-workspace-url-redaction-convention.md)
- [PROP-SIG-007](../../PROPERTIES.md)

## Scope

This audit covers:

- The `cow_sdk_core::UserRejection` trait and its `user_rejection_code` method
- Every signer-error type that implements `cow_sdk_core::UserRejection`
  (`cow_sdk_alloy_signer::SignerError`,
  `AlloyClientError`, the test mocks in `cow-sdk-signing`,
  and the trading native example)
- The `cow_sdk_signing::signer_error` routing helper plus
  the `cow_sdk_signing::SigningError::SignerRejection` variant
- The cross-crate propagation path through
  `cow_sdk_signing::sign_order` and
  `sign_order_cancellation*`

It does not cover any non-signing classification surface (transport
classes, provider error classes, contracts errors).

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| `UserRejection` trait | Exposes only the EIP-1193 numeric code; never an implementer-controlled string | Conforms |
| `Some(code)` contract | A typed rejection variant returns the carried EIP-1193 code; every non-rejection variant returns `None` | Conforms |
| `cow_sdk_alloy_signer::SignerError` impl | Every variant returns `None` because local-key signing never produces EIP-1193 rejections | Conforms |
| `AlloyClientError` impl | Every variant returns `None`; umbrella adapter never routes wallet prompts | Conforms |
| `signer_error` helper | Routes through the trait, emitting `SignerRejection` only when the trait returns `Some(_)` | Conforms |
| `SigningError::SignerRejection` | Display renders `user rejected {label} ({code})`; fields are static label plus numeric code only | Conforms |
| Redaction posture | `SigningError::Signer` redacted path preserved for every non-rejection failure; redaction contract sweep covers both variants | Conforms |

## Current Contract

### Trait surface

`cow_sdk_core::UserRejection::user_rejection_code` returns
`Option<i32>`. The default returns `None` so an implementer adopts
the trait with a one-line `impl` for signers that never represent
EIP-1193 rejections. The trait deliberately exposes only the
numeric code; implementers must not return strings, free-text
labels, or any wallet-supplied content.

### Rejection routing

The `cow_sdk_signing::order_signing::signer_error` helper consumes
the upstream error by value, calls `user_rejection_code`, and
returns either:

- `SigningError::SignerRejection { label, code }` when the trait
  returned `Some(code)`. `label` is the static operation label
  derived from the helper call site
  (`"typed-data signature"`, `"message signature"`, or
  `"signing request"`).
- `SigningError::Signer { operation, message: Redacted<String> }`
  otherwise. The redacted `Display` from the upstream error is
  carried verbatim so the workspace redaction convention
  ([ADR 0025](../adr/0025-workspace-url-redaction-convention.md))
  stays intact.

### Surface invariant for downstream consoles

`SigningError::SignerRejection`'s `Display` renders
`user rejected {label} ({code})`. A downstream JavaScript or
TypeScript console can scan for the `(NNNN)` parenthesised code to
look up the EIP-1193 label table; the static operation label gives
downstream renderers a stable substring (`"user rejected
typed-data signature"`) for an `errorText` panel
without exposing any wallet-controlled message text.

## Evidence

Primary implementation points:

- `crates/core/src/traits/signer.rs` (trait + courtesy impls for
  `String`, `&str`, `core::convert::Infallible`)
- `crates/core/src/lib.rs` (public re-export)
- `crates/signing/src/errors.rs` (`SignerRejection` variant)
- `crates/signing/src/order_signing.rs` (`signer_error` routing
  helper plus the `signer_operation_label` mapping)
- `crates/signing/src/cancellation.rs` (bound propagation on the
  cancellation helpers)
- `crates/alloy-signer/src/error.rs` (no-op classification)
- `crates/alloy/src/error.rs` (no-op classification)

Primary regression coverage:

- `crates/alloy-signer/tests/signer_error_trait_contract.rs`
- `crates/alloy/tests/signer_error_trait_contract.rs`
- `crates/signing/src/order_signing.rs::signer_error_tests`
  (helper-routing unit tests)
- `tests/signer_rejection_propagation_invariant.rs` (workspace
  end-to-end propagation through `sign_order`)
- `crates/sdk/tests/error_redaction_contract.rs` (redaction sweep
  including `SignerRejection`)

Validation surface:

```text
cargo test -p cow-sdk-alloy-signer --test signer_error_trait_contract
cargo test -p cow-sdk-alloy --test signer_error_trait_contract
cargo test -p cow-sdk-signing --lib signer_error_tests
cargo test -p cow-rs-workspace-tests --test signer_rejection_propagation_invariant
cargo test -p cow-sdk --test error_redaction_contract
```
