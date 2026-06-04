# Parity Maintenance Contract

`parity/` is a committed, reviewable evidence layer for the Rust SDK.

It is not a runtime dependency of any published crate.

## What lives here

- `source-lock.yaml`
- `fixtures/contracts.json`
- `fixtures/orderbook.json`
- `fixtures/trading.json`

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
  --contracts-root <path-to-contracts> \
  --services-root <path-to-services>
```

Running `validate` without upstream roots only proves that the committed lockfile and fixtures are
internally consistent. It does not prove that the pinned commits still match real upstream
checkouts.

The app-data metadata is validated by typed Rust construction, not a vendored
JSON-Schema bundle. `crates/app-data/schemas/` retains the latest-version schema
closure as test-only drift fixtures: the `schema_drift_contract` test asserts the
typed metadata structs still match the upstream field names, so an upstream
rename or addition surfaces at review time. Refresh those fixtures by hand from a
pinned `cowprotocol/app-data` checkout when the drift test flags a change.

Refresh the source lock from pinned working roots:

```sh
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- snapshot \
  --output parity/source-lock.yaml \
  --contracts-root <path-to-contracts> \
  --services-root <path-to-services>
```

The canonical fixture corpus stays committed in this repository so parity review and
normal CI do not depend on those upstream roots.

Embedded `source_refs[].commit` metadata inside `parity/fixtures/*.json` must stay aligned with
`parity/source-lock.yaml`. `validate` treats commit drift there as a real failure.

`fixtures/contracts.json` is the pinned low-level contracts contract.
It anchors order, signature-encoding, deployment, settlement, swap, proxy, vault,
and storage-reader expectations to upstream `contracts` sources and selected
`cow-sdk/packages/contracts-ts` public tests.

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

External reference implementations are not part of this parity contract. They may
be consulted as secondary implementation references, but they must never be used
as provenance sources for committed fixtures, placeholder values, or copied defaults.
