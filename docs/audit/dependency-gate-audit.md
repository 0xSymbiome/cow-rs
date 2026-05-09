# Dependency Gate Audit

Status: Current
Last reviewed: 2026-05-09
Owning surface: Release-facing dependency-audit gate for current published `cow-rs` surfaces
Refresh trigger: Changes to blocking dependency policy, Cargo.lock advisory posture, release or verification dependency commands, published CID dependency posture, shared transport-policy dependencies, transport crate advisory posture, native Alloy two-family lockfile posture, ADR 0026 Alloy absorption rehearsal, or browser-wallet alloy advisory posture
Related docs:
- [ADR 0006](../adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)
- [CID Dependency Audit](cid-dependency-audit.md)
- [Browser-Wallet Alloy Dependency Audit](browser-wallet-alloy-dependency-audit.md)
- [Alloy Umbrella Adapter Audit](alloy-umbrella-adapter-audit.md)
- [Release Checklist](../release-checklist.md)
- [Verification Guide](../verification-guide.md)

## Scope

This audit covers:

- the dependency-audit gate used by routine CI and release-readiness validation
- the published `rustls-webpki` patch uplift on the orderbook and subgraph
  transport path
- the shared `cow-sdk-transport-policy` dependency boundary used by
  orderbook and subgraph retry behavior
- the clean published CID dependency posture recorded for `cow-sdk-app-data`
- the canonical advisory tolerance register shared by the RustSec gates
- the canonical dependency-source whitelist
- the workspace dependency default-feature audit
- the native Alloy provider and signer dependency allow-list gates for
  shipped crates
- the native Alloy runtime and Alloy Core ABI two-family lockfile invariant
- the release-doc guard that requires RustSec ignore rationale entries
- the report-only alloy release-candidate canary and its failure response
- the `cow-sdk-wasm` wasm32 dependency exclusion list for browser-wallet,
  native Alloy, reqwest, and hyper families

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
| Workspace default features | Root workspace dependencies either disable default features explicitly or appear in the reviewed exception register for dependencies without a meaningful default-feature control | Conforms |
| Ignore rationale lint | Every canonical RustSec ignore token must appear in this audit before release-doc agreement passes | Conforms |
| Direct WASM randomness | Direct crate use of `getrandom` for wasm32 is centralized on the workspace `0.4.2` pin with the `wasm_js` feature | Conforms |
| Shared transport policy | Retry timers and browser timer dependencies are centralized in `cow-sdk-transport-policy` instead of duplicated in orderbook or subgraph | Conforms |
| Workspace dependency inheritance | Shared helper pins for timers, browser panic hooks, and test HTTP fixtures are centralized in the workspace table | Conforms |
| Duplicate-version exceptions | Residual duplicate roots are documented as explicit skip-tree entries; stale `tiny-keccak` and `getrandom 0.2` exceptions were removed because they are no longer in the workspace graph | Conforms |
| Legacy `thiserror` reachability | The remaining `thiserror 1.0.69` path is limited to the `graphql_client` codegen chain used by dev/test coverage | Conforms |
| Native Alloy allow-lists | Shipped crates that depend on `alloy-provider` or `alloy-signer-local` are limited to the reviewed adapter crates and fail the policy-maintainer gate if the dependency escapes | Conforms |
| Native Alloy two-family lockfile | The workspace lockfile keeps reviewed Alloy runtime crates on `2.0.4` and Alloy Core ABI crates on `1.5.7`, with exactly one resolved version per listed crate | Conforms |
| Alloy canary failures | Scheduled canary failures are triaged as upstream-compatibility reports, with local pins changed only after ordinary quality gates pass and without dependency-policy waivers | Conforms |
| `cow-sdk-wasm` wasm32 tree | The wasm32 dependency graph excludes `cow-sdk-browser-wallet`, `cow-sdk-alloy*`, `alloy-provider`, reqwest, and hyper families; `tokio` is limited to the existing cancellation-token path | Conforms |

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
tolerated by the current gate, and the former `RUSTSEC-2026-0097` `rand`
tolerance is retired because the lockfile resolves the affected path through
`rand 0.8.6`.

