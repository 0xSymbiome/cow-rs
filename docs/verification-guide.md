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

`cow-sdk-core` owns the shared runtime seams. Signer and provider
contracts are async by construction, and typed-data payloads remain structured
rather than being reconstructed from field-name heuristics. Review configuration changes at
the owning crate boundary as well: default diagnostics and serialized forms for
credential-bearing config must keep secrets redacted while leaving explicit
inputs and override seams intact. EIP-1271 verification routes through
`verify_eip1271_signature_cached` with a mandatory
`Eip1271VerificationCache` argument; the cache is a positive-only set
keyed on the full `(verifier, digest, signature_hash)` probe identity,
records only `Ok(())` outcomes, and re-hits the chain for a mismatch and
every other error class.

### Transport Ownership

HTTP dispatch for the orderbook and subgraph surfaces flows through the
`HttpTransport` trait in `cow-sdk-core`. The native default is
`ReqwestTransport`; the browser default is `FetchTransport` from
`cow-sdk-transport-wasm`. Every adapter strips the URL through
`reqwest::Error::without_url` (native) or explicit omission (browser)
before wrapping, so credential-bearing query strings never surface
through the typed `TransportError` enum. Retry behavior, rate limits,
GraphQL request shape, and API-key handling sit above the transport and
remain owned by the orderbook and subgraph crates. For
`cow-sdk-subgraph`, that includes keeping stable route identity and typed
request failures free of raw Graph API credentials.

### Stability Invariant

Native Alloy dependencies are intentionally narrow. `alloy-provider` is
allowed only in `cow-sdk-alloy-provider` and `cow-sdk-alloy`, while
`alloy-signer-local` is allowed only in `cow-sdk-alloy-signer` and
`cow-sdk-alloy`. Review every dependency change against these allow-lists. CI
normalises the raw Cargo tree output via
`cargo check-alloy-provider-invariant` and
`cargo check-alloy-signer-invariant`; contributors should use the wrappers
rather than reading raw Cargo output directly.

### Workflow Ownership

`cow-sdk-trading` owns quote-to-order orchestration. Review trading changes at
the workflow layer first, then inspect the lower-level crates it composes.
That surface is responsible for preserving reviewed balance semantics across
quote-derived and direct order construction, locking the quote-amounts projection that derives the signable order from a `/quote` response with a parity regression test, retrying order-id collisions
without reusing salts, falling back from an unset or zero receiver to the
effective owner address, enforcing one injected-orderbook validation contract
across all `Trading` builder terminals, and rejecting recoverable-signature
owner or signer mismatch before submission. User-facing partner-fee policy also remains typed
here until the explicit app-data metadata translation boundary.

### Browser-Runtime Support

Browser wallet support is explicit, bounded, and feature-gated. Deterministic
proof comes from crate tests, direct browser-bridge coverage, mock-wallet
flows, and fixture-backed browser automation. The deterministic Playwright lane
excludes installed-wallet live-extension specs; those checks remain manual
canary evidence under `scripts/validation-smoke/browser-wallet-live/`. When a
browser workflow already owns a chain authority,
`BrowserWallet::signer_for_chain` keeps address,
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
docs rather than as local implementation details. Workspace policy tests keep
the root MSRV aligned with CI, review root dependency default-feature posture,
and check that the native Alloy provider and signer allow-list invariants
enumerate every published crate. Dependency policy is split
deliberately: `cargo deny` owns bans, licenses, source policy, and yanked
advisory policy, while
`cargo audit --deny unsound --deny unmaintained --ignore RUSTSEC-2024-0388 --ignore RUSTSEC-2024-0436`
blocks RustSec vulnerabilities plus unsound and unmaintained advisories.
The ignored advisories are derived from `.github/config/deny.toml` in CI and
cover reviewed upstream postures for which no direct upgrade path exists; each
entry is tracked in
`docs/audit/dependency-gate-audit.md` and, where the reachability
flows through a crate family boundary, in the corresponding crate
dependency audit.
Yanked crates are denied by the cargo-deny advisory gate unless the current
published upstream state is covered by an explicit public audit exception.
Release artifacts ship reproducible at the source and lockfile level today;
the release checklist records the two-tier reproducibility posture and the path
to binary reproducibility for the WebAssembly artifacts.

