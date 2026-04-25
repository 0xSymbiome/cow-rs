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
- audits and ADRs: current-state review records and durable design history

## Where To Start

| Surface | Start with | Then inspect |
| --- | --- | --- |
| Crate boundaries and crate ownership | [Architecture](architecture.md) | [ADRs](adr/README.md) |
| Proof classes and support posture | [Validation Scope](validation-scope.md) | [Verification Matrix](verification-matrix.md) |
| Invariant ownership | [Properties Registry](../PROPERTIES.md) | crate-local contract and property tests |
| Release, publication, and provenance | [Release Checklist](release-checklist.md) | [Parity Matrix](parity-matrix.md), [Parity Sources](parity-sources.md) |
| Focused engineering review | [Audits](audit/README.md) | surface-local tests and source files |
| Example behavior | [Examples](examples.md) | example README files and scenario code |

When a change materially moves a named audited surface, the corresponding audit
should remain `Current` in the same change set.

## Boundary Checks

### Runtime And Typed-Data Contracts

`cow-sdk-core` owns the shared runtime seams. Sync and async signer/provider
contracts stay explicit, and typed-data payloads remain structured rather than
being reconstructed from field-name heuristics. Review configuration changes at
the owning crate boundary as well: default diagnostics and serialized forms for
credential-bearing config must keep secrets redacted while leaving explicit
inputs and override seams intact. EIP-1271 verification routes through
`verify_eip1271_signature_async` with a mandatory
`Eip1271VerificationCache` argument; only `Ok(())` and
`Eip1271MagicValueMismatch` outcomes are cached, every other error class
re-hits the chain.

### Transport Ownership

HTTP dispatch for the orderbook and subgraph surfaces flows through the
`HttpTransport` trait in `cow-sdk-core`. The native default is
`ReqwestTransport`; the browser default is `FetchTransport` from
`cow-sdk-transport-wasm`. Every adapter strips the URL through
`reqwest::Error::without_url` (native) or explicit omission (browser)
before wrapping, so credential-bearing query strings never surface
through the typed `TransportError` enum. Retry behavior, rate limits,
GraphQL request shape, API-key handling, and pinning semantics sit above
the transport and remain owned by the orderbook and subgraph crates. For
`cow-sdk-subgraph`, that includes keeping stable route identity and typed
request failures free of raw Graph API credentials.

### Stability Invariant

The published `cow-sdk` crate family (`cow-sdk`, `cow-sdk-core`,
`cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data`,
`cow-sdk-orderbook`, `cow-sdk-trading`, `cow-sdk-subgraph`,
`cow-sdk-browser-wallet`) does not transitively depend on
`alloy-provider`. Review every dependency change against this invariant;
the release-gating `cargo tree --invert alloy-provider` command returns
empty on the shipped workspace.

### Workflow Ownership

`cow-sdk-trading` owns quote-to-order orchestration. Review trading changes at
the workflow layer first, then inspect the lower-level crates it composes.
That surface is responsible for preserving reviewed balance semantics across
quote-derived and direct order construction, enforcing one injected-orderbook
validation contract across all `TradingSdk` constructors, separating ready-state
construction from explicit partial helper setup, and rejecting
recoverable-signature owner or signer mismatch before submission. User-facing
partner-fee policy also remains typed here until the explicit app-data metadata
translation boundary.

### Browser-Runtime Support

Browser wallet support is explicit, bounded, and feature-gated. Deterministic
proof comes from crate tests, direct browser-bridge coverage, mock-wallet
flows, and fixture-backed browser automation. When a browser workflow already
owns a chain authority, `BrowserWallet::signer_for_chain` keeps address,
signature, gas, and transaction operations bound to that chain. Typed
chain-management helpers such as `switch_chain` and `switch_or_add_chain`
return success only after the refreshed wallet session confirms the requested
chain. Live extension behavior remains environment-sensitive, and the shipped
static browser consoles keep production live orderbook calls explicitly gated
behind a proxy-enabled deployment requirement.

### Published Crate Policy

MSRV, docs.rs posture, public rustc lints, dependency policy, publication dry
runs, and provenance-sensitive parity checks are part of the published
crate-family contract. Review publication-policy changes through the release
docs rather than as local implementation details. Dependency policy is split
deliberately: `cargo deny` owns bans, licenses, and source policy, while
`cargo audit --deny unsound --deny unmaintained --ignore RUSTSEC-2026-0097 --ignore RUSTSEC-2024-0388 --ignore RUSTSEC-2024-0436 --ignore RUSTSEC-2026-0105`
blocks RustSec vulnerabilities plus unsound and unmaintained advisories.
The ignored advisories cover reviewed upstream postures for which no
direct upgrade path exists; each entry is tracked in
`docs/audit/dependency-gate-audit.md` and, where the reachability
flows through a crate family boundary, in the corresponding crate
dependency audit.
Yanked crates remain reviewed warnings only when the latest published upstream
release still provides no clean replacement, and that state must stay recorded
in public audit evidence.

The `cargo tree --invert alloy-provider` invariant, the `cargo audit --deny ... --ignore RUSTSEC-...` ignore-token list, and the browser-wallet Playwright install browser set are each guarded across their source-of-truth files by `scripts/check-release-docs-agree.sh`.

## Going Deeper

Use deeper evidence only when the change warrants it:

- search-profile tests for larger deterministic helper families
- targeted mutation scopes for deterministic transport or helper seams
- provenance-sensitive parity validation when fixture provenance changes
- saved query documents and test-only schema evidence when a schema-backed
  subgraph boundary changes
- optional smoke checks when browser pages or live services must be confirmed

The canonical command set lives in [Release Checklist](release-checklist.md).
Every shipped `README.md` is wired into crate rustdoc with a `cfg_attr(doctest, doc = include_str!("../README.md"))` shim, so `cargo test --workspace --doc` covers every fenced example.
The `services-drift.yml` workflow compares the upstream services repository's error tags and request or response shapes against the typed orderbook surface each week and records drift as a tracked report.

## Review Rules

- start from the owning crate, not from the facade
- use the properties registry to identify what must remain true
- use the matrix docs to identify the current executable evidence
- keep deterministic proof separate from environment-sensitive confirmation
- treat browser-runtime support, live services, and upstream provenance as
  explicit boundaries rather than hidden assumptions