### Gate Contract

Routine CI and release-readiness apply the same split dependency contract:
`cargo deny check --config .github/config/deny.toml` owns policy on allowed
sources, licenses, curated duplicate-version tolerances, and yanked advisory
handling, while `cargo audit --deny unsound --deny unmaintained` blocks RustSec
vulnerabilities plus unsound and unmaintained advisories. One identifier is
currently tolerated with a documented revisit trigger:

- `RUSTSEC-2024-0436` — covered by
  [Browser-Wallet Alloy Dependency Audit](browser-wallet-alloy-dependency-audit.md)

Each ignore lives in `.github/config/deny.toml` under `[advisories].ignore`.
The shared quality gate reads that TOML register at runtime and derives the
`cargo audit --ignore ...` arguments from it, so closure or addition of a
reviewed advisory has one committed source of truth. The
`docs-agree-on-release-gates` guard compares the public command examples
against the same canonical register and fails if any ignored RustSec token
lacks a matching rationale in this audit.

### Workspace Default-Feature Policy

Root workspace dependencies that expose meaningful default features are
declared with `default-features = false` and explicit features. Dependencies
that are intentionally kept on their defaults, or that expose no useful
default-feature switch, must be listed in
`tests/dependency_default_features_audit.rs` as reviewed exceptions. The
current exception set includes browser/WASM bridge crates such as
`serde-wasm-bindgen`, whose resolved manifest has no configurable feature
surface for this policy to narrow.

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

### Direct WASM Randomness Alignment

The workspace carries `getrandom 0.4.2` with the `wasm_js` feature as the
canonical first-party direct dependency for wasm32 consumers.
`cow-sdk-browser-wallet` and `cow-sdk-contracts` use the
workspace dependency instead of carrying leaf-local direct pins. The shared
Alloy workspace pins keep their default `std` features disabled so the
contracts crate can enable alloy-primitives' `k256` feature without also
activating the upstream `k256` / `rand_core 0.6` `getrandom 0.2` std path on
wasm32 builds.

### Workspace Dependency Inheritance

The workspace dependency table centralizes the shared `wiremock`,
`web-time`, `gloo-timers`, `futures-timer`, and
`console_error_panic_hook` pins. `cow-sdk-transport-policy` owns the retry
timer dependencies used by orderbook and subgraph. Consumer manifests inherit
those pins through workspace dependencies, keeping the reviewed versions in one
place while preserving the existing target-specific dependency boundaries.

### Duplicate-Version Exceptions

The duplicate-version policy is fail-closed except for reviewed skip-tree
roots that document why the duplicate is currently retained. The current
register covers the upstream-owned `getrandom 0.3.4` transitive root, `winnow
0.7.15` under the alloy Solidity parser chain, and the reviewed
browser-wallet alloy advisory roots. The retained `getrandom 0.3.4` path is
upstream-owned validation and TLS build-support debt, not the first-party
randomness API. The retired `tiny-keccak` license exception and stale
`getrandom 0.2` duplicate exception are gone because the workspace graph no
longer reaches them. `Cargo.lock` can still carry inactive package metadata
for `getrandom 0.2.17` through `rustls-webpki` / `ring`, but
`cargo tree --workspace --target all --all-features -i getrandom:0.2.17`
prints no dependency path and no first-party crate aliases that package.

### Legacy Thiserror Reachability

`thiserror 1.0.69` is no longer reachable through the native examples lockfile
after the examples-native lock regeneration. In the workspace lockfile, the
remaining old line is reached through `graphql-parser -> graphql_client_codegen
-> graphql_query_derive -> graphql_client`, which is used by dev/test
coverage for the subgraph and contracts crates. The release-facing gate keeps
the path visible as duplicate-version debt rather than hiding it behind an
advisory tolerance.

### Native Alloy Dependency Allow-Lists

