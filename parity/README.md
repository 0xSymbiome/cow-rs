# Parity Maintenance Contract

`parity/` is a committed, reviewable evidence layer for the Rust SDK.

It is not a runtime dependency of any published crate.

## What lives here

- `source-lock.yaml` — pinned upstream producer repositories and commits.
  **Pins are the only committed truth; every hash is derived from them at
  check time.** A hash is committed only where no pin can derive it (for
  example the COW Shed creation-code sha256 entries, which lock bytecode
  rather than a pinned source file).
- `fixtures/` — the committed fixture corpus. Every `fixtures/**/*.json` file
  carries its own provenance header (below) and is validated per-file by
  `cargo parity-validate`; a fixture cannot exist outside the contract.
- `openapi/` — the vendored services OpenAPI document (stamped with its source
  commit and gated against the lock pin) plus the DTO coverage manifest.

## Repo contract

- Normal `cow-rs` builds and tests must work without upstream checkouts.
- Fixture files are committed and reviewable.
- Upstream checkouts are only for maintainers performing an intentional parity
  refresh and for the release provenance lane.
- Vendored artifacts are committed compatibility assets, not a live dependency.

## Fixture header grammar

Every fixture starts with a provenance header; payload keys (`cases`, `rows`,
`examples`, `payload`, …) follow and stay test-owned:

```json
{
  "surface": "orderbook-trade",
  "dto": "cow_sdk_orderbook::Trade",
  "endpoint": "GET /api/v1/trades",
  "sources": {
    "services": {
      "commit": "<the lock pin for services>",
      "refs": ["crates/orderbook/openapi.yml#components.schemas.Trade"]
    }
  },
  "standards": ["RFC 7231 §7.1.1.1"],
  "derivation": "one sentence when our own code produced the golden bytes"
}
```

- `surface` (required) — unique across the corpus. Provenance-lookalike keys
  (`source`, `source_refs`, `@source_ref`) are rejected — unknown keys are
  payload by design, so a provenance-shaped key the grammar does not know
  would otherwise sit unvalidated while looking validated.
- `sources` — one entry per cited lock repository. The `commit` must equal
  that repository's pin (the freshness ratchet: bumping a pin fails every
  citing fixture by name until it is consciously re-verified and re-stamped).
  Each ref is `path#fragment`; the path must appear in the lock row's
  `producer_paths`, the fragment is human-facing (symbol names or OpenAPI
  schema paths preferred; `#L<start>-L<end>` line ranges are legal but rot).
- `standards` — non-repo authorities (RFCs, EIPs) as free-text strings.
- `derivation` — optional, for golden vectors our own implementation produced
  against a spec.
- Every fixture must declare `sources` and/or `standards`.
- Case-level refs use `"source_ref": "repo:path#fragment"` — no commit
  segment (commits live once per repo in `sources`), and only paths the
  file-level `sources` declare.
- Raw wire documents are wrapped under a `"payload"` key so the header never
  collides with the wire shape the consuming test round-trips.
- `dto` / `endpoint` are optional free-text conveniences mapping the fixture
  to the Rust type and HTTP surface.

Fixtures hold **class-T** content only: values transcribed from a pinned
upstream artifact (digests, byte vectors, wire-DTO samples, RFC-derived
dates). Self-derived **class-C** convention pins — outputs our own formula
produces from inputs, with no upstream byte to transcribe — live inline in the
consuming test as `const` literals with a derivation comment, not as a
fixture.

## Methodology

The committed parity fixture system is a structural parity contract, not a
runtime cross-language harness. `parity/source-lock.yaml` pins the upstream
repositories and commits the fixtures were derived from; every fixture keeps
its provenance visible and machine-checked through the header above.

The parity layer proves:

- structural anchoring to pinned upstream commits
- curated test-case integrity inside the committed fixture corpus
- per-file provenance validation through the `xtask` validator:

```sh
cargo parity-validate
```

Offline, that validates the lock by form (typed parsing rejects unknown
fields; GitHub `.git` remotes; 40-hex commits; unique non-traversing producer
paths), every fixture header against the pins, and the vendored OpenAPI stamp
against the services pin. It does not prove automated cross-language value
comparison at `cargo test` runtime: no TypeScript executes during the Rust
test suite, and behavioral cross-verification would require a separate
cross-language harness beyond this structural contract.

## Maintainer workflow

All commands share one upstream-root convention:
`--root`/`--upstream-root` > `XTASK_UPSTREAM_ROOT` > `target/upstream`, with
one checkout per lock repository at `<root>/<id>`. Long-lived personal
checkouts plug in with a single profile line
(`$env:XTASK_UPSTREAM_ROOT = "<your root>"`); deep validation requires each
checkout's `HEAD` at the pin, which `parity sync` establishes.

Materialize (or re-detach) the pinned checkouts as blob-less clones:

```sh
cargo xtask parity sync
```

Check whether upstream default branches have moved any producer path since the
pins (exit 0 = no drift, 1 = drift reported, 2 = a pin or fetch failed); the
`upstream-drift` workflow runs this weekly:

```sh
cargo xtask parity drift
```

Advance the pins: fetches each remote's default branch, prints the per-file
drift table (git blob OIDs — no committed checksums, the pin already
content-addresses every path), rewrites the `commit:` lines in
`parity/source-lock.yaml` (comments preserved), and fails closed if any
producer path is missing at the new pin:

```sh
cargo xtask parity sync --update
```

After an update, `cargo parity-validate` fails closed until the refresh is
complete — every fixture still stamped with the old commit is named by the
ratchet, and the vendored OpenAPI stamp gate names the re-vendor step:

```sh
cargo parity-vendor-openapi
```

(zero-argument: it materializes the services checkout at the pin under the
shared root, then writes the stamped document).

Deep-validate every pinned repository plus the vendored OpenAPI body against
the blob at the services pin — the release provenance lane runs exactly this:

```sh
cargo xtask parity sync --root <dir>
cargo parity-validate --upstream-root <dir>
```

The app-data metadata is validated by typed Rust construction, not a vendored
JSON-Schema bundle. `fixtures/app_data/schemas/` retains one self-contained
drift fixture per modeled metadata family: the `schema_drift_contract` test
asserts the typed metadata structs still match the producer field names.
Refresh them from the pinned `app-data` repository (the flash-loan mirror
tracks the `services` producer instead — its header says so) when the drift
test flags a change.

One home per provenance class: anything whose **values** come from upstream
or pin a cross-crate contract is a fixture here, under a validated header.
Crate-local `tests/fixtures/` directories may hold only mock scaffolding
whose bytes are incidental and self-owned (currently: none). Compiled crate
assets (for example the COW Shed proxy creation-code `.bin` files) live in
`src/` and are hash-locked by a fixture here instead of moving.

The canonical fixture corpus stays committed in this repository so parity
review and normal CI never depend on upstream roots.

External reference implementations are not part of this parity contract. They
may be consulted as secondary implementation references, but they must never
be used as provenance sources for committed fixtures, placeholder values, or
copied defaults.

See [docs/parity.md](../docs/parity.md) for the full authority and ownership
split.
