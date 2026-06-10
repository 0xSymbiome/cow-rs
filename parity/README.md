# Parity Maintenance Contract

`parity/` is a committed, reviewable evidence layer for the Rust SDK.

It is not a runtime dependency of any published crate.

## What lives here

- `source-lock.yaml` — pinned upstream producer repositories and commits
- `fixtures/` — committed parity fixtures with in-file `source_refs` provenance,
  covering the contracts, trading, app-data, cow-shed, signing, and
  orderbook surfaces

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
- source-lock validation through the `xtask` validator:

```sh
cargo parity-validate \
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

Materialize the pinned upstream checkouts (blob-less clones under
`target/upstream/`, or `--root`/`XTASK_UPSTREAM_ROOT` for a custom location):

```sh
cargo xtask parity sync
```

Check whether upstream default branches have moved any producer path since the
pins (exit 0 = no drift, 1 = drift reported, 2 = a pin or fetch failed):

```sh
cargo xtask parity drift
```

Advance the pins: fetches each remote's default branch, prints the per-file
drift table, rewrites the `commit:` lines in `parity/source-lock.yaml`
(comments preserved), and fails closed if any producer path is missing at the
new pin. Review the lock diff, refresh whatever the drift table cites
(re-vendor the OpenAPI document if `services` moved), and re-run the gates:

```sh
cargo xtask parity sync --update
```

Deep-validate pinned roots (the synced checkouts work directly):

```sh
cargo parity-validate \
  --source-lock parity/source-lock.yaml \
  --contracts-root target/upstream/contracts \
  --services-root target/upstream/services \
  --cow-sdk-root target/upstream/cow-sdk
```

Running `validate` without upstream roots only proves that the committed lockfile and fixtures are
internally consistent. It does not prove that the pinned commits still match real upstream
checkouts.

The app-data metadata is validated by typed Rust construction, not a vendored
JSON-Schema bundle. `crates/app-data/schemas/` retains one self-contained drift
fixture per modeled metadata family (`flashloan`, `partnerFee`, `quote`, and the
`hook` shape) as test-only fixtures: the `schema_drift_contract` test asserts the
typed metadata structs still match the upstream field names, so an upstream
rename or addition surfaces at review time. Refresh those fixtures by hand from a
pinned `cowprotocol/app-data` checkout when the drift test flags a change.

Refresh the source lock by editing the pinned rows in
`parity/source-lock.yaml` directly: bump the `commit` for the changed upstream
(and adjust its `producer_paths` if files moved), then rerun the root-checked
`validate` above against fresh checkouts at the new commits. The validator
checks the lock by form — schema version, GitHub `.git` remotes, 40-character
lowercase hex commits, known roles, and unique non-traversing producer paths —
so the committed file is the single source of truth rather than a copy of a
hardcoded contract.

The canonical fixture corpus stays committed in this repository so parity review and
normal CI do not depend on those upstream roots.

Embedded `source_refs[].commit` metadata inside `parity/fixtures/*.json` must stay aligned with
`parity/source-lock.yaml`. `validate` treats commit drift there as a real failure.

`parity/fixtures/` holds **class-T** files only: values transcribed from a
pinned upstream artifact, whose `source_refs` point at the transcription source
(digests, byte vectors, wire-DTO samples, RFC-derived dates). Self-derived
**class-C** convention pins — outputs our own formula produces from inputs, with
no upstream byte to transcribe — live inline in the consuming test as `const`
literals with a derivation comment, not as a fixture; their provenance is the
documented convention (the relevant ADR plus the cited services source), which a
comment records more faithfully than a `source_refs` block that would otherwise
cite where the convention is defined rather than where the numbers came from.

Each `fixtures/*.json` file carries its own `source_refs` provenance pinning it
to the CoW Protocol producer repositories recorded in `source-lock.yaml`:
`cowprotocol/services` for the orderbook and trading wire surfaces, and
`cowprotocol/contracts` — with `cowprotocol/ethflowcontract` and
`cowdao-grants/cow-shed` — for the on-chain surfaces. See
[docs/parity.md](../docs/parity.md) for the full authority and
ownership split.

External reference implementations are not part of this parity contract. They
may be consulted as secondary implementation references, but they must never be
used as provenance sources for committed fixtures, placeholder values, or copied
defaults.
