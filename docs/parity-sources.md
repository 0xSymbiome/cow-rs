# Parity Sources

## Repository Contract

`cow-rs` is a standalone Rust repository. Normal build, test, and publish flows
must not require local checkouts of:

- `cowprotocol/cow-sdk`
- `cowprotocol/contracts`
- `cowprotocol/services`

Those upstream repositories are used only during intentional parity refresh or
source-root validation work. They must not be committed into this repository.

The committed parity contract lives in:

- `parity/source-lock.yaml`
- `parity/fixtures/*.json`
- `crates/app-data/schemas/`

For implementation status and Rust crate coverage, see
[`docs/parity-matrix.md`](parity-matrix.md). This page only defines source
provenance and validation rules.

## Validation Modes

Standalone validation does not require local upstream checkouts:

```text
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate --source-lock parity/source-lock.yaml
```

This checks that the source lock, fixture contracts, embedded fixture commits, and
vendored app-data schema bundle are internally consistent.

Upstream-root validation is stricter and is only meaningful when the supplied
paths are independent git checkouts or worktrees of the pinned producer
repositories:

```text
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate --source-lock parity/source-lock.yaml --cow-sdk-root <cow-sdk-checkout> --contracts-root <contracts-checkout> --services-root <services-checkout>
```

For each supplied root, the validator requires:

- the supplied path to be the git top-level for that repository,
- a remote matching the expected upstream repository,
- `HEAD` to match the pinned commit in `parity/source-lock.yaml`,
- all declared producer paths to exist,
- all declared producer paths to be clean relative to `HEAD`.

Directory copies are not valid source evidence if Git resolves them upward into
the `cow-rs` repository. In that case the validator must fail and ask for an
independent upstream checkout or worktree.

## Pinned Revision Set

Current pinned SHA set in `parity/source-lock.yaml`:

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
|---|---|---|
| core | `cow-sdk/packages/common/src/adapters/*`, `common/src/utils/address.ts`, `common/src/utils/token.ts`, config types/constants, selected orderbook/signing shared type files, `contracts/src/ts/order.ts` | `parity/fixtures/core.json` |
| contracts | `contracts/src/ts/order.ts`, `sign.ts`, `settlement.ts`, `swap.ts`, `interaction.ts`, `vault.ts`, `proxy.ts`, selected Solidity tests, and selected `cow-sdk/packages/contracts-ts/src/*` plus tests | `parity/fixtures/contracts.json` |
| signing | `cow-sdk/packages/order-signing/src/orderSigningUtils.ts`, `utils.ts`, `types.ts`, order-signing tests, selected trading typed-data consumers, `contracts/src/ts/order.ts`, `contracts/src/ts/sign.ts` | `parity/fixtures/signing.json` |
| app-data | `cow-sdk/packages/app-data/src/api/*`, `src/types.ts`, `src/consts.ts`, `src/importSchema.ts`, `src/utils/*`, `src/generatedTypes/*`, app-data schema regression tests | `parity/fixtures/app-data.json` |
| orderbook | `cow-sdk/packages/order-book/src/api.ts`, `request.ts`, `transformOrder.ts`, `types.ts`, related tests, and selected `services` orderbook schema/app-data semantics | `parity/fixtures/orderbook.json` |
| trading | `cow-sdk/packages/trading/src/*` quote, order, post, cancellation, slippage, settlement, pre-sign, eth-flow, and trading SDK paths plus selected tests | `parity/fixtures/trading.json` |
| subgraph | `cow-sdk/packages/subgraph/src/api.ts`, `queries.ts`, `graphql.ts`, selected `api.spec.ts` scenarios | `parity/fixtures/subgraph.json` |
| sdk | `cow-sdk/packages/sdk/src/index.ts`, `typedoc-entry.ts`, `package.json`, `README.md` | `parity/fixtures/sdk.json` |

## Source-Schema Provenance

Orderbook schema evidence is tied to pinned upstream producer sources, including:

- `cow-sdk:packages/order-book/src/api.ts`
- `cow-sdk:packages/order-book/src/request.ts`
- `cow-sdk:packages/order-book/src/types.ts`
- `services:crates/orderbook/openapi.yml`

Subgraph evidence is tied to pinned upstream producer sources, including:

- `cow-sdk:packages/subgraph/src/api.ts`
- `cow-sdk:packages/subgraph/src/graphql.ts`
- `cow-sdk:packages/subgraph/src/queries.ts`

Generated or schema-derived mirrors must be clearly located and kept internal or
test-only unless a dedicated public API change promotes them into the SDK.

## Provenance Rule

Only repositories listed in `parity/source-lock.yaml` are parity sources.
Repositories that are not listed there are not fixture provenance, source-lock
inputs, or justification for copied literals, defaults, or placeholder behavior.

## Maintainer Commands

Refresh the vendored app-data schema bundle from an explicit upstream `cow-sdk`
checkout:

```text
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- vendor-app-data-schemas --source-lock parity/source-lock.yaml --cow-sdk-root <cow-sdk-checkout>
```

Refresh the source lock from explicit upstream roots:

```text
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- snapshot --output parity/source-lock.yaml --cow-sdk-root <cow-sdk-checkout> --contracts-root <contracts-checkout> --services-root <services-checkout>
```

These commands are intentionally local-root driven. They do not fetch floating
remote revisions during normal repository workflows.

## Maintenance Rules

- Do not point parity evidence at floating upstream `main`.
- Update pinned SHAs only in dedicated parity refresh changes.
- Keep fixture provenance explicit in every `parity/fixtures/*.json` file.
- Keep embedded fixture `source_refs[].commit` values aligned with `parity/source-lock.yaml`.
- Keep `crates/app-data/schemas/` synchronized from a real `cow-sdk` checkout at the pinned commit.
- Treat `crates/app-data/schemas/` as vendored compatibility assets, not as a handwritten local schema fork.
- Keep each fixture file synchronized with its implemented Rust surface.
- Keep local upstream roots out of the repository contract and out of published crate behavior.
- Do not use repositories outside `parity/source-lock.yaml` as fixture provenance or implementation sources.
