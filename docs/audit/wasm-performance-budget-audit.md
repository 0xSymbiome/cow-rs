# WASM Performance Budget Audit

Status: Current
Last reviewed: 2026-05-12
Owning surface: `cow-sdk-wasm` release profile, wasm optimization pass, flavor build outputs, and size-budget gate
Refresh trigger: Changes to wasm feature flavors, package build scripts, release profile size settings, wasm optimization flags, package export targets, or measured size budgets; Cloudflare Workers compressed-size limit changes
Related docs:
- [ADR 0044](../adr/0044-bundle-size-profile-and-flavor-builds.md)
- [ADR 0039](../adr/0039-typescript-callable-wasm-sdk-surface.md)
- [PROPERTIES.md](../../PROPERTIES.md)
- [cow-sdk-wasm Comparative Benchmark Validation Note](cow-sdk-wasm-comparative-benchmark-validation-note.md)

## Scope

This audit covers:

- feature-scoped wasm flavor builds for default, orderbook, signing, full, and
  Cloudflare targets
- package build scripts that render package exports and run a wasm optimization
  pass
- measured raw, brotli, and gzip size budgets recorded in the npm README
- the release gate that can fail when generated artifacts exceed their budgets

It does not cover live network latency, wallet popup latency, or third-party
application bundler behavior.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Flavor outputs | Package generation produces feature-scoped subpaths rather than one mandatory maximal import | Conforms |
| Size optimization | Release artifacts run through the size-oriented release profile and wasm optimization pass | Conforms |
| Budget evidence | The npm README records current raw, brotli, gzip, and gate values for each flavor | Conforms |
| Cloudflare budget | Cloudflare exposes a Worker-compatible facade and wasm module subpath with a dedicated gzip gate | Conforms |

## Current Contract

### Flavor Builds

The package keeps one installable package while exposing flavor-specific public
subpaths. Default, orderbook, signing, full, and Cloudflare outputs have their
own declarations and raw wasm snapshots. Public imports use those subpaths
instead of generated `dist/raw` paths.

### Size Gate

The package build and measurement scripts operate on generated package
artifacts. The current reviewed contract records raw, brotli, and gzip sizes
for each flavor, including the Cloudflare-specific gzip budget. The
cloudflare flavor's gzip budget is expressed as an explicit byte budget that
tracks Cloudflare's published Workers Free compressed-size limit (the
configured fail threshold is below the platform limit with safety margin to
avoid MB / MiB ambiguity).

### Comparative context

The bundle-size budgets enforced by the package release gate exist
independently of any comparison against the upstream `@cowprotocol/cow-sdk`
TypeScript SDK. For the comparative measurement of `cow-sdk-wasm` versus the
upstream TypeScript SDK at equivalent feature subsets, see the
[cow-sdk-wasm Comparative Benchmark Validation Note](cow-sdk-wasm-comparative-benchmark-validation-note.md).
The validation note documents the measured tradeoffs that inform when
`cow-sdk-wasm` is the appropriate choice and confirms that compiling the Rust
SDK to wasm32 produces a binary larger than the upstream TypeScript SDK at
equivalent feature subsets.

### Cloudflare runtime gates

Compressed-size compatibility is enforced on every release build, but full
Cloudflare Workers support depends on additional release-bundle and
startup-time gates that are tracked separately:

- Release-bundle verification with `wrangler deploy --dry-run`.
- Worker startup measurement against Cloudflare's 1-second startup limit
  (Wrangler reports `startup_time_ms` on deploy or version upload).

Both gates are listed as refresh-trigger items in the comparative benchmark
validation note. Cloudflare's published platform limits are at
`https://developers.cloudflare.com/workers/platform/limits/`.

### Optimization Boundary

The optimization pass is part of package generation and verification. It is a
release artifact contract, not a promise that every consumer bundler will
produce identical application bundles.

## Evidence

Primary implementation points:

- `Cargo.toml`
- `crates/wasm/Cargo.toml`
- `crates/wasm/npm/package.template.json`
- `crates/wasm/npm/scripts/build.sh`
- `crates/wasm/npm/scripts/measure-wasm-size.mjs`
- `crates/wasm/npm/scripts/render-package-json.mjs`
- `crates/wasm/npm/README.md`
- `crates/wasm/snapshots/raw/`

Primary regression coverage:

- `crates/wasm/tests/wasm_snapshot_surface_contract.rs::generated_type_declarations_match_flavour_matrix`
- `crates/wasm/tests/wasm_facade_snapshot_contract.rs::facade_declarations_match_flavour_matrix`
- `crates/wasm/tests/wasm_fail_closed_contract.rs::flavour_descriptor_exposes_cloudflare_wasm_subpath`

Validation surface:

```text
bash crates/wasm/npm/scripts/build.sh
node crates/wasm/npm/scripts/verify-exports.mjs
node crates/wasm/npm/scripts/measure-wasm-size.mjs
cargo test -p cow-sdk-wasm --test wasm_snapshot_surface_contract
cargo test -p cow-sdk-wasm --test wasm_facade_snapshot_contract
```
