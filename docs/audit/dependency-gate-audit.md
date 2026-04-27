# Dependency Gate Audit

Status: Current
Last reviewed: 2026-04-27
Owning surface: Release-facing dependency-audit gate for current published `cow-rs` surfaces
Refresh trigger: Changes to blocking dependency policy, Cargo.lock advisory posture, release or verification dependency commands, published CID dependency posture, transport crate advisory posture, or browser-wallet alloy advisory posture
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
- the clean published CID dependency posture recorded for `cow-sdk-app-data`
- the canonical advisory tolerance register shared by the RustSec gates
- the canonical dependency-source whitelist

It does not cover broader dependency-freshness reporting, license or source
policy details beyond the blocking gate split, or unrelated crate-specific
architecture reviews.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Transport advisories | The reqwest transport path resolves through a reviewed published `rustls-webpki` patch release | Conforms |
| Published CID posture | The app-data CID stack no longer reaches the yanked `core2` path after the `cid 0.11.3` uplift | Conforms |
| Gate ownership | `cargo deny` owns bans, licenses, sources, and yanked advisory policy, while `cargo audit` blocks vulnerabilities plus unsound and unmaintained advisories | Conforms |
| Advisory tolerance source | `.github/config/deny.toml` is the canonical RustSec ignore register; CI derives `cargo audit` ignore arguments from it | Conforms |
| Source whitelist | The dependency-source policy allows crates.io registry dependencies and rejects unknown registries and all git sources | Conforms |

## Current Contract

### Transport Advisory Remediation

The lockfile carries `rustls-webpki 0.103.13` across the reqwest transport
chain used by `cow-sdk-orderbook` and `cow-sdk-subgraph`, clearing the
reachable certificate-revocation-list parsing panic reported under
`RUSTSEC-2026-0104`. The reqwest pull chain into `rustls-platform-verifier`
resolves through the advisory-clean line without a workspace override.

### Published CID Posture

`cow-sdk-app-data` now reaches the refreshed published `cid 0.11.3` and
`multihash 0.19.5` path. That published path removes the prior yanked
`core2 0.4.0` reachability, so the CID stack no longer owns any RustSec
ignore entries. The retired `core2` advisory `RUSTSEC-2026-0105` is not
tolerated by the current gate, and the remaining `RUSTSEC-2026-0097`
tolerance is scoped to the browser-wallet alloy dependency chain.

### Gate Contract

Routine CI and release-readiness apply the same split dependency contract:
`cargo deny check --config .github/config/deny.toml` owns policy on allowed
sources, licenses, curated duplicate-version tolerances, and yanked advisory
handling, while `cargo audit --deny unsound --deny unmaintained` blocks RustSec
vulnerabilities plus unsound and unmaintained advisories. Three identifiers are
currently tolerated with documented revisit triggers:

- `RUSTSEC-2026-0097` — covered by
  [Browser-Wallet Alloy Dependency Audit](browser-wallet-alloy-dependency-audit.md)
- `RUSTSEC-2024-0388` — covered by
  [Browser-Wallet Alloy Dependency Audit](browser-wallet-alloy-dependency-audit.md)
- `RUSTSEC-2024-0436` — covered by
  [Browser-Wallet Alloy Dependency Audit](browser-wallet-alloy-dependency-audit.md)

Each ignore lives in `.github/config/deny.toml` under `[advisories].ignore`.
The shared quality gate reads that TOML register at runtime and derives the
`cargo audit --ignore ...` arguments from it, so closure or addition of a
reviewed advisory has one committed source of truth. The
`docs-agree-on-release-gates` guard compares the public command examples
against the same canonical register.

### Yanked Advisory Policy

`[advisories].yanked = "deny"` keeps yanked crate versions blocking in the
`cargo-deny` advisory gate. The current lockfile has no `core2` reachability
from the app-data CID stack, and any future yanked crate must be removed before
release.

### Source Whitelist

The dependency source policy is fail-closed: unknown registries are denied,
unknown git sources are denied, and the explicit git allow-list is empty. That
keeps the release-facing dependency graph anchored to the crates.io registry
plus first-party workspace paths.

## Evidence

Primary implementation points:

- `Cargo.lock`
- `.github/workflows/ci.yml`
- `.github/workflows/release-readiness.yml`
- `.github/workflows/_quality-gate.yml`
- `.github/config/deny.toml`
- `docs/release-checklist.md`
- `docs/verification-guide.md`
- `docs/verification-matrix.md`
- `docs/audit/cid-dependency-audit.md`

Validation surface:

```text
cargo deny check --config .github/config/deny.toml
cargo audit --deny unsound --deny unmaintained \
  --ignore RUSTSEC-2026-0097 \
  --ignore RUSTSEC-2024-0388 \
  --ignore RUSTSEC-2024-0436
cargo test -p cow-sdk-app-data
cargo test -p cow-sdk-orderbook
cargo test -p cow-sdk-subgraph
cargo test -p cow-sdk-browser-wallet
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-features --target wasm32-unknown-unknown
```
