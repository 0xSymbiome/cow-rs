# Dependency Gate Audit

Status: Current  
Last reviewed: 2026-04-22  
Owning surface: Release-facing dependency-audit gate for current published `cow-rs` surfaces  
Refresh trigger: Changes to blocking dependency policy, Cargo.lock advisory posture, release or verification dependency commands, the current published CID warning status, the transport crate advisory posture, or the alloy proc-macro advisory posture  
Related docs:
- [ADR 0006](../adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)
- [CID Dependency Audit](cid-dependency-audit.md)
- [Browser-Wallet Alloy Dependency Audit](browser-wallet-alloy-dependency-audit.md)
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

The lockfile carries `rustls-webpki 0.103.13` across the reqwest transport
chain used by `cow-sdk-orderbook` and `cow-sdk-subgraph`, clearing the
reachable certificate-revocation-list parsing panic reported under
`RUSTSEC-2026-0104`. The reqwest pull chain into `rustls-platform-verifier`
resolves through the advisory-clean line without a workspace override.

### Published CID Warning Treatment

`cow-sdk-app-data` reaches the refreshed published `multihash` path, but the
remaining `core2 0.4.0` reachability still comes from the latest published
`cid 0.11.1` release. That `core2` release is now additionally flagged
unmaintained and yanked under `RUSTSEC-2026-0105`. The repository keeps that
state visible as a reviewed warning instead of masking it with an unreleased
dependency override or a local fork.

### Gate Contract

Routine CI and release-readiness apply the same split dependency contract:
`cargo deny check bans licenses sources --config .github/config/deny.toml` owns
policy on allowed sources, licenses, and curated duplicate-version tolerances,
while `cargo audit --deny unsound --deny unmaintained` blocks RustSec
vulnerabilities plus unsound and unmaintained advisories. Four identifiers
are currently tolerated with documented revisit triggers:

- `RUSTSEC-2026-0097` — covered by
  [CID Dependency Audit](cid-dependency-audit.md)
- `RUSTSEC-2024-0388` — covered by
  [Browser-Wallet Alloy Dependency Audit](browser-wallet-alloy-dependency-audit.md)
- `RUSTSEC-2024-0436` — covered by
  [Browser-Wallet Alloy Dependency Audit](browser-wallet-alloy-dependency-audit.md)
- `RUSTSEC-2026-0105` — covered by
  [CID Dependency Audit](cid-dependency-audit.md)

Each ignore is mirrored in `.github/config/deny.toml` under
`[advisories].ignore` and in the `cargo audit --ignore ...` arguments in
`.github/workflows/ci.yml` and `.github/workflows/release-readiness.yml`.
Every ignore carries a matching revisit comment pointing to its owning
audit so policy drift stays reviewable in one place.

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
cargo audit --deny unsound --deny unmaintained \
  --ignore RUSTSEC-2026-0097 \
  --ignore RUSTSEC-2024-0388 \
  --ignore RUSTSEC-2024-0436 \
  --ignore RUSTSEC-2026-0105
cargo test -p cow-sdk-app-data
cargo test -p cow-sdk-orderbook
cargo test -p cow-sdk-subgraph
cargo test -p cow-sdk-browser-wallet
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-features --target wasm32-unknown-unknown
```
