# Validation Scope

This document maps the maintained `cow-rs` surface to the proof that is
committed in the repository.

## Validation Classes

| Class | Meaning | Typical examples |
| --- | --- | --- |
| Deterministic proof | Repository-owned tests, fixtures, builds, and workflow lanes that do not require floating external state. | Crate contract tests, doctests, package dry runs, source-lock validation, mock-wallet flows. |
| Environment-sensitive proof | Checks that depend on host OS, browser runtime, injected wallet, or external endpoint configuration. | Windows compatibility, browser-hosted WASM execution, injected-provider wallet flows. |
| Manual confirmation | Optional live checks that are useful before release but are not part of the routine blocking contract. | GitHub Pages inspection, live orderbook or subgraph smoke checks, extension-backed wallet checks. |

## Canonical References

- [Verification Guide](verification-guide.md)
- [Release Checklist](release-checklist.md)
- [Verification Matrix](verification-matrix.md)
- [Parity Matrix](parity-matrix.md)
- [Parity Sources](parity-sources.md)
- [Parity Scope](parity-scope.md)

## Surface Map

| Surface | Crates | Deterministic proof | Environment-sensitive or manual boundary |
| --- | --- | --- | --- |
| Order creation, signing, and submission | `cow-sdk-signing`, `cow-sdk-orderbook`, `cow-sdk-trading`, `cow-sdk` | Crate contract tests for signing, DTO conversion, post flows, facade exports, and deterministic example execution | Optional live API calls remain outside the routine blocking contract |
| Contracts parity | `cow-sdk-contracts`, `cow-sdk-signing` | Hashing, settlement, signature, reader, and EIP-1271 tests | Live chain-backed spot checks are optional |
| App-data parity | `cow-sdk-app-data`, `cow-sdk-trading` | CID conversion, schema handling, fetch, pinning seams, and fail-closed encoding tests | Live IPFS or pinning services remain optional integration checks |
| Subgraph support | `cow-sdk-subgraph` | Typed query construction, decode, and deterministic native scenarios | Live subgraph access depends on external endpoint configuration |
| Orderbook transport | `cow-sdk-orderbook` | Mocked request-shape, retry, decode, and conversion tests | Live orderbook behavior depends on remote endpoints |
| Browser-target HTTP transport | `cow-sdk-transport-wasm` | Cross-adapter parity against the native `ReqwestTransport` default and the shipped `wasm32-unknown-unknown` build | Live browser fetch behavior depends on vendor-specific network stacks |
| WASM target | `cow-sdk`, `cow-sdk-app-data`, `cow-sdk-transport-wasm`, WASM examples | WASM target builds, direct browser-bridge proof, deterministic verification-console checks, and committed browser automation | Browser-hosted rendering and deployment inspection remain environment-sensitive |
| Browser wallet integration | `cow-sdk-browser-wallet`, `cow-sdk`, browser-wallet console | Native crate tests, direct `wasm-bindgen-test` bridge proof, deterministic mock-wallet flows, console builds, and committed fixture-backed browser automation | Live extension-backed authorization, prompts, and vendor behavior remain environment-sensitive |
| Stability invariant | whole workspace | `cargo tree --invert alloy-provider -p ...` succeeds when no shipped crate transitively depends on `alloy-provider`; Cargo's success-case `did not match any packages` output is normalised by `cargo check-alloy-provider-invariant` for the published `cow-sdk` crate family (`cow-sdk`, `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data`, `cow-sdk-orderbook`, `cow-sdk-trading`, `cow-sdk-subgraph`, `cow-sdk-browser-wallet`) | None |
| Quality and publishability | whole workspace | Formatting, linting, tests, doctests, docs, source-lock validation, and package dry runs | Crates.io publication and independent-root provenance checks are separate operational steps |

## High-Signal Commands

Use [Release Checklist](release-checklist.md) for the full command set. The
highest-signal repository-level checks are:

```text
cargo test --workspace
cargo test --workspace --doc
cargo test --all-features --workspace --doc
cargo nextest run --workspace --all-features --config-file .github/config/nextest.toml
cargo run-deterministic-examples --locked
cargo doc --workspace --all-features --no-deps
cd crates/browser-wallet && wasm-pack test --headless --chrome
cd examples/wasm/sdk-verification-console && wasm-pack test --headless --chrome
bun run --cwd e2e/browser-wallet test
cargo parity-validate --source-lock parity/source-lock.yaml
```

## Explicit Boundaries

- Repo-local source-lock validation proves the committed parity contract from
  this repository checkout.
- Provenance-sensitive parity proof is separate and requires independent
  upstream checkouts at the pinned commits.
- Direct browser-wallet bridge proof and broader console automation cover
  different seams on purpose.
- Live orderbook, subgraph, and extension-backed wallet checks remain optional
  because they depend on external services or runtime state.
