# Source-Lock Provenance Audit

Status: Current
Last reviewed: 2026-05-06
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
| Re-affirmation outcome | CoW Protocol source-lock pins align with the 2026-05-04 upstream HEAD comparison, and native Alloy pins are tag-aligned for the reviewed dependency families | Conforms |
| Local-root warnings | Reviewer-supplied upstream roots are checked for independent git top-levels, expected remotes, and pinned `HEAD` commits without making repo-local validation depend on those roots | Conforms |
| Publication preflight | Source-lock validation metadata lists the complete package-family dry-run contract with local patches for unpublished intra-family crates | Conforms |
| Native Alloy provenance | `parity/source-lock.yaml` pins exact Alloy runtime and Alloy Core commits for source-derived dependency evidence used by the native adapter family | Conforms |
| Schema enforcement | Unsupported source-lock schema versions fail closed with a stable diagnostic, while schema version 3 is accepted | Conforms |
| Amount fixture roundtrip | Amount-shaped fixture strings parse through the shared `Amount` codec and round-trip byte-identically | Conforms |
| Historical snapshot scope | Historical progress snapshots stay readable and unmodified while active preflight authority skips them by directory-prefix policy | Conforms |
| Refresh mapping | The public audit-refresh map points source-lock changes and exclusion-policy changes back to this audit | Conforms |

## Current Contract

### Source-Lock Pins

`parity/source-lock.yaml` is the committed provenance contract for parity
fixtures and source-derived evidence. It currently pins:

- `cow-sdk` at `00c3dbd41c086ff9a51d5e5a30648615d4c66d0d`
- `contracts` at `c94c595a791681cf8ba7495117dcde397b932885`
- `services` at `0720b9bc15138ecc362078f505d0e3ba1c7b9883`
- `alloy` at `f3fe4cfff0553e9e234a53208bb69b7c222c66e5`
- `alloy-core` at `e6b30e4c2407cd1d2ea93e79f2768e5a4f21d266`

The lock is intentionally commit-based rather than branch-based. A release
claim that depends on upstream freshness has to compare these pins against the
upstream repositories before treating the evidence as current.

### Freshness State

Upstream HEADs were checked on 2026-05-04:

| Repository | Source-lock pin | Upstream HEAD | State |
| --- | --- | --- | --- |
| `cow-sdk` | `00c3dbd41c086ff9a51d5e5a30648615d4c66d0d` | `00c3dbd41c086ff9a51d5e5a30648615d4c66d0d` | Current |
| `contracts` | `c94c595a791681cf8ba7495117dcde397b932885` | `c94c595a791681cf8ba7495117dcde397b932885` | Current |
| `services` | `0720b9bc15138ecc362078f505d0e3ba1c7b9883` | `0720b9bc15138ecc362078f505d0e3ba1c7b9883` | Current |

All three pins are aligned with upstream HEAD for this review. Release claims
that depend on upstream freshness still have to rerun the comparison before
publication if any upstream repository moves again.

The Alloy runtime and Alloy Core pins are tag-aligned dependency evidence for
the native adapter family rather than CoW Protocol upstream parity evidence.
They are kept in the same source-lock contract so dependency provenance,
producer paths, and package dry-run metadata stay reviewable through the
existing validation gate.

### Release Re-affirmation

The 2026-05-04 pre-tag re-affirmation returned the same commit for every
source-lock repository:

| Repository | Source-lock pin | `git ls-remote ... HEAD` result | Action |
| --- | --- | --- | --- |
| `cow-sdk` | `00c3dbd41c086ff9a51d5e5a30648615d4c66d0d` | `00c3dbd41c086ff9a51d5e5a30648615d4c66d0d` | Re-affirmed; no bump |
| `contracts` | `c94c595a791681cf8ba7495117dcde397b932885` | `c94c595a791681cf8ba7495117dcde397b932885` | Re-affirmed; no bump |
| `services` | `0720b9bc15138ecc362078f505d0e3ba1c7b9883` | `0720b9bc15138ecc362078f505d0e3ba1c7b9883` | Re-affirmed; no bump |

This review does not mutate `parity/source-lock.yaml`. If a later release
candidate needs to move any upstream pin, that change remains a deliberate
reviewed pull request with rationale, followed by refreshed parity validation.

### Local-Root Warning Command

`cargo check-source-lock-roots` is a report-only policy-maintainer command for
reviewers who pass local upstream checkouts into provenance-sensitive
validation. When `--cow-sdk-root`, `--contracts-root`, or `--services-root` is
supplied, the command warns if the path resolves to a parent git checkout, if
the origin remote does not match `parity/source-lock.yaml`, or if `HEAD` does
not equal the pinned commit. The command intentionally emits warnings instead
of replacing `cargo parity-validate`; its purpose is to make suspicious local
root choices visible before reviewers rely on them.

### Refresh Outcome

