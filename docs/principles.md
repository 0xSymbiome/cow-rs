# Principles

These principles define the public engineering posture of `cow-rs`.

## Deterministic Protocol Transforms

Hashing, signing, UID packing, app-data encoding, and CID handling must stay
deterministic for the same canonical input.

## Explicit Runtime Boundaries

Pure transform crates do not perform hidden HTTP, RPC, GraphQL, or pinning I/O.
Runtime interaction belongs in explicit clients and adapters.

## Thin Facade, Real Crate Boundaries

`cow-sdk` is the ergonomic entrypoint, not a second implementation layer.
Leaf crates own transport, orchestration, browser integration, and other
specialized behavior.

## Instance-Scoped Configuration

Policy-heavy behavior such as quote settings, transport tuning, caching, and
provider selection must be configured per instance through typed builders or
options. `cow-rs` does not hide process-global mutable state behind convenience
APIs.

## Strong Typed Public Surfaces

Public Rust APIs prefer domain types for protocol meanings such as addresses,
hashes, identifiers, and amounts. String-heavy representations are reserved for
explicit wire contracts and compatibility boundaries.

## Additive Optional Ecosystems

Optional capabilities grow through leaf crates and feature-gated additions.
Browser-runtime behavior, provider-specific behavior, and future capability
families do not silently widen the default facade contract.

## Sole Construction Seam

`OrderBookApi`, `SubgraphApi`, and `TradingSdk` construct exclusively
through their typestate builders. The required inputs (chain,
environment or API key, transport) are encoded as compile-time markers
so a misconstructed client is a build error rather than a first-quote
runtime surprise. No free-function public constructors remain on any of
the three.

## Chain-RPC Runtime Neutrality

The published `cow-sdk` crate family does not transitively depend on
`alloy-provider`. Consumers own their chain-RPC runtime through the
`AsyncProvider` seam in `cow-sdk-core`, and the `cargo tree --invert
alloy-provider` check on every crate in the family is a release-gating
invariant rather than an aspiration.

## Canonical Contract Bindings

Every ABI binding the SDK emits call-data against is generated through
`alloy::sol!` from Solidity excerpts committed under
`crates/contracts/abi/`. Hand-rolled encoders are not allowed in
shipped crates, and every chain-scoped address lookup routes through the
typed `Registry` authority in `cow-sdk-contracts`.

## Evidence-Backed Public Claims

Compatibility, support posture, parity, and release claims must be justified by
repository-visible tests, examples, fixtures, and reproducible validation
documentation.
