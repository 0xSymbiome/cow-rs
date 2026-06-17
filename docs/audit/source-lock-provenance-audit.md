# Source-Lock Provenance Audit

Status: Current
Last reviewed: 2026-06-15
Owning surface: source-lock provenance and release preflight authority
Refresh trigger: Changes to `parity/source-lock.yaml`, vendored parity OpenAPI or fixture provenance, any change to the maintained exclusion-list policy for historical progress snapshots, or any newly archived progress snapshot that should stay outside active preflight authority
Related docs:
- [ADR 0026](../adr/0026-alloy-major-release-absorption-plan.md)
- [ADR 0030](../adr/0030-workspace-locked-versioning-tag-baseline.md)
- [Alloy Adapters Audit](alloy-adapters-audit.md)

## Scope

This audit covers:

- the committed source-lock pins that define upstream provenance for parity
  fixtures and source-derived review evidence
- the per-file provenance validation holding every `parity/fixtures/**/*.json` to
  a pinned commit, plus the vendored OpenAPI stamp and body gates
- the upstream HEAD comparison that makes source-lock freshness explicit
- the deep upstream-root validation mode for reviewer-supplied checkouts
- the publication preflight that validates the committed lock before package dry runs
- the native Alloy runtime/core upstream pins used for dependency evidence
- the exclusion-list rule keeping historical progress snapshots readable but
  outside active preflight authority, and the audit-refresh mapping back to this record

It does not cover future source-lock refreshes, fixture authoring methodology, or
changing SDK behavior.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Source-lock pins | `parity/source-lock.yaml` pins exact upstream commits for every repository that contributes parity evidence | Conforms |
| Freshness disclosure | Current upstream HEADs are checked explicitly so stale pins are visible before release evidence relies on freshness | Conforms |
| Refresh outcome | The 2026-06-11 refresh advanced the `services` pin (its OpenAPI document and order-validation source had moved) and the `cow-sdk` pin (no producer path changed), re-vendored the services OpenAPI, and re-stamped the twelve services-citing fixtures after re-verifying each against the new commit; parity validation passes offline and deep, and the contract suites reproduce the fixture values | Conforms |
| Deep upstream-root validation | Reviewer-supplied upstream roots (`--upstream-root <dir>`, one checkout per lock repository) are fail-closed checked for independent git top-levels, expected remotes, pinned `HEAD` commits, clean producer paths, and the vendored OpenAPI body at the services pin, without making repo-local validation depend on those roots | Conforms |
| Publication preflight | The package-family dry-run contract (with local patches for unpublished intra-family crates) lives in the release-readiness publication job, which validates the committed lock before the dry runs | Conforms |
| Native Alloy provenance | The native adapter family pins Alloy by crates.io version (`alloy-* = 2.0.4`, `alloy-core-* = 1.5.7`), enforced by `Cargo.lock` and the two-family lockfile invariant | Conforms |
| App-data schema drift fixtures | `parity/fixtures/app_data/schemas/` retains one self-contained drift fixture per modeled metadata family for the typed metadata structs, with lock-validated provenance headers (the flash-loan mirror cites its real producer, `services`) | Conforms |
| Form enforcement | The lock parses through typed models that reject unknown or missing fields, row rules fail closed on malformed remotes, commits, or paths, and every fixture under `parity/fixtures/**/*.json` is validated per-file against the pins | Conforms |
| Fixture wire-value fidelity | Every fixture payload value is a legal instance of the upstream schema its `sources` header cites, and each ref names the authoritative upstream producer symbol for the value it pins | Conforms |
| Amount fixture roundtrip | Amount-shaped fixture strings parse through the shared `Amount` codec and round-trip byte-identically | Conforms |
| Historical snapshot scope | Historical progress snapshots stay readable and unmodified while active preflight authority skips them by directory-prefix policy | Conforms |
| Binding source pins | The upstream Solidity repository each `cow-sdk-contracts` inline `alloy::sol!` binding mirrors is pinned by commit under `repositories:` in `parity/source-lock.yaml`; the bindings themselves are proven byte-for-byte by the fixtures' `sources` headers and crate parity tests rather than a per-file source mirror | Conforms |
| Refresh mapping | The public audit-refresh map points source-lock changes and exclusion-policy changes back to this audit | Conforms |

## Current Contract

### Source-Lock Pins

`parity/source-lock.yaml` is the committed provenance contract for parity
fixtures and source-derived evidence. It pins `contracts`, `services`,
`cow-sdk`, `cow-shed`, and `ethflowcontract`, each by exact commit. The
app-data JSON Schema families are pinned from the `cow-sdk` monorepo
(`packages/app-data/`, published as `@cowprotocol/sdk-app-data`), their
canonical home. The `cow-shed` pin is held at the **v1.0.1 tag commit** — the
deployed generation the inline `sol!` bindings mirror — and is intentionally
behind the upstream default branch, whose v2.x generations are deployed only as
the out-of-family Gnosis chain-100 redeploy (ADR 0049). Pinning the deployed tag
is what makes the v1.0.x fixture refs blob-verifiable.

The lock is intentionally commit-based rather than branch-based. A release claim
that depends on upstream freshness compares these pins against the upstream
repositories before treating the evidence as current.

### Freshness State

`cargo xtask parity drift` checks each pin against its upstream default-branch
HEAD. Every pin except `cow-shed` matches its upstream default branch; the
`cow-shed` pin is deliberately held at the v1.0.1 tag because the SDK binds the
deployed generation, not source HEAD, so the drift report is expected to flag it
until upstream's v2.x generation replaces the v1.0.x deployments — at which point
the pin advances together with new `CowShedVersion` variants. A release claim
that depends on freshness re-runs `cargo xtask parity drift` before relying on
the evidence.

