# WASM Component Model Future Prep Audit

Status: Current
Last reviewed: 2026-05-13
Owning surface: `cow-sdk-pure-helpers` host-safe helper crate and the deterministic helper boundary consumed by `cow-sdk-wasm`
Refresh trigger: Changes to `crates/pure-helpers/**`, helper DTO envelopes, wasm helper exports, or any dependency that introduces JavaScript FFI into the pure-helper crate
Related docs:
- [ADR 0042](../adr/0042-pure-helpers-extraction.md)
- [ADR 0039](../adr/0039-typescript-callable-wasm-sdk-surface.md)
- [PROPERTIES.md](../../PROPERTIES.md)

## Scope

This audit covers:

- the pure-helper crate boundary used by TypeScript-callable wasm exports
- host-safe helper coverage for chains, app-data, typed-data, order UIDs,
  EIP-1271 payloads, and helper error formatting
- the absence of wasm-bindgen, `js-sys`, and `web-sys` imports from
  `cow-sdk-pure-helpers`
- the future adapter posture that keeps deterministic protocol logic separate
  from JavaScript ABI mechanics

It does not cover WebAssembly Component Model packaging or a published WASI
target.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Crate boundary | Deterministic helpers live in `cow-sdk-pure-helpers` and are consumed by `cow-sdk-wasm` | Conforms |
| FFI exclusion | The pure-helper crate does not import JavaScript or wasm-bindgen FFI crates | Conforms |
| Host parity | Host tests prove helper outputs match signing, app-data, and wasm-facing expectations | Conforms |
| Future adapter posture | A future component adapter can reuse pure helpers without inheriting facade or callback state | Conforms |

## Current Contract

### Pure Helper Boundary

`cow-sdk-pure-helpers` owns deterministic helper code that does not need a
JavaScript runtime. The wasm crate converts JavaScript inputs into those helper
types, maps helper errors into `WasmError`, and keeps ABI concerns in the wasm
export and facade layers.

### FFI Exclusion

The pure-helper source tree is checked for imports of `wasm_bindgen`, `js_sys`,
`web_sys`, and `serde_wasm_bindgen`. That keeps helper code suitable for host
tests and future non-JavaScript wasm adapters.

### Future Adapter Readiness

The current package does not claim WebAssembly Component Model support. The
reviewed contract is narrower: deterministic protocol helpers are isolated so
future adapters can reuse them without accepting callback registries, raw
wasm-bindgen output, or TypeScript facade state as protocol dependencies.

## Evidence

Primary implementation points:

- `crates/pure-helpers/src/`
- `crates/pure-helpers/Cargo.toml`
- `crates/pure-helpers/README.md`
- `crates/wasm/src/exports/chains.rs`
- `crates/wasm/src/exports/dto/` (core, app-data, signing, transport,
  orderbook, trading, contracts, and subgraph DTO modules)
- `crates/wasm/src/exports/eip1271.rs`
- `crates/wasm/src/exports/signing.rs`

Primary regression coverage:

- `crates/pure-helpers/tests/no_ffi_imports.rs::pure_helpers_do_not_import_ffi_bindings`
- `crates/wasm/tests/host_pure_helpers.rs::typed_data_payload_matches_signing_module_output`
- `crates/wasm/tests/host_pure_helpers.rs::generated_order_uid_uses_canonical_strings`
- `crates/wasm/tests/host_pure_helpers.rs::eip1271_payload_matches_signing_module_output_and_vector`
- `crates/wasm/tests/host_pure_helpers.rs::app_data_hex_and_cid_round_trip_for_two_vectors`

Validation surface:

```text
cargo test -p cow-sdk-pure-helpers
cargo test -p cow-sdk-wasm --test host_pure_helpers
cargo test --workspace --all-features
```
