# Parity Maintenance Contract

`parity/` is a committed, reviewable evidence layer for the Rust SDK.

It is not a runtime dependency of any published crate.

## What lives here

- `source-lock.yaml`
- `fixtures/core.json`
- `fixtures/contracts.json`
- `fixtures/signing.json`
- `fixtures/app-data.json`
- `fixtures/orderbook.json`
- `fixtures/trading.json`
- `fixtures/subgraph.json`

## Repo contract

- Normal `cow-rs` builds and tests must work without upstream checkouts.
- Fixture files are committed and reviewable.
- Upstream checkouts are only for maintainers performing an intentional parity refresh.
- Vendored app-data schemas are committed compatibility assets, not a live dependency on
  `cow-sdk`.

## Methodology

The committed parity fixture system is a structural parity contract, not a
runtime cross-language harness. `parity/source-lock.yaml` pins the upstream
repositories and commits that the fixtures were derived from, and the fixture
files keep those provenance anchors visible through committed `source_refs`
metadata.

That means the parity layer proves three things:

- structural anchoring to pinned upstream commits recorded in
  `parity/source-lock.yaml`
- curated test-case integrity inside the committed fixture corpus
- source-lock validation through the parity maintainer:

```sh
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate \
  --source-lock parity/source-lock.yaml
```

When maintainers supply explicit upstream roots to the same `validate`
command, the proof surface widens to include commit alignment between the
committed source lock and those independent checkouts.

This layer does not prove automated cross-language value comparison at
`cargo test` runtime. No TypeScript runtime executes during the Rust test
suite, and the committed parity fixtures are not a live oracle that reruns the
upstream SDK on demand. Behavioral cross-verification would require a separate
cross-language comparison harness beyond the structural parity contract
documented here.

## Maintainer workflow

Validate pinned upstream roots:

```sh
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate \
  --source-lock parity/source-lock.yaml \
  --cow-sdk-root <path-to-cow-sdk> \
  --contracts-root <path-to-contracts> \
  --services-root <path-to-services>
```

Running `validate` without upstream roots only proves that the committed lockfile and fixtures are
internally consistent. It does not prove that the pinned commits still match real upstream
checkouts.

Refresh the vendored app-data schema bundle from a pinned `cow-sdk` checkout:

```sh
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- vendor-app-data-schemas \
  --source-lock parity/source-lock.yaml \
  --cow-sdk-root <real-cow-sdk-clone>
```

This command is intentionally explicit:

- it does not fetch from GitHub on its own
- it requires the provided `cow-sdk` root to match the pinned commit from `source-lock.yaml`
- it replaces `crates/app-data/schemas/` with the exact upstream tree from
  `packages/app-data/src/schemas/`

Refresh the source lock from pinned working roots:

```sh
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- snapshot \
  --output parity/source-lock.yaml \
  --cow-sdk-root <path-to-cow-sdk> \
  --contracts-root <path-to-contracts> \
  --services-root <path-to-services>
```

The canonical fixture corpus stays committed in this repository so parity review and
normal CI do not depend on those upstream roots.

Embedded `source_refs[].commit` metadata inside `parity/fixtures/*.json` must stay aligned with
`parity/source-lock.yaml`. `validate` treats commit drift there as a real failure.

When `validate` is given `--cow-sdk-root`, it also proves that `crates/app-data/schemas/`
matches `packages/app-data/src/schemas/` byte-for-byte.

`fixtures/core.json` is the pinned internal-foundation contract. It
anchors shared type, config, and trait expectations to upstream `common`, `config`,
selected `order-book` / `order-signing`, and `contracts` sources.

`fixtures/contracts.json` is the pinned low-level contracts contract.
It anchors order, signature-encoding, deployment, settlement, swap, proxy, vault,
and storage-reader expectations to upstream `contracts` sources and selected
`cow-sdk/packages/contracts-ts` public tests.

`fixtures/signing.json` is the pinned signing contract. It anchors
typed-data field layout, domain precedence, domain separator helpers, deterministic
order ID generation, cancellation signing, and EIP-1271 payload encoding to
upstream `packages/order-signing`, direct `packages/trading` typed-data consumers,
and the canonical `contracts` order and signing sources.

`fixtures/app-data.json` is the pinned app-data contract. It anchors
deterministic document generation, schema lookup and validation, latest and legacy
CID conversion, deterministic app-data info derivation, explicit fetch and upload
transport seams, and selected package-level schema regression tests to upstream
`packages/app-data` sources.

`fixtures/orderbook.json` is the pinned orderbook contract. It anchors
endpoint breadth, request-helper behavior, typed API errors, chain/env URL
resolution, multi-env fallback, app-data GET and PUT transport semantics, and
deterministic order transforms to upstream `packages/order-book` sources plus
selected `services` schemas.

`fixtures/trading.json` is the pinned trading contract. It anchors
quote-only flows, quote-to-order conversion, post-from-quote orchestration, limit
orders, native-sell / EthFlow transactions, pre-sign and cancellation routing,
allowance and approval boundaries, slippage suggestion helpers, and `TradingSdk`
parameter precedence to upstream `packages/trading` sources.

`fixtures/subgraph.json` is the pinned subgraph contract. It anchors
API-key-derived production URL resolution, totals/day/hour query helpers, generic
custom-query execution, generated GraphQL response shapes, explicit override
behavior, and typed unsupported-network / empty-result / query-failure paths to
upstream `packages/subgraph` sources.

External reference implementations are not part of this parity contract. They may
be consulted as secondary implementation references, but they must never be used
as provenance sources for committed fixtures, placeholder values, or copied defaults.
