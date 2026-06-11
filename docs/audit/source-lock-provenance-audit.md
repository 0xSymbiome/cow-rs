# Source-Lock Provenance Audit

Status: Current
Last reviewed: 2026-06-10
Owning surface: source-lock provenance and release preflight authority
Refresh trigger: Changes to `parity/source-lock.yaml`, vendored parity OpenAPI or fixture provenance, any change to the maintained exclusion-list policy for historical progress snapshots, or any newly archived progress snapshot that should stay outside active preflight authority
Related docs:
- [ADR 0026](../adr/0026-alloy-major-release-absorption-plan.md)
- [ADR 0030](../adr/0030-workspace-locked-versioning-tag-baseline.md)
- [Alloy Umbrella Adapter Audit](alloy-umbrella-adapter-audit.md)

## Scope

This audit covers:

- the committed source-lock pins that define upstream provenance for parity
  fixtures and source-derived review evidence
- the current upstream HEAD comparison used to make source-lock freshness
  explicit before release evidence relies on it
- the source-lock refresh outcome for the first functional release evidence
- the report-only local-root warning command for reviewer-supplied upstream
  checkouts
- the repo-local package dry-run command contract embedded in source-lock
  validation metadata
- the native Alloy runtime and core upstream pins used for source-derived
  dependency evidence and release-candidate validation
- the exclusion-list rule that keeps historical progress snapshots readable but
  outside active preflight path-normalization authority
- the audit-refresh mapping that points provenance changes back to this record

It does not cover future source-lock refreshes, fixture authoring methodology,
or changing SDK behavior.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Source-lock pins | `parity/source-lock.yaml` pins exact upstream commits for every repository that contributes parity evidence | Conforms |
| Freshness disclosure | Current upstream HEADs are checked explicitly so stale pins are visible before release evidence relies on freshness | Conforms |
| Refresh outcome | The 2026-05-29 sync advanced the two CoW Protocol pins (`contracts`, `services`) to upstream HEAD, re-vendored the services OpenAPI, and re-aligned fixture provenance; parity validation and OpenAPI coverage pass, and the `git ls-remote` upstream HEAD comparison shows both pins Current | Conforms |
| Deep upstream-root validation | Reviewer-supplied upstream roots (`--upstream-root <dir>`, one checkout per lock repository) are fail-closed checked for independent git top-levels, expected remotes, pinned `HEAD` commits, clean producer paths, and the vendored OpenAPI body at the services pin, without making repo-local validation depend on those roots | Conforms |
| Publication preflight | The package-family dry-run contract (with local patches for unpublished intra-family crates) lives in the release-readiness publication job, which validates the committed lock before the dry runs | Conforms |
| Native Alloy provenance | The native adapter family pins Alloy by crates.io version (`alloy-* = 2.0.4`, `alloy-core-* = 1.5.7`), enforced by `Cargo.lock` and the two-family lockfile invariant | Conforms |
| App-data schema drift fixtures | `parity/fixtures/app_data/schemas/` retains one self-contained drift fixture per modeled metadata family for the typed metadata structs, with lock-validated provenance headers (the flash-loan mirror cites its real producer, `services`) | Conforms |
| Form enforcement | The lock parses through typed models that reject unknown or missing fields, row rules fail closed on malformed remotes, commits, or paths, and every fixture under `parity/fixtures/**/*.json` is validated per-file against the pins | Conforms |
| Amount fixture roundtrip | Amount-shaped fixture strings parse through the shared `Amount` codec and round-trip byte-identically | Conforms |
| Historical snapshot scope | Historical progress snapshots stay readable and unmodified while active preflight authority skips them by directory-prefix policy | Conforms |
| Binding source pins | The upstream Solidity repository each `cow-sdk-contracts` inline `alloy::sol!` binding mirrors is pinned by commit under `repositories:` in `parity/source-lock.yaml`; the bindings themselves are proven byte-for-byte by the fixtures' `sources` headers and crate parity tests rather than a per-file source mirror | Conforms |
| Refresh mapping | The public audit-refresh map points source-lock changes and exclusion-policy changes back to this audit | Conforms |

## Current Contract

### Source-Lock Pins

`parity/source-lock.yaml` is the committed provenance contract for parity
fixtures and source-derived evidence. It currently pins:

- `contracts` at `c6b61ce75841ce4c25ab126def9cc981c568e6c6`
- `services` at `1f80d54bc3521b3fa81cd8ad66d9f749c5450591`
- `cow-sdk` at `1c3c9619c3d0ee832ce43a2d695ad650c2ec7a18`
- `cow-shed` at `9e01a88e0010314ee1e4c1a822105897a87d3bda`
- `ethflowcontract` at `762d182674f8f890bd27917872ee62125171b54d`
- `app-data` at `31f130d1838ea5018facdfe240aef46ff0cc1881`

