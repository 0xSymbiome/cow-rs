# Source-Lock Provenance Audit

Status: Current
Last reviewed: 2026-05-01
Owning surface: source-lock provenance and lifecycle preflight authority
Refresh trigger: Changes to `parity/source-lock.yaml`, any change to the maintained exclusion-list policy for historical progress snapshots, or any newly archived progress snapshot that should stay outside active preflight authority
Related docs:
- [ADR 0026](../adr/0026-alloy-major-release-absorption-plan.md)
- [ADR 0030](../adr/0030-workspace-locked-versioning-tag-baseline.md)

## Scope

This audit covers:

- the committed source-lock pins that define upstream provenance for parity
  fixtures and source-derived review evidence
- the current upstream HEAD comparison used to make source-lock freshness
  explicit before release evidence relies on it
- the source-lock refresh outcome for the first functional release evidence
- the repo-local package dry-run command contract embedded in source-lock
  validation metadata
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
| Refresh outcome | Source-lock pins are aligned with current upstream commits after the 2026-04-29 refresh | Conforms |
| Publication preflight | Source-lock validation metadata lists the complete package-family dry-run contract with local patches for unpublished intra-family crates | Conforms |
| Historical snapshot scope | Historical progress snapshots stay readable and unmodified while active preflight authority skips them by directory-prefix policy | Conforms |
| Refresh mapping | The public audit-refresh map points source-lock changes and exclusion-policy changes back to this audit | Conforms |

## Current Contract

### Source-Lock Pins

`parity/source-lock.yaml` is the committed provenance contract for parity
fixtures and source-derived evidence. It currently pins:

- `cow-sdk` at `00c3dbd41c086ff9a51d5e5a30648615d4c66d0d`
- `contracts` at `c94c595a791681cf8ba7495117dcde397b932885`
- `services` at `bf40548684828ad72c1e10fbe8fe3467c90eba45`

The lock is intentionally commit-based rather than branch-based. A release
claim that depends on upstream freshness has to compare these pins against the
upstream repositories before treating the evidence as current.

### Freshness State

Upstream HEADs were checked on 2026-04-29 after the refresh:

| Repository | Source-lock pin | Upstream HEAD | State |
| --- | --- | --- | --- |
| `cow-sdk` | `00c3dbd41c086ff9a51d5e5a30648615d4c66d0d` | `00c3dbd41c086ff9a51d5e5a30648615d4c66d0d` | Current |
| `contracts` | `c94c595a791681cf8ba7495117dcde397b932885` | `c94c595a791681cf8ba7495117dcde397b932885` | Current |
| `services` | `bf40548684828ad72c1e10fbe8fe3467c90eba45` | `bf40548684828ad72c1e10fbe8fe3467c90eba45` | Current |

The source-lock pins are aligned with upstream commits at `cow-sdk @
00c3dbd4`, `contracts @ c94c595a`, and `services @ bf405486`.

### Refresh Outcome

The 2026-04-29 upstream comparison found producer-path updates in `cow-sdk`
and `services`, plus no producer-path drift in `contracts`. The source-lock was
refreshed instead of retained at the older pins, fixture provenance was aligned
to the refreshed commits, and the services OpenAPI was re-vendored. The covered
orderbook DTO inventory remained unchanged after the OpenAPI refresh.

### Publication Preflight Metadata

The validation metadata in `parity/source-lock.yaml` records the repo-local
package dry-run contract used before release evidence relies on the committed
parity fixtures. The contract covers the full published crate family, including
`cow-sdk-transport-wasm`, and patches unpublished local crate dependencies for
pre-publication dry-runs. In particular, `cow-sdk-contracts` patches
`cow-sdk-orderbook` and `cow-sdk-subgraph` because they are dev-dependencies of
the contracts crate, and `cow-sdk-trading` patches `cow-sdk-transport-wasm`
until the first package family has been published.

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

## Pending verification evidence

This section records evidence expected from the next verification refresh. It
is removed once every permanent evidence pointer has landed in the sections
above.

- `scripts/parity-maintainer/tests/source_lock_schema_version.rs::source_lock_with_schema_v2_is_rejected_with_stable_diagnostic`
  and
  `scripts/parity-maintainer/tests/source_lock_schema_version.rs::source_lock_with_schema_v4_is_rejected_with_stable_diagnostic`
  will pin deterministic rejection for unsupported source-lock schema versions.
- `scripts/parity-maintainer/tests/source_lock_schema_version.rs::source_lock_with_schema_v3_is_accepted`
  will pin acceptance of the current schema version.
- `.github/workflows/_quality-gate.yml` will run
  `cargo test --manifest-path scripts/parity-maintainer/Cargo.toml` so the
  maintainer-side provenance tests are enforced by the quality gate.
- `crates/sdk/tests/cross_fixture_amount_roundtrip.rs` will pin canonical
  amount parsing and byte-identical roundtrips across the committed parity
  fixtures.

## Evidence

Primary implementation points:

- `parity/source-lock.yaml`
- `scripts/parity-maintainer/src/main.rs`
- `.github/config/audit-refresh-map.yml`
- `docs/audit/source-lock-provenance-audit.md`

Primary regression coverage:

- Maintainer-side exclusion tests cover exclusion-list loading, directory-prefix
  skipping, and rejection of file-level entries.

Validation surface:

```text
git ls-remote https://github.com/cowprotocol/services HEAD
git ls-remote https://github.com/cowprotocol/contracts HEAD
git ls-remote https://github.com/cowprotocol/cow-sdk HEAD
cargo parity-validate --source-lock parity/source-lock.yaml
cargo test --manifest-path scripts/parity-maintainer/Cargo.toml
```
