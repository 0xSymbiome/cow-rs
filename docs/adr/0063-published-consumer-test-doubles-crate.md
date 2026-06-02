# ADR 0063: Publish Consumer Test Doubles As The `cow-sdk-test` Crate

- Status: Accepted
- Date: 2026-06-02
- Last reviewed: 2026-06-02
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: testing, crate-boundary, public-api, feature-gating, panic
- Related: [ADR 0004](0004-feature-gated-browser-wallet-sidecar.md), [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md), [ADR 0033](0033-minimum-viable-panic-surface.md), [ADR 0062](0062-internal-shared-test-support-crate.md)

## Decision

Consumer-facing test doubles for the SDK public trait seams ship as a published
`cow-sdk-test` crate built only on public APIs. It provides in-memory recording
doubles for `OrderbookClient`, `Signer`, and `Provider` / `SigningProvider`,
ready-made typed-error constructors, and a one-call wired `Trading` convenience,
so a downstream application can test its CoW integration without a live
orderbook, RPC endpoint, or wallet. The root facade re-exports it behind an
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
  published cow crates plus `async-trait`; it never depends on the
  `publish = false` `cow-sdk-test-utils` of ADR 0062, because a published crate
  cannot normal-depend on an unpublished one. It is built strictly on the public
  trait surface — no private or internal APIs.
- Runtime and support: facade exposure is additive and opt-in. The `testing`
  feature re-exports the crate as `cow_sdk::testing` and leaves the default
  facade contract unchanged; the doubles reach a build only through
  `[dev-dependencies]`, so test code cannot enter a production dependency graph.
  Doubles are instance-scoped (ADR 0006) and hold no credentials (ADR 0025).
  Native `Send` doubles ship first; a `wasm32` (`?Send`) variant is an additive
  follow-on behind a feature.
- Validation and review: as a published crate it is part of the panic-free
  shipped surface of ADR 0033 — canned defaults are built through infallible
  constructors with no `unwrap`/`expect`/`panic!` and no allowlist carve-out.
  The crate's own tests drive a real `Trading` through the doubles to prove they
  satisfy the actual trait contracts.
- Cost: one more published crate to version, and the doubles must track the
  public trait seams they implement.

## Alternatives Rejected

- A `testing` feature on `cow-sdk-trading`: weaker release isolation, and it
  couples test scaffolding to a product crate against the thin-facade,
  real-crate-boundary posture of ADR 0001 and ADR 0008.
- A mocking-framework dependency: an expectation DSL fits `#[async_trait]`
  traits but not the native-async `Signer` / `Provider` traits, and would
  publish a third-party DSL as part of the surface; hand-written recording
  doubles cover both trait forms uniformly.
- Leave the doubles in the examples only: consumers must copy glue that drifts
  and is not versioned.

## Links

- [Architecture](../architecture.md)
- [Principles](../principles.md)
- [ADR 0004](0004-feature-gated-browser-wallet-sidecar.md)
- [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md)
- [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md)
- [ADR 0033](0033-minimum-viable-panic-surface.md)
- [ADR 0053](0053-typed-signer-rejection-classification.md)
- [ADR 0062](0062-internal-shared-test-support-crate.md)
