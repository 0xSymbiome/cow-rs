# Parity Sources

## Repository Contract

`cow-rs` is a standalone Rust repository. Normal build, test, and publish
flows must not require local checkouts of:

- `cowprotocol/cow-sdk`
- `cowprotocol/contracts`
- `cowprotocol/services`
- `alloy-rs/alloy`
- `alloy-rs/core`

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
   - `https://github.com/alloy-rs/alloy`
   - `https://github.com/alloy-rs/core`

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

3. `cargo parity-provision-upstreams --output-root <dir>` is the
   supported provisioning command for reviewers who want to reproduce
   the parity verification step locally. The Cargo alias dispatches to
   the canonical Rust subcommand under `scripts/parity-maintainer/`,
   which reads `parity/source-lock.yaml`, clones each pinned upstream
   repository under `<output-root>/<id>`, checks out the pinned commit
   detached, and reports the resolved paths so the reviewer can pass
   them straight into the upstream-root validator command.

## Validation Modes

Repo-local validation does not require upstream checkouts:

```text
cargo parity-validate --source-lock parity/source-lock.yaml
```

Upstream-root validation is stricter and is only meaningful when the supplied
paths are independent git checkouts or worktrees of the pinned producer
repositories:

```text
cargo parity-validate --source-lock parity/source-lock.yaml --cow-sdk-root <cow-sdk-checkout> --contracts-root <contracts-checkout> --services-root <services-checkout>
```

For each supplied root, the validator requires:

- the supplied path to be the git top-level for that repository
- a remote matching the expected upstream repository
- `HEAD` to match the pinned commit in `parity/source-lock.yaml`
- all declared producer paths to exist
- all declared producer paths to be clean relative to `HEAD`

Before relying on manually supplied upstream roots, reviewers can run the
report-only root check:

```text
cargo check-source-lock-roots --cow-sdk-root <cow-sdk-checkout> --contracts-root <contracts-checkout> --services-root <services-checkout>
```

The command warns when a supplied path resolves to a parent checkout, has a
remote that differs from the source-lock repository, or has `HEAD` checked out
at a different commit than the source-lock pin.

## Pinned Revisions

- `cow-sdk`: `74393ee2923a2932584998169daca6ce3c2da60c`
- `contracts`: `c6b61ce75841ce4c25ab126def9cc981c568e6c6`
- `ethflowcontract`: `762d182674f8f890bd27917872ee62125171b54d`
- `services`: `1f80d54bc3521b3fa81cd8ad66d9f749c5450591`
- `alloy`: `f3fe4cfff0553e9e234a53208bb69b7c222c66e5`
- `alloy-core`: `e6b30e4c2407cd1d2ea93e79f2768e5a4f21d266`

The native Alloy adapter family pins two version generations: Alloy runtime
crates at `2.0.4` for provider, transport, network, RPC, signer, and
signer-local crates, and Alloy Core ABI crates at `1.5.7` for primitives,
dynamic ABI, JSON ABI, Solidity macro, and Solidity types. The two families
ship on independent release cadences; the workspace lockfile invariant enforces
single-version resolution across both families.

## Source Ownership

`cow-rs` interoperates with a live protocol, so its parity authorities are the
upstream producers that define the protocol contract on the wire and on-chain.

Primary protocol authorities:

- `https://github.com/cowprotocol/services.git` — the off-chain authority for
  the orderbook HTTP API, the OpenAPI schemas, the wire DTOs, and the
  order-validation and rejection semantics the SDK must match to interoperate.
- `https://github.com/cowprotocol/contracts.git` — the on-chain authority for
  EIP-712 order hashing, the settlement ABI, and deployment addresses.
- `https://github.com/cowprotocol/ethflowcontract.git` — the on-chain authority
  for the EthFlow order surface.

Cross-language reference:

- `https://github.com/cowprotocol/cow-sdk.git` — the upstream TypeScript SDK. It
  is the reference for which consumer workflows a CoW Protocol SDK should offer
  and for the app-data and subgraph query shapes, but it is a different language
  with different idioms. It is not the authority for the Rust public API shape
  (Rust idiom governs that) nor for the wire format (services governs that).
  Where the TypeScript SDK and services differ on a name or shape, services is
  authoritative.

Dependency provenance:

- `https://github.com/alloy-rs/alloy.git`
- `https://github.com/alloy-rs/core.git`

The Alloy repositories are dependency-provenance evidence for the native adapter
crates. None of the repositories above are publish-time git dependencies.

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
| native Alloy adapters | Alloy runtime and Alloy Core producer paths pinned in `parity/source-lock.yaml` | Adapter crate tests, transaction broadcast / receipt shape invariants, and native examples |

## Provenance Rule

Only repositories listed in `parity/source-lock.yaml` are parity sources.
Repositories that are not listed there are not fixture provenance, source-lock
inputs, or justification for copied literals or defaults.

## Maintainer Commands

Materialize each pinned upstream repository as an independent worktree
under a chosen output root:

```text
cargo parity-provision-upstreams --output-root <dir>
```

The command reads `parity/source-lock.yaml`, writes each repository to
`<dir>/<id>` (e.g., `<dir>/services`, `<dir>/contracts`,
`<dir>/cow-sdk`), and reports the resolved paths.

Refresh the source lock from explicit upstream roots:

```text
cargo parity-snapshot --output parity/source-lock.yaml --cow-sdk-root <cow-sdk-checkout> --contracts-root <contracts-checkout> --services-root <services-checkout>
```

Generate the report-only services drift summary for a pinned services checkout:

```text
scripts/check-services-drift.sh --upstream <services-checkout> --cow-rs-root . --summary-output <summary.md>
```

The Markdown report schema has three stable sections: `errorType Drift`, `DTO
Field Drift`, and `Summary Count`. CI also emits a `drift_detected` output so
scheduled drift runs can open or update tracking issues without making routine
builds depend on the upstream services repository.

## Maintenance Rules

- do not point parity evidence at floating upstream `main`
- update pinned SHAs only in dedicated parity refresh changes
- keep fixture provenance explicit in every `parity/fixtures/*.json` file
- keep embedded fixture commits aligned with `parity/source-lock.yaml`
- keep `crates/app-data/schemas/` synchronized from a real `cow-sdk` checkout
- keep local upstream roots out of the normal repository contract
