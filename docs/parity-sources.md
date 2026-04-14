# Parity Sources

## Repository Contract

`cow-rs` is a standalone Rust repository. Normal build, test, and publish
flows must not require local checkouts of:

- `cowprotocol/cow-sdk`
- `cowprotocol/contracts`
- `cowprotocol/services`

Those repositories are used only during explicit parity refresh or
provenance-sensitive validation.

The committed parity contract lives in:

- `parity/source-lock.yaml`
- `parity/fixtures/*.json`
- `crates/app-data/schemas/`

## Validation Modes

Repo-local validation does not require upstream checkouts:

```text
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate --source-lock parity/source-lock.yaml
```

Upstream-root validation is stricter and is only meaningful when the supplied
paths are independent git checkouts or worktrees of the pinned producer
repositories:

```text
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate --source-lock parity/source-lock.yaml --cow-sdk-root <cow-sdk-checkout> --contracts-root <contracts-checkout> --services-root <services-checkout>
```

For each supplied root, the validator requires:

- the supplied path to be the git top-level for that repository
- a remote matching the expected upstream repository
- `HEAD` to match the pinned commit in `parity/source-lock.yaml`
- all declared producer paths to exist
- all declared producer paths to be clean relative to `HEAD`

## Pinned Revisions

- `cow-sdk`: `17fcfc590be8529dc4fe05b1c472fef1b07b47f4`
- `contracts`: `c94c595a791681cf8ba7495117dcde397b932885`
- `services`: `cfbec985dfe476bf7ef42750435f7d5a12223a85`

## Source Ownership

Primary sources:

- `https://github.com/cowprotocol/cow-sdk.git`
- `https://github.com/cowprotocol/contracts.git`

Reference-only source:

- `https://github.com/cowprotocol/services.git`

`services` is used for transport and validation semantics only. It is not a
publish-time dependency.

## Surface Map

| Surface | Primary upstream paths | Committed fixture |
| --- | --- | --- |
| core | common adapters, address/token helpers, config types, selected shared type files, and `contracts` order helpers | `parity/fixtures/core.json` |
| contracts | `contracts` order, sign, settlement, swap, interaction, vault, proxy, and selected test paths | `parity/fixtures/contracts.json` |
| signing | order-signing utilities, typed-data helpers, selected trading consumers, and contract signing sources | `parity/fixtures/signing.json` |
| app-data | app-data helpers, constants, schema imports, utilities, and generated schema references | `parity/fixtures/app-data.json` |
| orderbook | orderbook API, request, transform, and type sources plus selected `services` references | `parity/fixtures/orderbook.json` |
| trading | trading quote, order, post, cancellation, slippage, settlement, pre-sign, and EthFlow sources | `parity/fixtures/trading.json` |
| subgraph | subgraph API, GraphQL, query, and selected test scenarios | `parity/fixtures/subgraph.json` |
| sdk | SDK root exports, typedoc entrypoint, package metadata, and README surface | `parity/fixtures/sdk.json` |

## Provenance Rule

Only repositories listed in `parity/source-lock.yaml` are parity sources.
Repositories that are not listed there are not fixture provenance, source-lock
inputs, or justification for copied literals or defaults.

## Maintainer Commands

Refresh the vendored app-data schema bundle from an explicit upstream
`cow-sdk` checkout:

```text
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- vendor-app-data-schemas --source-lock parity/source-lock.yaml --cow-sdk-root <cow-sdk-checkout>
```

Refresh the source lock from explicit upstream roots:

```text
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- snapshot --output parity/source-lock.yaml --cow-sdk-root <cow-sdk-checkout> --contracts-root <contracts-checkout> --services-root <services-checkout>
```

## Maintenance Rules

- do not point parity evidence at floating upstream `main`
- update pinned SHAs only in dedicated parity refresh changes
- keep fixture provenance explicit in every `parity/fixtures/*.json` file
- keep embedded fixture commits aligned with `parity/source-lock.yaml`
- keep `crates/app-data/schemas/` synchronized from a real `cow-sdk` checkout
- keep local upstream roots out of the normal repository contract
