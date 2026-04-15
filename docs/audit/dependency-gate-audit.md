# Dependency Gate Audit

Status: Current  
Last reviewed: 2026-04-15  
Owning surface: Release-facing dependency-audit gate for current published `cow-rs` surfaces  
Refresh trigger: Changes to blocking dependency policy, Cargo.lock advisory posture, release or verification dependency commands, or the current published CID warning status  
Related docs:
- [CID Dependency Audit](cid-dependency-audit.md)
- [Release Checklist](../release-checklist.md)
- [Verification Guide](../verification-guide.md)

## Scope

This audit covers:

- the dependency-audit gate used by routine CI and release-readiness validation
- the published `rustls-webpki` patch uplift on the orderbook and subgraph
  transport path
- the current published CID dependency warning recorded for `cow-sdk-app-data`

It does not cover broader dependency-freshness reporting, license or source
policy details beyond the blocking gate split, or unrelated crate-specific
architecture reviews.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Transport advisories | The reqwest transport path resolves through a reviewed published `rustls-webpki` patch release | Conforms |
| Published CID warning | The latest published `cid` release still reaches yanked `core2`, so that state remains an explicit reviewed warning rather than a hidden override | Reviewed warning |
| Gate ownership | `cargo deny` owns bans, licenses, and sources, while `cargo audit` blocks vulnerabilities plus unsound and unmaintained advisories | Conforms |

## Current Contract

### Transport Advisory Remediation

The lockfile carries the published `rustls-webpki` patch release that clears
the current RustSec findings on the reqwest transport chain used by
`cow-sdk-orderbook` and `cow-sdk-subgraph`.

### Published CID Warning Treatment

`cow-sdk-app-data` reaches the refreshed published `multihash` path, but the
remaining `core2 0.4.0` reachability still comes from the latest published
`cid 0.11.1` release. The repository therefore keeps that state visible as a
reviewed warning instead of masking it with an unreleased dependency override
or a local fork.

### Gate Contract

Routine CI and release-readiness apply the same split dependency contract:
`cargo deny check bans licenses sources --config .github/config/deny.toml` owns
policy on allowed sources, licenses, and curated duplicate-version tolerances,
while `cargo audit --deny unsound --deny unmaintained --ignore
RUSTSEC-2026-0097` blocks RustSec vulnerabilities plus unsound and unmaintained
advisories. This keeps real supply-chain regressions blocking without treating
the current published-only CID warning as silent policy drift.

## Evidence

Primary implementation points:

- `Cargo.lock`
- `.github/workflows/ci.yml`
- `.github/workflows/release-readiness.yml`
- `docs/release-checklist.md`
- `docs/verification-guide.md`
- `docs/verification-matrix.md`
- `docs/audit/cid-dependency-audit.md`

Validation surface:

```text
cargo deny check bans licenses sources --config .github/config/deny.toml
cargo audit --deny unsound --deny unmaintained --ignore RUSTSEC-2026-0097
cargo test -p cow-sdk-app-data
cargo test -p cow-sdk-orderbook
cargo test -p cow-sdk-subgraph
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-features --target wasm32-unknown-unknown
```
