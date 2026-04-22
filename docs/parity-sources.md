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

## Provenance Layers

The public parity contract is layered so that authoritative provenance is
always reproducible from the committed parity record, never from any
caller-local copy.

1. Authoritative provenance is `parity/source-lock.yaml`. The source-lock
   pins each upstream producer repository to a specific commit:

   - `https://github.com/cowprotocol/cow-sdk`
   - `https://github.com/cowprotocol/contracts`
   - `https://github.com/cowprotocol/services`

   Every committed parity fixture and every embedded schema cites its
   producer paths under one of those pinned commits, so provenance is
   anchored in the repository record itself rather than in any local
   filesystem layout.

2. Parity-sensitive verification materializes each pinned upstream
   repository as an independent git worktree at the pinned commit, in a
   directory outside the cow-rs tree. The worktree's git remote and `HEAD`
   are validated against the pinned upstream repository and commit, so
   only an authentically reproduced upstream root passes the
   provenance-sensitive validator.

3. `scripts/fetch-upstream-pins.sh` is the supported provisioning tool
   for reviewers who want to reproduce the parity verification step
   locally. The script reads `parity/source-lock.yaml`, clones each
   pinned upstream repository to a sibling directory of the cow-rs
   checkout (overridable through `--into <dir>`), checks out the pinned
   commit detached, and prints the resolved paths so the reviewer can
   pass them straight into the upstream-root validator command.

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

Materialize each pinned upstream repository as an independent worktree in
a sibling directory of the cow-rs checkout:

```text
scripts/fetch-upstream-pins.sh
```

Pass `--into <dir>` to provision the worktrees under a chosen base
directory instead of the default sibling layout. The script is
idempotent: existing destinations are left untouched.

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
