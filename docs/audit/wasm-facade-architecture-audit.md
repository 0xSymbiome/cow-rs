# WASM Facade Architecture Audit

Status: Current
Last reviewed: 2026-05-11
Owning surface: TypeScript facade modules under `crates/wasm/npm/src/**` and their adaptation boundary over raw wasm-bindgen output
Refresh trigger: Changes to facade source, raw binding adapters, disposal behavior, facade declaration snapshots, raw export denylist, or package resolution tests
Related docs:
- [ADR 0039](../adr/0039-typescript-callable-wasm-sdk-surface.md)
- [ADR 0043](../adr/0043-callback-registry-internalization.md)
- [ADR 0054](../adr/0054-typescript-facade-architecture.md)

## Scope

This audit covers:

- the TypeScript facade as the public package contract
- raw wasm-bindgen output kept under package-internal adapter modules
- facade-owned resource cleanup and callback retention
- public declaration snapshots for default, orderbook, signing, full, and
  Cloudflare flavors
- package resolution tests that protect public import paths

It does not cover final package publication or third-party bundler plugin
configuration outside the repository fixtures.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Facade ownership | Public APIs are TypeScript facade classes and helpers, not raw wasm-bindgen classes | Conforms |
| Raw boundary | Raw binding imports remain package-internal and are excluded from public export paths | Conforms |
| Resource cleanup | Facade clients expose explicit disposal and hide raw resource-management members | Conforms |
| Runtime flavors | Default, orderbook, signing, full, and Cloudflare declarations match the flavor matrix | Conforms |
| Error normalization | Facade errors normalize raw wasm errors into `SdkError` envelopes | Conforms |

## Current Contract

### Facade Modules

The facade modules adapt raw wasm-bindgen output into stable TypeScript classes,
helpers, and config objects. Public users import from the package root or
flavor subpaths; they do not import generated `dist/raw` files.

### Internal Raw Adapters

Raw binding imports live behind facade modules and package-internal adapter
files. Verification scripts reject public raw export entries and declaration
snapshots assert that raw wasm-bindgen classes do not leak into facade
declarations.

### Cleanup And Errors

Facade clients own callback retention and expose explicit `dispose` behavior.
Errors crossing the facade are converted into `SdkError` values with
schema-versioned envelopes and redacted details.

## Evidence

Primary implementation points:

- `crates/wasm/npm/src/index.ts`
- `crates/wasm/npm/src/orderbook.ts`
- `crates/wasm/npm/src/signing.ts`
- `crates/wasm/npm/src/full.ts`
- `crates/wasm/npm/src/cloudflare.ts`
- `crates/wasm/npm/src/internal.ts`
- `crates/wasm/npm/src/raw/`
- `crates/wasm/npm/scripts/compile-facade.sh`
- `crates/wasm/npm/scripts/verify-no-raw-exports.mjs`
- `crates/wasm/npm/scripts/verify-package-resolution.sh`
- `crates/wasm/snapshots/facade/`

Primary regression coverage:

- `crates/wasm/tests/wasm_facade_snapshot_contract.rs::facade_declarations_match_flavour_matrix`
- `crates/wasm/tests/wasm_facade_snapshot_contract.rs::facade_declarations_hide_raw_wasm_bindgen_surface`
- `crates/wasm/tests/wasm_facade_snapshot_contract.rs::facade_declarations_expose_dispose_and_named_callback_types`
- `crates/wasm/npm/tests/facade-default.test.ts`
- `crates/wasm/npm/tests/facade-orderbook.test.ts`
- `crates/wasm/npm/tests/facade-signing.test.ts`
- `crates/wasm/npm/tests/facade-cancellation.test.ts`
- `crates/wasm/npm/tests/facade-resource-cleanup.test.ts`
- `crates/wasm/npm/tests/facade-error-normalization.test.ts`

Validation surface:

```text
cargo test -p cow-sdk-wasm --test wasm_facade_snapshot_contract
bash crates/wasm/npm/scripts/build.sh
node crates/wasm/npm/scripts/verify-no-raw-exports.mjs
node crates/wasm/npm/scripts/verify-facade-denylist.mjs
bash crates/wasm/npm/scripts/verify-package-resolution.sh
pnpm --dir crates/wasm/npm test
```
