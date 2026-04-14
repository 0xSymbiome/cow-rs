# Verification Guide

Use this guide to understand how `cow-rs` justifies its public behavior.

## Verification Model

`cow-rs` uses a layered public evidence model:

- [Properties Registry](../PROPERTIES.md): the canonical index of invariants and
  state contracts
- crate contract, property, and state-machine tests: the primary executable
  proof for crate behavior
- examples: consumer-facing scenario proof
- workflow lanes: repository-wide quality, compatibility, documentation, and
  publication gates
- parity fixtures and source locks: provenance and upstream traceability
- audits and ADRs: focused review records and durable design history

## Where To Start

| Surface | Start with | Then inspect |
| --- | --- | --- |
| Crate boundaries and crate ownership | [Architecture](architecture.md) | [ADRs](adr/README.md) |
| Proof classes and support posture | [Validation Scope](validation-scope.md) | [Verification Matrix](verification-matrix.md) |
| Invariant ownership | [Properties Registry](../PROPERTIES.md) | crate-local contract and property tests |
| Release, publication, and provenance | [Release Checklist](release-checklist.md) | [Parity Matrix](parity-matrix.md), [Parity Sources](parity-sources.md) |
| Focused engineering review | [Audits](audit/README.md) | surface-local tests and source files |
| Example behavior | [Examples](examples.md) | example README files and scenario code |

## Boundary Checks

### Runtime And Typed-Data Contracts

`cow-sdk-core` owns the shared runtime seams. Sync and async signer/provider
contracts stay explicit, and typed-data payloads remain structured rather than
being reconstructed from field-name heuristics.

### Transport Ownership

Shared HTTP client policy is intentionally narrow. Retry behavior, rate limits,
GraphQL request shape, API-key handling, and pinning semantics remain owned by
the transport crates that define those behaviors.

### Workflow Ownership

`cow-sdk-trading` owns quote-to-order orchestration. Review trading changes at
the workflow layer first, then inspect the lower-level crates it composes.

### Browser-Runtime Support

Browser wallet support is explicit, bounded, and feature-gated. Deterministic
proof comes from crate tests, direct browser-bridge coverage, mock-wallet
flows, and fixture-backed browser automation. When a browser workflow already
owns a chain authority, `BrowserWallet::signer_for_chain` keeps address,
signature, gas, and transaction operations bound to that chain. Live extension
behavior remains environment-sensitive, and the shipped static browser consoles
keep production live orderbook calls explicitly gated behind a proxy-enabled
deployment requirement.

### Published Crate Policy

MSRV, docs.rs posture, public rustc lints, dependency policy, publication dry
runs, and provenance-sensitive parity checks are part of the published
crate-family contract. Review publication-policy changes through the release
docs rather than as local implementation details.

## Going Deeper

Use deeper evidence only when the change warrants it:

- search-profile tests for larger deterministic helper families
- targeted mutation scopes for deterministic transport or helper seams
- provenance-sensitive parity validation when fixture provenance changes
- saved query documents and test-only schema evidence when a schema-backed
  subgraph boundary changes
- optional smoke checks when browser pages or live services must be confirmed

The canonical command set lives in [Release Checklist](release-checklist.md).

## Review Rules

- start from the owning crate, not from the facade
- use the properties registry to identify what must remain true
- use the matrix docs to identify the current executable evidence
- keep deterministic proof separate from environment-sensitive confirmation
- treat browser-runtime support, live services, and upstream provenance as
  explicit boundaries rather than hidden assumptions
