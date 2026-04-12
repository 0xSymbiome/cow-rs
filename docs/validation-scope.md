# Validation Scope

This document maps the maintained `cow-rs` surface to the committed validation proof in the repository. It identifies the authoritative repository documents, the primary executable checks, and the boundaries that remain dependent on browser runtimes, external services, or explicit manual confirmation.

## Validation Classes

| Class | Meaning | Typical examples |
| --- | --- | --- |
| Deterministic proof | Committed tests, fixtures, builds, and workflow lanes that run from the repository without requiring live wallets or floating upstream state. | Crate contract tests, doctests, `cargo doc`, source-lock validation, package dry runs, mock-wallet flows. |
| Environment-sensitive proof | Checks whose behavior depends on the host OS, browser runtime, injected wallet, or external service configuration. | Windows compatibility, browser-hosted WASM execution, injected-provider browser-wallet flows, live subgraph access. |
| Manual confirmation | Optional live checks that are useful before release, but are not part of the routine blocking contract. | GitHub Pages inspection, live orderbook or subgraph smoke checks, injected-wallet end-to-end checks. |

## Canonical References

- [Release Checklist](release-checklist.md) for release, publication, parity, and workflow steps.
- [Verification Guide](verification-guide.md) for package boundaries, runtime seams, and validation entry points.
- [Security And Validation Matrix](security-matrix.md) for the crate-by-crate and workflow-by-workflow test inventory.
- [Parity Matrix](parity-matrix.md) for pinned upstream producers, fixtures, and crate ownership.
- [Parity Sources](parity-sources.md) and [Parity Scope](parity-scope.md) for source-lock provenance and upstream-root rules.

## Surface Map

| Surface | Packages | Deterministic proof | Environment-sensitive or manual boundary | Canonical references |
| --- | --- | --- | --- | --- |
| Order creation, signing, and submission | `cow-sdk-signing`, `cow-sdk-orderbook`, `cow-sdk-trading`, `cow-sdk` | Crate contract tests for signing, request/response conversion, trading post flows, and facade exports. | Optional live API calls remain outside the blocking repository contract. | [Parity Matrix](parity-matrix.md), [Security And Validation Matrix](security-matrix.md), [Release Checklist](release-checklist.md) |
| Contracts parity | `cow-sdk-contracts`, `cow-sdk-signing` | Contract helper, hashing, settlement, vault, reader, proxy, and EIP-1271 tests. | Live chain-backed spot checks are optional and separate from the committed fixture contract. | [Parity Matrix](parity-matrix.md), [Parity Sources](parity-sources.md) |
| App-data parity | `cow-sdk-app-data`, `cow-sdk-trading` | CID conversion, schema handling, fetch, pinning seams, and fail-closed encoding tests. | Live IPFS or pinning services remain optional integration checks. | [Parity Matrix](parity-matrix.md), [Security And Validation Matrix](security-matrix.md) |
| Subgraph support | `cow-sdk-subgraph` | Typed query construction, decode, and error-boundary tests plus deterministic native scenarios. | The opt-in live subgraph example depends on external endpoint configuration and remains manual. | [Parity Matrix](parity-matrix.md), [Verification Guide](verification-guide.md), [Examples](examples.md) |
| Blockchain fetch and decode | `cow-sdk-orderbook` | Mocked orderbook transport, request-shape, and response-conversion tests. | Live orderbook behavior depends on remote endpoints and is not part of the routine blocking lane. | [Parity Matrix](parity-matrix.md), [Security And Validation Matrix](security-matrix.md) |
| WASM target | `cow-sdk`, `cow-sdk-app-data`, WASM examples | WASM target builds, deterministic SDK verification console checks, and committed browser automation for the SDK verification console. | Browser-hosted rendering and deployment inspection remain environment-sensitive; GitHub Pages inspection is manual. | [Release Checklist](release-checklist.md), [Examples](examples.md) |
| Quality and publishability | whole workspace | Formatting, linting, tests, doctests, docs, feature-matrix checks, dependency policy, source-lock validation, and package dry runs. | Crates.io publication and independent upstream-root parity validation are separate operational steps. | [Release Checklist](release-checklist.md), [Security And Validation Matrix](security-matrix.md) |
| Browser wallet integration | `cow-sdk-browser-wallet`, `cow-sdk`, browser-wallet console | Browser-wallet crate tests, deterministic mock-wallet flows, WASM builds, and console mock mode. | Injected-provider discovery, authorization, chain inventory, and wallet UX depend on the browser and extension; injected-wallet checks remain manual. | [Verification Guide](verification-guide.md), [Security And Validation Matrix](security-matrix.md), [Release Checklist](release-checklist.md) |

## Primary Commands

Use the release checklist for the full command set. The highest-signal repository-level commands are:

```text
cargo test --workspace
cargo test --workspace --doc
cargo test --all-features --workspace --doc
cargo nextest run --workspace --all-features --config-file .github/config/nextest.toml
cargo doc --workspace --all-features --no-deps
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate --source-lock parity/source-lock.yaml
```

## Explicit Boundaries

- Repo-local source-lock validation proves that the committed fixtures, vendored schemas, and pinned producer metadata are coherent from this repository checkout.
- Provenance-sensitive parity proof is separate and requires independent upstream checkouts at the pinned commits in `parity/source-lock.yaml`.
- `sdk-verification-console` has committed deterministic browser automation. `browser-wallet-console` currently exposes deterministic mock-wallet proof and build validation, while injected-wallet execution remains environment-sensitive.
- Live quote, orderbook, and subgraph interactions remain optional manual checks because they depend on external services or credentials.
- GitHub Pages deployment inspection is useful for release verification, but it is not part of the routine blocking contract.