The `app-data` row closes the last unpinned authority: the hooks parity
fixture cites its schema families, and the `parity/fixtures/app_data/schemas/`
drift mirrors are refreshed from this pin rather than from "a real checkout" with no
recorded commit.

This review removed the deferred composable-order pins (`composable-cow` and
its `lib/safe` submodule row): the SDK ships no composable surface, no fixture
cites their paths, and the deferral is recorded by ADR 0048 — the capability
re-pins its upstream when it lands. The cow-shed row was trimmed to the files
the inline bindings and address tables actually mirror; two stale paths that
do not exist at the pinned commit (`src/interfaces/ICOWAuthHook.sol`,
`src/interfaces/IERC1271.sol`) were removed with it.

The lock is intentionally commit-based rather than branch-based. A release
claim that depends on upstream freshness has to compare these pins against the
upstream repositories before treating the evidence as current.

### Freshness State

Upstream HEADs were checked on 2026-05-29:

| Repository | Source-lock pin | Upstream HEAD | State |
| --- | --- | --- | --- |
| `contracts` | `c6b61ce75841ce4c25ab126def9cc981c568e6c6` | `c6b61ce75841ce4c25ab126def9cc981c568e6c6` | Current |
| `services` | `1f80d54bc3521b3fa81cd8ad66d9f749c5450591` | `1f80d54bc3521b3fa81cd8ad66d9f749c5450591` | Current |

The source lock remains intentionally commit-based. In this review the two
CoW Protocol pins (contracts and services) were advanced to upstream HEAD, so no freshness drift remains
for parity evidence to triage.

### App-Data Schema Drift Fixtures

`parity/fixtures/app_data/schemas/` holds one self-contained drift fixture per
modeled metadata family (`flashloan`, `partnerFee`, `quote`, and the `hook`
shape) under lock-validated provenance headers. The `hook`/`quote`/`partnerFee`
mirrors cite the pinned `cowprotocol/app-data` schema files; the flash-loan
mirror cites its real producer — the `services` `Flashloan` hint struct —
because `cowprotocol/app-data` defines no schema with those fields. They are
test-only fixtures, not resolved at runtime: validation is typed by
construction (ADR 0064). The `schema_drift_contract` regression test
field-name-probes each fixture so an upstream rename of a field the typed
structs depend on fails at review time.

### Release Re-affirmation

The 2026-05-29 refresh returned:

| Repository | Source-lock pin | `git ls-remote ... HEAD` result | Action |
| --- | --- | --- | --- |
| `contracts` | `c6b61ce75841ce4c25ab126def9cc981c568e6c6` | `c6b61ce75841ce4c25ab126def9cc981c568e6c6` | Advanced to HEAD |
| `services` | `1f80d54bc3521b3fa81cd8ad66d9f749c5450591` | `1f80d54bc3521b3fa81cd8ad66d9f749c5450591` | Advanced to HEAD |

This review advances the two CoW Protocol pins to upstream HEAD as a
deliberate, reviewed change, followed by refreshed parity validation. Future
pin moves remain deliberate reviewed changes with rationale and re-run parity
validation.

### Deep Upstream-Root Validation

`cargo parity-validate --upstream-root <dir>` is the fail-closed check for
reviewers who pass local upstream checkouts into provenance-sensitive
validation. It requires one checkout per lock repository at `<dir>/<id>` (the
layout `cargo xtask parity sync --root <dir>` materializes) and fails if any
path resolves to a parent git checkout, the origin remote does not match
`parity/source-lock.yaml`, `HEAD` does not equal the pinned commit, a producer
path is missing or dirty, or the vendored OpenAPI body does not match
`crates/orderbook/openapi.yml` at the services pin. The earlier report-only
`check-source-lock-roots` policy was retired with this change: it could not
fail in any lane that ran it (both `policy all` and the release workflow
invoked it without roots), and every condition it warned about is now enforced
fail-closed here.

### Refresh Outcome

The 2026-05-29 upstream comparison advanced `contracts` and `services` to
upstream HEAD. Every `contracts` producer path and pinned binding source is
unchanged at the new commit, so the inline `alloy::sol!` contract bindings and
their fixture proofs are unaffected.
`services` producer-path drift is confined to
`crates/shared/src/order_validation.rs` and `crates/orderbook/openapi.yml`; the
OpenAPI change removes the deprecated v1 `solver_competition` paths (the
`SolverCompetitionResponse` schema and the v2 routes are retained), expands the
`SimulationRequest` schema, and rewords the quote `timeout` description, while
every quote and order DTO schema (`OrderParameters`,
`OrderQuoteRequest`/`OrderQuoteResponse`/`OrderQuoteSide`/`OrderQuoteValidity`,
`PriceQuality`) is unchanged. The services OpenAPI was re-vendored, fixture
provenance was aligned to the refreshed commits, and OpenAPI DTO coverage was
re-validated.

