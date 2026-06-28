# ADR 0053: Typed signer rejection classification through the `UserRejection` trait

- Status: Accepted
- Date: 2026-05-19
- Last reviewed: 2026-06-15
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: signing, error-surface, eip-1193, alloy, classification
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0040](0040-wallet-provider-callback-boundary-for-js-consumers.md), [ADR 0017](0017-typed-orderbook-rejection-parser.md), [ADR 0025](0025-workspace-url-redaction-convention.md), [ADR 0045](0045-async-signer-trait-narrowing.md)

## Decision

- `cow_sdk_core` exposes a `UserRejection` trait whose single
  `user_rejection_code(&self) -> Option<i32>` method names the
  EIP-1193 provider error code carried by a user-driven rejection.
  Every other failure class returns `None` and falls through to the
  redacted `cow_sdk_signing::SigningError::Signer` display path.
- Every typed signer error in the workspace implements
  `UserRejection` against its own variants:
  `cow_sdk_alloy_signer::SignerError` and `cow_sdk_alloy::AlloyClientError`
  return `None` for every variant because local-key signing cannot produce an
  EIP-1193 4xxx user rejection. A signer that surfaces EIP-1193 provider error
  codes — for example a host-supplied wallet reached through the `cow-sdk-js`
  typed-data signer callback (ADR 0040) — returns the carried code from its
  user-rejection variant.
- The helpers in `cow-sdk-signing` bound the signer's
  associated error on `fmt::Display + cow_sdk_core::UserRejection`.
  `signer_error` consumes the trait result first: when the code is
  present it emits `SigningError::SignerRejection { label, code }`
  with a static, non-sensitive operation label
  (`"typed-data signature"`, `"message signature"`, or
  `"signing request"`); otherwise it falls back to the existing
  `SigningError::Signer { operation, message: Redacted<String> }`
  variant.

## Why

This is the first **shared classification trait** in the workspace.
The existing convention is per-type classification: every error
type owns its own `class()` accessor returning a
type-specific enum (`cow_sdk_alloy_signer::SignerError::class()` →
`SignerErrorClass`, `AlloyClientError::class()` →
`AlloyClientErrorClass`, `ProviderError::class()` →
`ProviderErrorClass`), and typed-rejection surfaces follow
[ADR 0017](0017-typed-orderbook-rejection-parser.md)'s per-crate
`#[non_exhaustive]` enum plus parser-function shape. Introducing a
new pattern requires a specific justification — the rest of this
section is that justification.

The signing crate's public helpers (`sign_order`,
`sign_order_cancellation`, and their cancellation-batch siblings) are
**generic over `S: TypedDataSigner`** with the associated
`Error` type opaque. The crate cannot pattern-match on a concrete
EIP-1193 signer's `UserRejectedRequest` variant because it never sees
the concrete signer type at any of its call sites. Three
alternatives were considered before adopting the shared trait:

- **Per-type accessor on the concrete signer error only**
  (`user_rejection_code(&self) -> Option<i32>`) plus
  classification at the consumer, keeping the signing crate ignorant
  of rejection classes. Rejected because it pushes the responsibility onto
  every consumer of `sign_*`, defeats the discoverable
  typed-error surface the signing crate already exposes, and
  cannot extend to a future hardware-wallet or transport-bridged
  signer that surfaces EIP-1193 4xxx codes through its own typed
  variant.
- **`std::any::Any` downcast from `S::Error` to a concrete
  signer error type**, performed inside the signing crate.
  Rejected because `Any` requires `'static` (not always satisfied
  by adapter error types that carry a borrowed lifetime), couples
  the signing crate to a specific signer crate it does not depend
  on, and offers no compile-time guarantee that future signer
  error types will participate in classification.
