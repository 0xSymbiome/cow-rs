# Documentation

This directory contains the public guides, assurance material, focused audits,
and design history for `cow-rs`.

## For SDK Consumers

- [Getting Started](getting-started.md)
- [Integrations](integrations.md)
- [Principles](principles.md)
- [Architecture](architecture.md)
- [Transport](transport.md)
- [Deployments](deployments.md)
- [Examples](examples.md)
- [Provider Adapters](providers/README.md)

Start with [Getting Started](getting-started.md) for the canonical onboarding
path. Then use the other consumer pages to choose crates, understand public
boundaries, integrate custom runtimes, and branch into the maintained example
families.

## Common Boundary Questions

- Why is `cow-sdk-subgraph` separate? The default `cow-sdk` facade stays
  trading-first, so read-only analytics and custom GraphQL access remain
  explicit through `cow-sdk-subgraph`. See
  [Architecture](architecture.md#facade-and-adapter-faq) and
  [ADR 0003](adr/0003-separate-read-only-subgraph-crate.md).
- Where do native runtime integrations fit? `cow-sdk-core::{Signer,
  AsyncSigner, AsyncSigningProvider, Provider, AsyncProvider}` defines the stable extension contract
  for signer and RPC adapters. Native Alloy integrations ship as opt-in
  adapter crates, and other provider-specific integrations remain additive
  leaf crates rather than widening the default facade. See
  [Integrations](integrations.md) and
  [Architecture](architecture.md#provider-and-signer-adapter-seams).
- Which Alloy crate should I use? Use `cow-sdk-alloy-provider` for read-only
  RPC, `cow-sdk-alloy-signer` for local private-key signing, and
  `cow-sdk-alloy` when the same native client should satisfy both provider and
  signer helper paths. See [Adapting Alloy](providers/adapting-alloy.md).
- How do I plug in a custom HTTP transport? Every `HttpTransport` impl
  installs through the builder's `.transport(...)` setter on both
  `OrderBookApi` and `SubgraphApi`. Native consumers get
  `ReqwestTransport` by default; browser consumers install
  `FetchTransport` from `cow-sdk-transport-wasm`. See
  [Transport](transport.md) for the full seam.
- Where do deployed contract addresses come from? Every address routes
  through the typed `Registry` in `cow-sdk-contracts`. See
  [Deployments](deployments.md).

## For Verification And Review

- [Validation Scope](validation-scope.md)
- [Verification Guide](verification-guide.md)
- [Verification Matrix](verification-matrix.md)
- [Properties Registry](../PROPERTIES.md)

## For Trust And Maintenance

- [Change History](../CHANGELOG.md)
- [Security Policy](../SECURITY.md)
- [Release Checklist](release-checklist.md)
- [Publication Handoff](publication-handoff.md)

Use these pages when you need the public maintenance posture, disclosure path,
or publication-readiness contract.

## For Parity And Provenance

- [Parity Matrix](parity-matrix.md)
- [Parity Sources](parity-sources.md)
- [Parity Scope](parity-scope.md)

## For Performance And Transport Tuning

- [Transport](transport.md)
- [Performance Posture](performance.md)

The transport page explains the `HttpTransport` seam and its native and
browser defaults. The performance posture records the benchmarked hot
paths, reported measurement ranges, the shared transport pattern, and
the production-bot HTTP/2 keep-alive recipe.

## For Observability

- [Observability](observability.md)

The observability page documents the opt-in `tracing` feature family, the
subscriber setup, the complete structured-field registry, and the
`SdkError::class` classification surface used by downstream telemetry.

## For Focused Reviews And Design History

- [Audits](audit/README.md)
- [ADRs](adr/README.md)

Audits are current-state review records for named trust-significant surfaces.
ADRs capture durable design decisions and their rationale.

## For Contributors

- [Contributing](../CONTRIBUTING.md)
- [Code of Conduct](code-of-conduct.md)