### Publication Preflight

The package-family dry-run contract lives in the release-readiness
publication job (`.github/workflows/release-readiness.yml`): it validates the
committed lock (`cargo parity-validate`), then dry-run packages and publishes
the full published crate family with local patches for unpublished
intra-family dependencies. The source lock intentionally carries no
metadata block; its purpose is stated in a header comment, and the dry-run
contract is workflow-owned, not lock-embedded.

### Form Enforcement

`xtask` validates the source lock by form rather than by matching it against a
hardcoded contract: the typed model rejects unknown or missing fields
(`deny_unknown_fields`, so a misspelled key — or the retired `fixtures:`
section — cannot be silently ignored), and each repository row must carry a
GitHub `.git` remote, a 40-character lowercase hex commit, and unique
non-traversing producer paths. The lock carries no schema-version field: its
only parser (`xtask`) ships in the same commit as the file, so tool/file skew
cannot occur, and shape changes fail closed at parse time.

Fixture provenance is validated per-file by globbing
`parity/fixtures/**/*.json`: every fixture must carry a unique `surface` and a
`sources` and/or `standards` header; each `sources` entry must cite a pinned
repository at exactly the pinned commit (the freshness ratchet that names
every stale fixture after a pin bump) with refs confined to declared producer
paths; case-level `source_ref` strings may not carry commit segments; and
provenance-lookalike keys (`source`, `source_refs`, `@source_ref`) fail closed
— unknown keys are payload by design, so a provenance-shaped key the grammar
does not know would otherwise sit unvalidated while looking validated. The
vendored OpenAPI document's stamp must cite
the services pin on every run, and deep validation compares its body against
the blob at that pin. The parse, row-form, and fixture rules are pinned by the
unit tests in `xtask/src/parity/mod.rs`, which the shared quality gate runs
through `cargo test --workspace`.

### Cross-Fixture Amount Roundtrip

The workspace-level SDK integration test at
`crates/sdk/tests/amount_roundtrip.rs` pins the atomic-unit `Amount`
round-trip invariant directly against canonical literals rather than any
fixture file: a representative set of decimal atomic-unit strings — zero,
one, common token magnitudes, and the full uint256 ceiling — parse through
`cow_sdk_core::Amount::new` and render back byte-identically, and the parse
is deterministic, so the same literal always decodes to the same typed
`Amount`, compared bit-for-bit on its inner `U256`.

### Historical Snapshot Scope

Historical progress snapshots are review history, not active lifecycle
authority. They remain readable and are not rewritten in place for path
normalization. Active preflight authority uses a maintained directory-prefix
exclusion policy for those snapshots, while active strategy authority remains
in scope for normalization and validation.

The exclusion policy is deliberately directory-prefix based. That keeps the
rule auditable, avoids fragile file-by-file suppression, and gives future
archive additions a single refresh point.

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
- `crates/sdk/tests/cross_fixture_amount_roundtrip.rs`
- `.github/workflows/_quality-gate.yml`
- `.github/workflows/upstream-drift.yml`
- `.github/config/audit-refresh-map.yml`
- `docs/audit/source-lock-provenance-audit.md`

Primary regression coverage:

- Maintainer-side exclusion tests cover exclusion-list loading, directory-prefix
  skipping, and rejection of file-level entries.
- `xtask/src/parity/mod.rs::tests::malformed_source_lock_files_fail_closed`
- `xtask/src/parity/mod.rs::tests::fixtures_without_provenance_fail_closed`
- `xtask/src/parity/mod.rs::tests::fixtures_citing_a_stale_commit_trip_the_ratchet`
- `xtask/src/parity/mod.rs::tests::vendored_openapi_stamp_must_match_the_services_pin`
- `crates/orderbook/tests/wire_contract.rs::openapi_response_dtos_roundtrip_required_fixture_fields`
- `crates/orderbook/tests/wire_contract.rs::promoted_amount_dtos_roundtrip_byte_identical`
- `crates/sdk/tests/cross_fixture_amount_roundtrip.rs::cross_fixture_amount_roundtrip`

Validation surface:

```text
cargo parity-validate
cargo xtask parity sync
cargo xtask parity drift
cargo parity-validate --upstream-root <dir>
cargo test -p xtask
cargo test --workspace --all-features cross_fixture_amount_roundtrip
```
