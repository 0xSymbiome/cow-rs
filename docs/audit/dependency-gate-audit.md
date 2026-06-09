# Dependency Gate Audit

Status: Current
Last reviewed: 2026-06-09
Owning surface: Release-facing dependency-audit gate for current published `cow-rs` surfaces
Refresh trigger: Changes to blocking dependency policy, Cargo.lock advisory posture, release or verification dependency commands, published CID dependency posture, shared transport-policy dependencies, transport crate advisory posture, native Alloy two-family lockfile posture, ADR 0026 Alloy absorption rehearsal, the canonical primitive layer dependency closure per ADR 0052, or browser-wallet alloy advisory posture
Related docs:
- [ADR 0006](../adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)
- [ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md)
- [CID Dependency Audit](cid-dependency-audit.md)
- [Browser-Wallet Alloy Dependency Audit](browser-wallet-alloy-dependency-audit.md)
- [Alloy Umbrella Adapter Audit](alloy-umbrella-adapter-audit.md)
- [WASM Component Model Future Prep Audit](wasm-component-model-future-prep-audit.md)
- [Release Checklist](../release-checklist.md)
- [Verification Guide](../verification.md)

## Scope

This audit covers:

- the dependency-audit gate used by routine CI and release-readiness validation
- the published `rustls-webpki` patch uplift on the orderbook and subgraph
  transport path
- the shared `cow-sdk-transport-policy` dependency boundary used by
  orderbook and subgraph retry behavior
- the clean published CID dependency posture recorded for `cow-sdk-app-data` and `cow-sdk-core`
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
- the pure-helper crate dependency boundary that keeps deterministic wasm
  helpers free of JavaScript FFI dependencies

It does not cover broader dependency-freshness reporting, license or source
policy details beyond the blocking gate split, or unrelated crate-specific
architecture reviews.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Transport advisories | The reqwest transport path resolves through a reviewed published `rustls-webpki` patch release | Conforms |
| Published CID posture | The app-data CID stack no longer reaches the yanked `core2` path after the `cid 0.11.3` uplift, and `cow-sdk-app-data` consumes the published `cid`/`multihash`/`multibase` trio through shared workspace pins | Conforms |
| Gate ownership | `cargo deny` owns bans, licenses, sources, and yanked advisory policy, while `cargo audit` blocks vulnerabilities plus unsound and unmaintained advisories | Conforms |
| Advisory tolerance source | `.github/config/deny.toml` is the canonical RustSec ignore register; CI derives `cargo audit` ignore arguments from it | Conforms |
| Source whitelist | The dependency-source policy allows crates.io registry dependencies and rejects unknown registries and all git sources | Conforms |
| Workspace default features | Root workspace dependencies either disable default features explicitly or appear in the reviewed exception register for dependencies without a meaningful default-feature control | Conforms |
| Ignore rationale lint | Every canonical RustSec ignore token must appear in this audit before release-doc agreement passes | Conforms |
| Direct WASM randomness | Direct crate use of `getrandom` for wasm32 is centralized on the workspace `0.4.2` pin with the `wasm_js` feature | Conforms |
| Shared transport policy | Retry timers and browser timer dependencies are centralized in `cow-sdk-transport-policy` instead of duplicated in orderbook or subgraph | Conforms |
| Workspace dependency inheritance | Shared helper pins for timers, browser panic hooks, and test HTTP fixtures are centralized in the workspace table | Conforms |
| Duplicate-version exceptions | Residual duplicate roots are documented as explicit skip-tree entries; stale `tiny-keccak`, `getrandom 0.2`, and `graphql_client` exceptions were removed because they are no longer in the workspace graph | Conforms |
| Legacy `thiserror` reachability | The remaining `thiserror 1.0.69` line is reached only through the Android-target `jni -> rustls-platform-verifier -> reqwest` path; the former `graphql_client` dev/test chain was removed | Conforms |
| Native Alloy allow-lists | Shipped crates that depend on `alloy-provider` or `alloy-signer-local` are limited to the reviewed adapter crates and fail the policy-maintainer gate if the dependency escapes | Conforms |
| Native Alloy two-family lockfile | The workspace lockfile keeps reviewed Alloy runtime crates on `2.0.4` and Alloy Core ABI crates on `1.5.7`, with exactly one resolved version per listed crate | Conforms |
| Alloy canary failures | Scheduled canary failures are triaged as upstream-compatibility reports, with local pins changed only after ordinary quality gates pass and without dependency-policy waivers | Conforms |
| `cow-sdk-wasm` wasm32 tree | The wasm32 dependency graph excludes `cow-sdk-browser-wallet`, `cow-sdk-alloy*`, `alloy-provider`, reqwest, and hyper families; `tokio` is limited to the existing cancellation-token path | Conforms |
| Helper-module FFI boundary | The `cow-sdk-wasm::helpers` module remains independent of wasm-bindgen, `js-sys`, `web-sys`, and `serde-wasm-bindgen` | Conforms |
| Canonical primitive layer dependency closure | The workspace-level `sha3` and `num-bigint` declarations carry zero first-party direct production consumers and only resolve through `[dev-dependencies]` or transitive paths; the alloy-core ABI workspace pins, `httpdate`, and `serde_jcs` are consumed at the documented callsites per [ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md) | Conforms |
| `encode_prefixed` mechanical fence | The `.github/workflows/encode-prefixed-grep-gate.yml` workflow blocks the `format!("0x{}", alloy_primitives::hex::encode(...))` legacy hand-roll and unqualified `use alloy_primitives::hex::encode` imports in production sources, locking the canonical-primitive-layer hex-string contract from [ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md) | Conforms |
| Workspace dependency hygiene | The orphan `async-lock` workspace pin has been retired; no first-party crate referenced the pin and the lockfile no longer carries a first-party direct edge into the crate | Conforms |

