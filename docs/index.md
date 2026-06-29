# Documentation

This directory contains the public guides, assurance material, focused audits,
and design history for `cow-rs`.

## For SDK Consumers

- [Getting Started](guides/getting-started.md)
- [Integrations](guides/integrations.md)
- [Principles](principles/index.md)
- [Architecture](guides/architecture.md)
- [Transport](guides/transport.md)
- [Deployments](guides/deployments.md)
- [Examples](guides/examples.md)
- [Provider Adapters](providers/index.md)

Start with [Getting Started](guides/getting-started.md) for the canonical onboarding
path. Then use the other consumer pages to choose crates, understand public
boundaries, integrate custom runtimes, and branch into the maintained example
families.

Use [Getting Started](guides/getting-started.md) for facade-first Rust flows,
[Integrations](guides/integrations.md) for custom HTTP, signer, provider, and callback
boundaries, and [Architecture](guides/architecture.md) for crate ownership and
contracts-test entry points.

For JavaScript and TypeScript consumers, the
[when-to-use table in the root README](../README.md#when-to-use-cow-rs) maps each
runtime to the upstream
[`@cowprotocol/cow-sdk`](https://www.npmjs.com/package/@cowprotocol/cow-sdk) or a
`cow-sdk-js` flavor. See
[Architecture](guides/architecture.md#javascript-and-typescript-wasm-surface),
[Integrations](guides/integrations.md#typescript-and-javascript-runtime-boundary), and
[cow-sdk-js](../crates/js/README.md) for the detail.

## Common Boundary Questions

- Why is `cow-sdk-subgraph` separate? The default `cow-sdk` facade stays
  trading-first, so read-only analytics and custom GraphQL access remain
  explicit through `cow-sdk-subgraph`. See
  [Architecture](guides/architecture.md#why-the-subgraph-crate-is-opt-in) and
  [ADR 0003](adr/0003-separate-read-only-subgraph-crate.md).
- Where do native runtime integrations fit? `cow-sdk-core::{Signer,
  SigningProvider, Provider}` defines the stable extension contract for signer
  and RPC adapters. Native Alloy integrations ship as opt-in adapter crates,
  and other provider-specific integrations remain additive leaf crates rather
  than widening the default facade. See [Integrations](guides/integrations.md) and
  [Architecture](guides/architecture.md#provider-and-signer-adapter-seams).
- Which Alloy crate should I use? Use `cow-sdk-alloy-provider` for read-only
  RPC, `cow-sdk-alloy-signer` for local private-key signing, and
  `cow-sdk-alloy` when the same native client should satisfy both provider and
  signer helper paths. See [Adapting Alloy](providers/adapting-alloy.md).
- What does transaction submission return? Signers return
  `TransactionBroadcast`, a broadcast-hash acknowledgement. Provider receipt
  lookups return `TransactionReceipt` with mined fields when available. See
  [ADR 0038](adr/0038-transaction-lifecycle-types.md).
- How do I plug in a custom HTTP transport? Every `HttpTransport` impl
  installs through the builder's `.transport(...)` setter on both
  `OrderbookApi` and `SubgraphApi`. Native consumers get
  `ReqwestTransport` by default; browser consumers get `cow-sdk-core`'s
  `FetchTransport` (the browser sibling of `ReqwestTransport`, in its
  `transport::fetch` module). See [Transport](guides/transport.md) for the full seam.
- How do TypeScript apps use the SDK? Use `cow-sdk-js` after npm
  publication. Browser bundlers can use the default fetch-backed path, while
  Node.js, Workers, Deno, and custom runtimes provide `CowFetchCallback`
  through the callback transport. See [Integrations](guides/integrations.md).
- Where do deployed contract addresses come from? Every address routes
  through the typed `Registry` in `cow-sdk-contracts`. See
  [Deployments](guides/deployments.md).

## For Verification And Review

- [Verification](guides/verification.md)
- [Properties Registry](properties/index.md)

## For Trust And Maintenance

- [Change History](../CHANGELOG.md)
- [Security Policy](../SECURITY.md)
- [Release Checklist](guides/release-checklist.md)
- [Publication Handoff](guides/publication-handoff.md)

Use these pages when you need the public maintenance posture, disclosure path,
or publication-readiness contract.

## For Parity And Provenance

- [Parity And Provenance](guides/parity.md)
- [cow-rs and the TypeScript SDK](guides/comparison-with-typescript-sdk.md)

## For Performance And Transport Tuning

- [Transport](guides/transport.md)
- [Performance Posture](guides/performance.md)

The transport page explains the `HttpTransport` seam and its native and
browser defaults. The performance posture records the benchmarked hot
paths, reported measurement ranges, the shared transport pattern, and
the production-bot HTTP/2 keep-alive recipe.

## For Observability

- [Observability](guides/observability.md)

The observability page documents the opt-in `tracing` feature family, the
subscriber setup, the complete structured-field registry, and the
`CowError::class` classification surface used by downstream telemetry.

## For Focused Reviews And Design History

- [Audits](audit/index.md)
- [ADRs](adr/index.md)
- [Alloy Doctrine](guides/alloy-doctrine.md)

Audits are current-state review records for named trust-significant surfaces.
ADRs capture durable design decisions and their rationale.
The Alloy Doctrine is the canonical human-readable consolidation of the
ADR set on when cow-rs uses alloy directly, when it owns logic, and when
it routes through an adapter.

## For Contributors

- [Contributing](../CONTRIBUTING.md)
- [Code of Conduct](code-of-conduct.md)