The `cargo tree --invert alloy-provider` package list, the `cargo audit --deny ... --ignore RUSTSEC-...` ignore-token list, each ignored RustSec rationale entry, and the browser-wallet Playwright install browser set are guarded against their source-of-truth files by `scripts/check-release-docs-agree.sh`.

### Deployment And Capability Evidence

Contract deployment verification is split into addressable registry evidence
and non-addressable coverage evidence. Registry rows carry one of four
verification statuses:

- `CodeHashVerified`: the deployed bytecode is code-hash-verified at the pinned
  upstream manifest (upstream deployments are explorer/Sourcify-verified); cow-rs
  does not commit a local code-hash digest
- `ExternalVerified`: a third-party verifier or explorer attested the bytecode
- `ReadmeTableUnverified`: the row is sourced from an upstream README table and
  has not yet been independently probed
- `CanonicalUnverified`: the row is canonical source evidence, but no committed
  hash or external attestation is available

Coverage rows carry not-deployed, not-supported, or out-of-scope status and do
not resolve through `Registry::address`. The review procedure is:

1. Confirm `registry.toml` and `deployment-provenance.yaml` have identical
   `(contract, chain, environment, address, verification)` rows.
2. For code-hash rows, confirm the upstream manifest at the pinned `source_commit`
   lists the address, and that the live presence probe (`registry-confirm`)
   reports non-empty `eth_getCode` bytecode on the expected chain.
3. For external rows, inspect the named explorer or attestation source and
   confirm the address, chain, and contract family match.
4. For canonical-unverified rows, confirm the address comes from the pinned
   source-lock commit; these carry no upstream-manifest entry or external
   attestation.
5. For not-deployed coverage, confirm the probe returned empty bytecode.
6. For unsupported coverage, confirm the chain is outside the Rust runtime
   support set and is not present in the registry.

COW Shed adds one extra bytecode check: proxy creation-code files under the
contracts ABI directory carry neighboring SHA-256 files, and `build.rs`
validates those bytes before fixture-based CREATE2 address derivation is
trusted.

### CI Architecture Gates

The workflow layer carries three static architecture gates in addition to the
ordinary Rust build and test jobs. The `wasm-imports-grep-gate.yml` workflow
rejects native-only Alloy, `reqwest`, Tokio runtime, Tokio macro, and
`cow-sdk-core` reqwest re-export references in `cow-sdk-wasm` sources. The
shared quality gate runs the standard nextest suite on Ubuntu, macOS, and
Windows with `fail-fast: false`, replacing duplicate single-host jobs with one
matrix-owned host-coverage lane. The same shared quality gate also checks
that every `fetch_doc_from_*` caller awaits the returned future and every
`IpfsFetchTransport` implementation keeps `get` async.

## Going Deeper

Use deeper evidence only when the change warrants it:

- search-profile tests for larger deterministic helper families
- targeted mutation scopes for deterministic transport or helper seams
- provenance-sensitive parity validation when fixture provenance changes
- report-only source-lock root warnings before relying on manually supplied
  upstream checkouts
- saved query documents and test-only schema evidence when a schema-backed
  subgraph boundary changes
- optional smoke checks when browser pages or live services must be confirmed

The canonical command set lives in [Release Checklist](release-checklist.md).
Every shipped `README.md` is wired into crate rustdoc with a `cfg_attr(doctest, doc = include_str!("../README.md"))` shim, so `cargo test --workspace --doc` covers every fenced example.
The `services-drift.yml` workflow compares the upstream services repository's error tags and request or response shapes against the typed orderbook surface each week and records drift as a tracked report.
`retry-soak.yml` runs the deterministic long-run retry and timeout soak nightly,
while `test-depth.yml` publishes scheduled mutation reports without turning
mutation score into a branch-protection threshold.

## Review Rules

- start from the owning crate, not from the facade
- use the properties registry to identify what must remain true
- use the matrix docs to identify the current executable evidence
- keep deterministic proof separate from environment-sensitive confirmation
- treat browser-runtime support, live services, and upstream provenance as
  explicit boundaries rather than hidden assumptions