The 2026-05-02 upstream comparison found `services` producer-path drift in
`crates/shared/src/order_validation.rs` and no producer-path drift in
`cow-sdk`, `contracts`, `crates/orderbook/openapi.yml`, or
`crates/orderbook/src/app_data.rs`. The source-lock was refreshed to the
current services HEAD, fixture provenance was aligned to the refreshed commit,
the services OpenAPI was re-vendored, and the solver-execution DTO coverage was
aligned with the committed OpenAPI `executedAmounts` payload shape.

### Publication Preflight Metadata

The validation metadata in `parity/source-lock.yaml` records the repo-local
package dry-run contract used before release evidence relies on the committed
parity fixtures. The contract covers the full published crate family, including
`cow-sdk-transport-wasm`, and patches unpublished local crate dependencies for
pre-publication dry-runs. In particular, `cow-sdk-contracts` patches
`cow-sdk-orderbook` and `cow-sdk-subgraph` because they are dev-dependencies of
the contracts crate, and `cow-sdk-trading` patches `cow-sdk-transport-wasm`
until the first package family has been published.

The package dry-run contract also covers `cow-sdk-alloy-provider`,
`cow-sdk-alloy-signer`, and `cow-sdk-alloy`, with `cow-sdk` patched to the
local adapter crates when validating the facade before publication.

### Schema Version Enforcement

The maintainer validates source-lock schema version 3 as the only supported
schema. The fixture tests in
`scripts/parity-maintainer/tests/source_lock_schema_version.rs` pin v2 and v4
rejection with the stable diagnostic substring `expected source-lock
schema_version 3`, and pin v3 acceptance against the current validation
contract. The shared quality gate now runs
`cargo test --manifest-path scripts/parity-maintainer/Cargo.toml`, so these
checks are CI-enforced with the rest of the maintainer suite.

### Cross-Fixture Amount Roundtrip

The workspace-level SDK integration test at
`crates/sdk/tests/cross_fixture_amount_roundtrip.rs` loads
`parity/fixtures/core.json`, `parity/fixtures/orderbook.json`, and
`parity/fixtures/trading.json`, collects amount-shaped strings, and asserts
they parse through `cow_sdk_core::Amount::new` with byte-identical display
roundtrips. When an identical hex string appears across fixture files, the
decoded `BigUint` value is compared across appearances.

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
- `parity/openapi/solver-execution-inventory.yaml`
- `parity/fixtures/orderbook/solver_execution.json`
- `crates/orderbook/src/types.rs`
- `.cargo/config.toml`
- `scripts/parity-maintainer/src/main.rs`
- `scripts/policy-maintainer/src/check_source_lock_roots.rs`
- `scripts/parity-maintainer/tests/fixtures/source-lock-v2.yaml`
- `scripts/parity-maintainer/tests/fixtures/source-lock-v3.yaml`
- `scripts/parity-maintainer/tests/fixtures/source-lock-v4.yaml`
- `crates/sdk/tests/cross_fixture_amount_roundtrip.rs`
- `.github/workflows/_quality-gate.yml`
- `.github/config/audit-refresh-map.yml`
- `docs/audit/source-lock-provenance-audit.md`

Primary regression coverage:

- Maintainer-side exclusion tests cover exclusion-list loading, directory-prefix
  skipping, and rejection of file-level entries.
- `scripts/parity-maintainer/tests/source_lock_schema_version.rs::source_lock_with_schema_v2_is_rejected_with_stable_diagnostic`
- `scripts/parity-maintainer/tests/source_lock_schema_version.rs::source_lock_with_schema_v3_is_accepted`
- `scripts/parity-maintainer/tests/source_lock_schema_version.rs::source_lock_with_schema_v4_is_rejected_with_stable_diagnostic`
- `crates/orderbook/tests/openapi_dto_coverage.rs::openapi_coverage_manifest_roundtrips_required_orderbook_dtos`
- `crates/orderbook/tests/wire_contract.rs::promoted_amount_dtos_roundtrip_byte_identical`
- `crates/sdk/tests/cross_fixture_amount_roundtrip.rs::cross_fixture_amount_roundtrip`

Validation surface:

```text
git ls-remote https://github.com/cowprotocol/services HEAD
git ls-remote https://github.com/cowprotocol/contracts HEAD
git ls-remote https://github.com/cowprotocol/cow-sdk HEAD
cargo parity-validate --source-lock parity/source-lock.yaml
cargo parity-check-freshness --source-lock parity/source-lock.yaml
cargo check-source-lock-roots --cow-sdk-root <cow-sdk-checkout> --contracts-root <contracts-checkout> --services-root <services-checkout>
cargo test --manifest-path scripts/parity-maintainer/Cargo.toml
cargo test --manifest-path scripts/policy-maintainer/Cargo.toml check_source_lock_roots
cargo test --workspace --all-features cross_fixture_amount_roundtrip
```
