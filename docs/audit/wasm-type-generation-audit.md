# WASM Type Generation Audit

Status: Current
Last reviewed: 2026-05-13
Owning surface: `cow-sdk-wasm` DTO exports, tsify-derived TypeScript declarations, and npm declaration snapshots
Refresh trigger: Changes to exported DTOs, `tsify` usage, wasm-pack targets, declaration snapshots, or package export targets
Related docs:
- [ADR 0039](../adr/0039-typescript-callable-wasm-sdk-surface.md)
- [ADR 0043](../adr/0043-callback-registry-internalization.md)
- [ADR 0046](../adr/0046-transport-policy-js-exposure.md)
- [ADR 0047](../adr/0047-typescript-facade-architecture.md)
- [WASM Surface Audit](wasm-surface-audit.md)
- [WASM Public API Stability Audit](wasm-public-api-stability-audit.md)
- [PROPERTIES.md](../../PROPERTIES.md)

## Scope

This audit covers:

- tsify-derived TypeScript declarations for wasm-bindgen exports
- host gating that keeps wasm-bindgen and tsify out of host-safe pure modules
- declaration snapshots for web, bundler, and nodejs wasm-pack targets
- facade declaration snapshots for public package flavors
- the package export-map gate that prevents stale declaration targets

It does not cover TypeScript consumer application type-checking beyond the
committed e2e fixtures.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Host gating | Pure helper modules compile natively without wasm-bindgen, JsValue, or tsify-derived public types | Conforms |
| DTO generation | Cross-ABI DTOs are generated from Rust types and exposed as TypeScript declarations | Conforms |
| Snapshot gate | Committed declarations for web, bundler, and nodejs targets detect export drift | Conforms |
| Facade snapshots | Public facade declarations hide raw wasm-bindgen internals and callback registry handles | Conforms |
| Package exports | Every declared npm export target exists and declaration files include required lib references | Conforms |
| Generated metadata | wasm-pack README and package metadata are removed from nested dist targets before verification | Conforms |

## Current Contract

### tsify Policy

Types that cross the wasm ABI live in the `exports` module tree and derive the
TypeScript declaration shape there. Host-safe protocol helpers live in the
`cow-sdk-pure-helpers` crate and are tested without wasm32-only dependencies.

### Snapshot Gate

The committed `crates/wasm/snapshots/raw/` declarations for web, bundler, and
nodejs represent the public TypeScript contract for default targets. A
declaration that uses `[Symbol.dispose]` must include the `esnext.disposable`
reference so editor and TypeScript compiler defaults do not report false
errors.

The committed `crates/wasm/snapshots/facade/` declarations represent the
consumer-facing package surface. They are checked separately from raw
wasm-bindgen snapshots so generated implementation classes do not become the
published TypeScript SDK contract.

### Package Export Verification

The package verification script recursively walks string and conditional
exports, asserts every package-relative target exists, rejects nested wasm-pack
metadata in `dist`, and checks declaration files for the disposable reference.

## Evidence

Primary implementation points:

- `crates/pure-helpers/src/`
- `crates/wasm/src/exports/dto/` (domain DTO modules)
- `crates/wasm/src/exports/callbacks.rs`
- `crates/wasm/src/exports/envelope.rs`
- `crates/wasm/snapshots/raw/cow_sdk_wasm_web.d.ts`
- `crates/wasm/snapshots/raw/cow_sdk_wasm_bundler.d.ts`
- `crates/wasm/snapshots/raw/cow_sdk_wasm_nodejs.d.ts`
- `crates/wasm/snapshots/facade/`
- `crates/wasm/npm/scripts/build.sh`
- `crates/wasm/npm/scripts/verify-exports.mjs`

Primary regression coverage:

- `crates/wasm/tests/host_pure_helpers.rs::typed_data_payload_matches_signing_module_output`
- `crates/wasm/tests/host_pure_helpers.rs::wasm_version_matches_package_version`
- `crates/wasm/tests/wasm_surface_contract.rs::order_typed_data_serializes_to_expected_js_shape`
- `crates/wasm/tests/wasm_surface_contract.rs::wasm_version_matches_crate_version`
- `crates/wasm/tests/wasm_error_abi_contract.rs::invalid_input_variant_round_trips`
- `crates/wasm/tests/wasm_envelope_contract.rs::envelope_serializes_schema_version_and_payload`
- `crates/wasm/tests/wasm_snapshot_surface_contract.rs::generated_type_declarations_version_errors_and_outputs`
- `crates/wasm/tests/wasm_snapshot_surface_contract.rs::generated_type_declarations_hide_callback_registry`
- `crates/wasm/tests/wasm_snapshot_surface_contract.rs::generated_type_declarations_expose_transport_policy_config_for_http_flavours`
- `crates/wasm/tests/wasm_facade_snapshot_contract.rs::facade_declarations_match_flavour_matrix`
- `crates/wasm/tests/wasm_facade_snapshot_contract.rs::facade_declarations_hide_raw_wasm_bindgen_surface`
- `crates/wasm/tests/wasm_fail_closed_contract.rs::flavour_descriptor_exposes_cloudflare_wasm_subpath`
- `e2e/wasm-typescript/tests/signing.spec.ts`

Validation surface:

```text
cargo test -p cow-sdk-wasm --test host_pure_helpers
wasm-pack test crates/wasm --headless --chrome
bash crates/wasm/npm/scripts/build.sh
node crates/wasm/npm/scripts/verify-exports.mjs
cargo test -p cow-sdk-wasm --test wasm_facade_snapshot_contract
```