The native Alloy adapters are the only shipped crates allowed to carry
`alloy-provider` or `alloy-signer-local` reachability. The policy-maintainer
commands `cargo check-alloy-provider-invariant` and
`cargo check-alloy-signer-invariant` parse the inverse dependency tree and
fail if those runtime dependencies appear in any other shipped `cow-sdk*`
crate. The umbrella crate is allow-listed because it intentionally composes
the reviewed provider and signer leaves into one native client surface.

### Native Alloy Two-Family Lockfile Invariant

The lockfile invariant treats Alloy runtime crates and Alloy Core ABI crates as
separate reviewed dependency families. Runtime crates are pinned at `2.0.4`,
Core ABI crates are pinned at `1.5.7`, and the workspace regression test fails
if any listed crate resolves to a second version. This keeps ABI conversion
changes from entering through a runtime-only update and keeps runtime transport
changes from entering through an ABI-only update.

### Alloy Canary Failure Response

The alloy release-candidate canary is report-only and has no pull-request
trigger. A failed scheduled run is triaged as upstream compatibility drift:
inspect the workflow summary and failing crate, decide whether the failure is
caused by an upstream release-candidate regression or by a required local
adaptation, and keep the committed workspace pins unchanged until the ordinary
quality gates pass against a reviewed update. The workflow creates or reuses a
tracking issue through `gh api` using the repository token and `issues: write`;
it does not introduce a new third-party action. Do not add a RustSec ignore,
license exception, source exception, or `alloy-provider` dependency waiver in
response to the canary alone. If a local change is needed, it must preserve the
published-crate invariant that no shipped leaf crate transitively depends on
`alloy-provider`.

### `cow-sdk-wasm` Dependency Boundary

`cow-sdk-wasm` is a peer leaf of `cow-sdk-browser-wallet`,
`cow-sdk-transport-wasm`, and the native Alloy adapter family. Its wasm32
dependency tree must not pull browser-wallet, native Alloy provider/signer
crates, reqwest, hyper, or native Alloy RPC transport families. The workspace
test reads cargo metadata for the wasm32 target and fails if any forbidden
crate appears in the dependency closure. This keeps the TypeScript-callable
crate browser-safe and preserves the native Alloy adapter boundary.

## Evidence

Primary implementation points:

- `Cargo.lock`
- `.github/workflows/alloy-release-candidate.yml`
- `.github/workflows/ci.yml`
- `.github/workflows/release-readiness.yml`
- `.github/workflows/_quality-gate.yml`
- `.github/config/deny.toml`
- `tests/dependency_default_features_audit.rs`
- `tests/alloy_two_family_lockfile_invariant.rs`
- `tests/wasm_dependency_invariant.rs`
- `scripts/check-release-docs-agree.sh`
- `scripts/policy-maintainer/src/check_alloy_provider_invariant.rs`
- `scripts/policy-maintainer/src/check_alloy_signer_invariant.rs`
- `docs/release-checklist.md`
- `docs/verification-guide.md`
- `docs/verification-matrix.md`
- `docs/audit/cid-dependency-audit.md`
- `docs/audit/browser-wallet-alloy-dependency-audit.md`
- `crates/wasm/Cargo.toml`
- `crates/browser-wallet/Cargo.toml`
- `crates/contracts/Cargo.toml`
- `crates/orderbook/Cargo.toml`
- `examples/native/Cargo.lock`

Validation surface:

```text
cargo deny check --config .github/config/deny.toml
cargo audit --deny unsound --deny unmaintained \
  --ignore RUSTSEC-2024-0436
cargo tree --workspace --invert thiserror:1.0.69 -e no-build
cargo check-alloy-provider-invariant
cargo check-alloy-signer-invariant
cargo test -p cow-rs-workspace-tests --test alloy_two_family_lockfile_invariant
gh workflow run alloy-release-candidate.yml
cargo build --workspace --all-features
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-features --target wasm32-unknown-unknown
cargo test -p cow-rs-workspace-tests --test dependency_default_features_audit
bash scripts/check-release-docs-agree.sh
```