## Current Contract

### Transport Advisory Remediation

The lockfile carries `rustls-webpki 0.103.13` across the reqwest transport
chain used by `cow-sdk-orderbook` and `cow-sdk-subgraph`, clearing the
reachable certificate-revocation-list parsing panic reported under
`RUSTSEC-2026-0104`. The reqwest pull chain into `rustls-platform-verifier`
resolves through the advisory-clean line without a workspace override.

### Published CID Posture

`cow-sdk-app-data` and `cow-sdk-core` now both reach the refreshed
published `cid 0.11.3` and `multihash 0.19.5` path through workspace
dependency pins. The two crates share the same lockfile resolution, so
the CID surface stays byte-for-byte equivalent across the workspace.
That published path removes the prior yanked `core2 0.4.0`
reachability, so the CID stack no longer owns any RustSec ignore
entries. The retired `core2` advisory `RUSTSEC-2026-0105` is not
tolerated by the current gate, and the former `RUSTSEC-2026-0097` `rand`
tolerance is retired because the lockfile resolves the affected path
through `rand 0.8.6`.

### Gate Contract

Routine CI and release-readiness apply the same split dependency contract:
`cargo deny check --config .github/config/deny.toml` owns policy on allowed
sources, licenses, curated duplicate-version tolerances, and yanked advisory
handling, while `cargo audit --deny unsound --deny unmaintained` blocks RustSec
vulnerabilities plus unsound and unmaintained advisories. Two identifiers are
currently tolerated with documented revisit triggers:

- `RUSTSEC-2024-0388` — `derivative` is reachable only through the
  `ark-ff` / `ruint` / `nybbles` / `alloy-trie` subtree pulled by the
  `alloy-consensus` native adapter family. The crate is a derive-macro helper
  and does not compile into runtime `cow-sdk` code. Revisit when the alloy
  consensus stack drops `derivative` or when an actively-maintained drop-in
  replacement lands upstream.
- `RUSTSEC-2024-0436` — covered by
  [Browser-Wallet Alloy Dependency Audit](browser-wallet-alloy-dependency-audit.md)

Each ignore lives in `.github/config/deny.toml` under `[advisories].ignore`.
The shared quality gate reads that TOML register at runtime and derives the
`cargo audit --ignore ...` arguments from it, so closure or addition of a
reviewed advisory has one committed source of truth. The
`docs-agree-on-release-gates` guard compares the public command examples
against the same canonical register and fails if any ignored RustSec token
lacks a matching rationale in this audit.

### Canonical Primitive Layer Closure

The workspace retains the `sha3 = "0.11.0"` and `num-bigint = "0.4.6"`
declarations at the workspace-dependencies level so the per-crate
`[dev-dependencies]` `sha3.workspace = true` declarations on
`cow-sdk-contracts`, `cow-sdk-signing`, and `cow-sdk-cow-shed` resolve
for their parity-oracle tests, and so the `num-bigint` transitive paths
through `jsonschema` (consumed by `cow-sdk-app-data`) resolve. No
first-party crate retains a production direct dependency on either
crate after the canonical primitive layer landed: the `cow-sdk-app-data`
digest path routes through `alloy_primitives::keccak256`, and the cow
`Amount` newtype wraps `alloy_primitives::U256` directly per
[ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md).
The alloy-core ABI workspace dependency family (`alloy-primitives`,
`alloy-sol-types`, `alloy-sol-macro`, `alloy-dyn-abi`, `alloy-json-abi`,
`alloy-serde`), `httpdate 1.0` (consumed by `cow-sdk-transport-policy`
to parse `Retry-After` HTTP-date headers), and `serde_jcs 0.2.0`
(consumed by `cow-sdk-app-data` for the RFC 8785 canonical JSON that
feeds the keccak256 digest input) are consumed at the callsites
enumerated by their respective per-surface audits.