### App-Data Schema Drift Fixtures

`parity/fixtures/app_data/schemas/` holds one self-contained drift fixture per
modeled metadata family (`flashloan`, `partnerFee`, `quote`, and the `hook`
shape) under lock-validated provenance headers, all citing the pinned `cow-sdk`
`packages/app-data/src/schemas/` files. They are test-only fixtures, not resolved
at runtime: validation is typed by construction (ADR 0064). The
`schema_drift_contract` regression test field-name-probes each fixture so an
upstream rename of a field the typed structs depend on fails at review time.

### Deep Upstream-Root Validation

`cargo parity-validate --upstream-root <dir>` is the fail-closed check for
reviewer-supplied upstream checkouts. It requires one checkout per lock
repository at `<dir>/<id>` (the layout `cargo xtask parity sync --root <dir>`
materializes) and fails if any path resolves to a parent git checkout, the origin
remote does not match `parity/source-lock.yaml`, `HEAD` does not equal the pinned
commit, a producer path is missing or dirty, or the vendored OpenAPI body does
not match `crates/orderbook/openapi.yml` at the services pin.

### Publication Preflight

The package-family dry-run contract lives in the release-readiness publication
job (`.github/workflows/release-readiness.yml`): it validates the committed lock
(`cargo parity-validate`), then dry-run packages and publishes the full published
crate family with local patches for unpublished intra-family dependencies. The
source lock carries no metadata block; its purpose is stated in a header comment
and the dry-run contract is workflow-owned, not lock-embedded.

### Form Enforcement

`xtask` validates the source lock by form rather than by matching it against a
hardcoded contract: the typed model rejects unknown or missing fields
(`deny_unknown_fields`, so a misspelled key — or the retired `fixtures:`
section — cannot be silently ignored), and each repository row must carry a
GitHub `.git` remote, a 40-character lowercase hex commit, and unique
non-traversing producer paths. The lock carries no schema-version field: its
only parser (`xtask`) ships in the same commit as the file, so tool/file skew
cannot occur, and shape changes fail closed at parse time.

Fixture provenance is validated per-file by globbing `parity/fixtures/**/*.json`:
every fixture must carry a unique `surface` and a `sources` and/or `standards`
header; each `sources` entry must cite a pinned repository at exactly the pinned
commit (the freshness ratchet) with refs confined to declared producer paths;
case-level `source_ref` strings may not carry commit segments; and
provenance-lookalike keys (`source`, `source_refs`, `@source_ref`) fail closed.
The vendored OpenAPI document's stamp must cite the services pin on every run, and
deep validation compares its body against the blob at that pin. The parse,
row-form, and fixture rules are pinned by the unit tests in
`xtask/src/parity/mod.rs`, which the shared quality gate runs through
`cargo test --workspace`.

### Fixture Wire-Value Fidelity

Each `parity/fixtures/**` payload value is a legal instance of the upstream schema
its `sources` header cites — response samples carry only fields the vendored
OpenAPI defines, and enum-valued fields use members the upstream producer actually
serializes — and each ref names the authoritative producer symbol for the value it
pins. SDK-side transform outputs with no upstream wire field (for example the
orderbook `total_fee` projection) stay out of the wire-shape fixtures and are
pinned by their own crate tests, so a fixture under a `services` header never
presents an SDK-only field as an upstream one.

### Amount Fixture Roundtrip

Amount-shaped fixture strings parse through the shared `cow_sdk_core::Amount` codec
and render back byte-identically. The orderbook wire-contract suite pins this
against the committed fixtures: every promoted amount DTO field round-trips
byte-for-byte through `Amount::new`, so a fixture value that drifts from its
canonical decimal form fails closed.

### Historical Snapshot Scope

Historical progress snapshots are review history, not active lifecycle authority.
They remain readable and unmodified while active preflight authority skips them by
a maintained directory-prefix exclusion policy — auditable, free of fragile
file-by-file suppression, with a single refresh point for future archive additions.

### Refresh Ownership

`.github/config/audit-refresh-map.yml` maps source-lock changes and the named
preflight exclusion policy to this audit. The public map records the review
contract without exposing maintainer-only path names.

## Evidence

Primary implementation points:

- `parity/source-lock.yaml`
- `parity/openapi/coverage.yaml`
- `parity/openapi/services-orderbook.yml`
- `parity/fixtures/orderbook/solver_execution.json`
- `crates/orderbook/src/types/order.rs`
- `.cargo/config.toml`
- `xtask/src/main.rs`
- `xtask/src/parity/mod.rs`
- `xtask/src/parity/sync.rs`
- `.github/workflows/_quality-gate.yml`
- `.github/workflows/upstream-drift.yml`
- `.github/config/audit-refresh-map.yml`
- `docs/audit/source-lock-provenance-audit.md`

Primary regression coverage:

- `xtask/src/parity/mod.rs::tests::malformed_source_lock_files_fail_closed`
- `xtask/src/parity/mod.rs::tests::fixtures_without_provenance_fail_closed`
- `xtask/src/parity/mod.rs::tests::fixtures_citing_a_stale_commit_trip_the_ratchet`
- `xtask/src/parity/mod.rs::tests::vendored_openapi_stamp_must_match_the_services_pin`
- `crates/orderbook/tests/wire_contract.rs::openapi_response_dtos_roundtrip_required_fixture_fields`
- `crates/orderbook/tests/wire_contract.rs::promoted_amount_dtos_roundtrip_byte_identical`

Validation surface:

```text
cargo parity-validate
cargo xtask parity sync
cargo xtask parity drift
cargo parity-validate --upstream-root <dir>
cargo test -p xtask
```