- **Embedding a numeric `code: Option<i32>` field directly on
  `SigningError::Signer`** and discovering it through a
  best-effort string scan of the upstream `Display`. Rejected
  because string-shape coupling has no compile-time signal when
  the upstream `Display` is refactored — exactly the brittleness
  the typed surfaces in [ADR 0017](0017-typed-orderbook-rejection-parser.md)
  were introduced to avoid.

A shared trait is the minimum cross-crate seam that gives the
signing crate a typed view of "is this a user rejection" without
coupling to any specific signer implementation. The trait surface
is intentionally small (one method, returning the public EIP-1193
numeric code only) so the new pattern carries the least possible
surface-area cost while still solving the generic-classification
problem the existing per-type `class()` convention does not.

## Must Remain True

- Public surface: `cow_sdk_core::UserRejection` is a new trait in the
  workspace's public-API perimeter. Signer crates and downstream
  callers that implement `TypedDataSigner` (or any other capability
  trait that routes through the signing helpers) must implement
  `UserRejection` for their associated
  `Error` type. The trait carries a `None` default so adoption is a
  one-line `impl UserRejection for MyError {}` for signers that never
  represent EIP-1193 rejections. Courtesy impls on `String`,
  `&str`, and `core::convert::Infallible` cover the canonical
  test-signer `Error` shapes without forcing a per-test impl.
- Runtime and support: the `SigningError::Signer` redacted path
  stays in place for every non-rejection class, preserving the
  workspace redaction convention from
  [ADR 0025](0025-workspace-url-redaction-convention.md). The new
  `SignerRejection` variant exposes only the static operation
  label and the numeric provider code; no wallet-supplied message
  text crosses the redaction boundary.
- Trait-bound placement: the `S::Error: fmt::Display + UserRejection`
  bound lives on every signing-helper signature in
  `cow-sdk-signing` plus every trading SDK API that forwards an
  upstream signer error. The bound is **not** added to the
  `TypedDataSigner` trait itself; signer adapters that never
  route through the signing crate stay free of the requirement.
- Validation and review: per-crate `signer_error_trait_contract`
  host tests enumerate every variant of every implementer and pin
  the trait result; signing-crate unit tests cover the helper
  routing from `user_rejection_code` through to the
  `SignerRejection` variant and the new variant `Display`; the
  redaction contract sweep continues to exercise
  `SignerRejection` alongside `Signer`. The standing audit at
  `docs/audit/error-classification-audit.md` carries the
  refresh trigger for the next 90-day review.
- Cost: the shared classification trait is the first of its kind
  in the workspace, so any future signer added to the workspace
  must extend the classification surface alongside its typed
  error. Future error-class surfaces that need cross-crate
  visibility may follow this pattern, or stay per-type if the
  generic-bound problem this ADR solves does not apply.

## Alternatives Rejected

- **Per-type accessor only, classify at the consumer**: see *Why*
  above. Loses discoverability through `?` / `map_err` and cannot
  extend to future signers.
- **`std::any::Any` downcast inside the signing crate**: see *Why*
  above. `'static` requirement and lack of compile-time
  participation guarantee.
- **String-shape parsing of the upstream `Display`**: see *Why*
  above. Brittle, no compile-time signal under upstream refactor.
- **Bound `TypedDataSigner::Error: UserRejection` on the trait
  itself**: forces every implementer of the trait to pay the bound
  even if they never route through the signing crate. The current
  decision narrows the requirement to the actual call sites and
  keeps the trait reusable for adapter surfaces that do not need
  rejection classification.

## Links

- [crates/core/src/traits/signer.rs](../../crates/core/src/traits/signer.rs)
- [crates/signing/src/errors.rs](../../crates/signing/src/errors.rs)
- [crates/signing/src/order_signing.rs](../../crates/signing/src/order_signing.rs)
- [crates/alloy-signer/src/error.rs](../../crates/alloy-signer/src/error.rs)
- [crates/alloy/src/error.rs](../../crates/alloy/src/error.rs)
- [docs/audit/error-classification-audit.md](../audit/error-classification-audit.md)