The workspace has fully retired the upstream `hex` crate from its
dependency graph. No `[workspace.dependencies]` entry for `hex`
remains in the root `Cargo.toml`; no per-crate `[dependencies]` or
`[dev-dependencies]` entry for `hex.workspace = true` remains in any
first-party crate under `crates/`. Every production and test hex
encode and decode callsite across `crates/core/**`,
`crates/contracts/**`, `crates/signing/**`,
`crates/alloy-provider/**`, `crates/alloy-signer/**`,
`crates/alloy/**`, `crates/app-data/**`, `crates/trading/**`,
`crates/browser-wallet/**`, `crates/wasm/**`, and
`crates/cow-shed/**` routes through `alloy_primitives::hex::{encode,
decode}`, which resolves transitively to the `const-hex 1.18.x`
re-export carried by `alloy-primitives 1.5.x`. The
`ContractsError::DecodeHex { source }` variant carries the typed
`alloy_primitives::hex::FromHexError` value (a re-export of
`const_hex::FromHexError`) so the production error surface no longer
references the upstream `hex` crate's error type. The standalone
example workspace at `examples/native/Cargo.toml` carries its own
`alloy-primitives` declaration for the same canonical resolution and
no longer declares `hex` in either its workspace or its package
dependency block. The workspace lockfile contains a single `hex`
node, brought in transitively through `const-hex`'s wide compatibility
range, and is not a direct edge from any first-party manifest.

Every `format!("0x{}", alloy_primitives::hex::encode(...))` hand-roll
across the workspace has collapsed onto the single-call
`alloy_primitives::hex::encode_prefixed(...)` form anchored by ADR 0052.
The cascade touched twenty production call sites plus three sites
inside `#[cfg(test)] mod tests {}` blocks embedded in `src/`, spanning
the alloy adapter, contracts, app-data, trading, browser-wallet, and
wasm crates. The emitted hex strings remain byte-identical. The new
`.github/workflows/encode-prefixed-grep-gate.yml` workflow fences
future regression with two parallel jobs: the first rejects any
production-source `format!("0x{}", alloy_primitives::hex::encode(...))`
hand-roll on the gate's regex, and the second rejects unqualified
`use alloy_primitives::hex::encode` imports in production sources so
the call-site regex's coverage envelope stays honest. Both jobs filter
`//`-prefixed lines so doc-comment narratives that name the forbidden
symbol cannot self-trigger them.

The orphan `async-lock = "3.4.2"` workspace dependency declaration has
been retired from the root `Cargo.toml`. At the prior HEAD the pin had
zero first-party consumers: no `[dependencies]`, `[dev-dependencies]`,
or `[target.'cfg(...)'.dependencies]` table inside the eighteen
workspace members referenced the workspace pin. The lockfile node
retires on the next `cargo update`, removing one row of supply-chain
attack surface and one entry from the workspace default-feature
exception register.

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
remaining old line is reached only through the Android-target
`jni -> rustls-platform-verifier -> reqwest` path; it does not appear on the
host build graph. The earlier `graphql_client` schema-codegen chain that also
carried `thiserror 1.0.69` was removed together with the subgraph test-only
schema evidence. The release-facing gate keeps the path visible as
duplicate-version debt rather than hiding it behind an advisory tolerance.

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

The `cow-sdk-wasm::helpers` module is a pure Rust boundary for deterministic
wasm helper logic. Its FFI-neutrality test rejects JavaScript FFI imports so the helper module
does not pull wasm-bindgen concerns into reusable protocol code.

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
- `docs/verification.md`
- `docs/verification.md`
- `docs/audit/cid-dependency-audit.md`
- `docs/audit/browser-wallet-alloy-dependency-audit.md`
- `crates/wasm/Cargo.toml`
- `crates/wasm/Cargo.toml`
- `crates/wasm/tests/no_ffi_helpers.rs`
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
gh workflow run encode-prefixed-grep-gate.yml
cargo build --workspace --all-features
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-features --target wasm32-unknown-unknown
cargo test -p cow-rs-workspace-tests --test dependency_default_features_audit
cargo test -p cow-sdk-wasm --test no_ffi_helpers
bash scripts/check-release-docs-agree.sh
```
