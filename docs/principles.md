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

## Evidence-Backed Public Claims

Compatibility, support posture, parity, and release claims must be justified by
repository-visible tests, examples, fixtures, and reproducible validation
documentation.
