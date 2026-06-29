---
type: Decision Record
id: ADR-0063
title: "ADR 0063: Publish Consumer Test Doubles As The cow-sdk-test Crate"
description: "Consumer-facing test doubles for the SDK public trait seams ship as a published cow-sdk-test crate built only on public APIs."
status: Accepted
date: 2026-06-02
last_reviewed: 2026-06-12
authors: ["0xSymbiotic"]
tags: [testing, crate-boundary, public-api, feature-gating, panic]
related: [ADR-0001, ADR-0033, ADR-0062]
timestamp: 2026-06-12T00:00:00Z
---

# ADR 0063: Publish Consumer Test Doubles As The `cow-sdk-test` Crate

## Decision

Consumer-facing test doubles for the SDK public trait seams ship as a published
`cow-sdk-test` crate built only on public APIs. It provides in-memory recording
doubles for `OrderbookClient`, `Signer`, and `Provider` / `SigningProvider`,
ready-made typed-error constructors, and a one-call wired `Trading` convenience,
so a downstream application can test its CoW integration without a live
orderbook, RPC endpoint, or wallet. The `Signer` double (`MockSigner`) signs
with a public development key and produces real, recoverable signatures, so a
double-driven posting flow clears the client-side owner-recovery gate rather
than failing it. The root facade re-exports it behind an
opt-in `testing` feature as `cow_sdk::testing`, for use from a consumer's
`[dev-dependencies]`.

## Why

Applications integrating the SDK need to assert what their own code sends — that
a swap posts exactly one order, that a rejection is handled — without network or
signing. Those doubles previously lived only inside the examples as copyable
glue. Publishing them as a versioned crate (the `tokio-test` / `tower-test`
idiom) gives consumers a supported surface and, because they are built only on
the public traits, continuously dogfoods that those seams are implementable from
outside the workspace.

## Must Remain True

- Public surface: `cow-sdk-test` is `publish = true` and depends only on
  published cow crates, `async-trait`, and the confined crypto/serde deps named
  below; it never depends on the
  `publish = false` `cow-sdk-test-utils` of ADR 0062, because a published crate
  cannot normal-depend on an unpublished one. It is built strictly on the public
  trait surface — no private or internal APIs.
- Runtime and support: facade exposure is additive and opt-in. The `testing`
  feature re-exports the crate as `cow_sdk::testing` and leaves the default
  facade contract unchanged; the doubles reach a build only through
  `[dev-dependencies]`, so test code cannot enter a production dependency graph.
  Doubles are instance-scoped (ADR 0006) and hold no credentials (ADR 0025).
  The `OrderbookClient` double carries both arms today, target-cfg selected:
  native gets the `Send` `async_trait` and `wasm32` gets the `?Send` variant,
  with no Cargo feature gating the choice.
- Validation and review: as a published crate it is part of the panic-free
  shipped surface of ADR 0033 — canned defaults are built through infallible
  constructors with no `unwrap`/`expect`/`panic!` and no allowlist carve-out.
  The crate's own tests drive a real `Trading` through the doubles to prove they
  satisfy the actual trait contracts.
- Real signing: the `Signer` double (`MockSigner`) produces real, recoverable
  signatures by default. It signs EIP-712 typed data and EIP-191 messages with a
  public development key — the secp256k1 scalar `1`, the canonical key in Alloy's
  `signer-local` tests and the `CoW` services signature-recovery vectors, never a
  secret — emitting the canonical legacy-`v` recoverable form through
  `RecoverableSignature` (ADR 0022), so a signed order recovers to the reported
  address and clears the client-side owner-recovery gate (ADR 0015) end to end.
- Identity knobs: the reported address (`MockSignerBuilder::address`) defaults to
  the development key's address; setting it to a different address models a wallet
  that reports one identity but signs with another (the owner-recovery mismatch
  case). The fixed-signature overrides (`MockSignerBuilder::typed_data_signature`
  / `message_signature`) remain for error-path and wire-shape tests.
- Confined crypto: the added dependencies are the pure `alloy-dyn-abi` typed-data
  hasher, `k256`, `cow-sdk-contracts`, `alloy-primitives`, and `serde_json` — none
  keystore-capable.
  `alloy-signer-local` stays confined to the alloy-adapter crates, so the workspace
  no longer dev-depends on `cow-sdk-alloy-signer` outside those crates and the
  dependency-isolation gate covers the full dev-edge-inclusive graph.
- Cost: one more published crate to version, and the doubles must track the
  public trait seams they implement.

## Alternatives Rejected

- A `testing` feature on `cow-sdk-trading`: weaker release isolation, and it
  couples test scaffolding to a product crate against the thin-facade,
  real-crate-boundary posture of ADR 0001.
- A mocking-framework dependency: an expectation DSL fits `#[async_trait]`
  traits but not the native-async `Signer` / `Provider` traits, and would
  publish a third-party DSL as part of the surface; hand-written recording
  doubles cover both trait forms uniformly.
- Leave the doubles in the examples only: consumers must copy glue that drifts
  and is not versioned.

## Links

- [Architecture](../guides/architecture.md)
- [Principles](../principles/index.md)
- [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md)
- [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md)
- [ADR 0033](0033-minimum-viable-panic-surface.md)
- [ADR 0053](0053-typed-signer-rejection-classification.md)
- [ADR 0062](0062-internal-shared-test-support-crate.md)
